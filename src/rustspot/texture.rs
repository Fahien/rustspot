// Copyright © 2021
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use std::{fs::File, path::Path};

use super::*;

fn to_gl_format(color_type: png::ColorType) -> gl::types::GLenum {
    match color_type {
        png::ColorType::Grayscale => todo!(),
        png::ColorType::RGB => gl::RGB,
        png::ColorType::Indexed => todo!(),
        png::ColorType::GrayscaleAlpha => todo!(),
        png::ColorType::RGBA => gl::RGBA,
    }
}

pub struct Texture {
    pub handle: u32,
    pub extent: Extent2D,
    pub path: Option<String>,
}

impl Texture {
    pub fn new() -> Texture {
        let mut handle: u32 = 0;
        unsafe { gl::GenTextures(1, &mut handle) };
        Texture {
            handle,
            extent: Extent2D::new(0, 0),
            path: None,
        }
    }

    pub fn attachment(
        extent: Extent2D,
        format: gl::types::GLenum,
        type_: gl::types::GLenum,
    ) -> Self {
        let mut texture = Self::new();
        texture.extent = extent;
        texture.bind();

        unsafe {
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                format as _,
                extent.width as _,
                extent.height as _,
                0,
                format,
                type_,
                std::ptr::null(),
            );

            // Clamping to border is important for the shadowmap
            gl::TexParameteri(
                gl::TEXTURE_2D,
                gl::TEXTURE_WRAP_S,
                gl::CLAMP_TO_BORDER as i32,
            );
            gl::TexParameteri(
                gl::TEXTURE_2D,
                gl::TEXTURE_WRAP_T,
                gl::CLAMP_TO_BORDER as i32,
            );

            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);

            let transparent: [f32; 4] = [1.0, 1.0, 1.0, 0.0];
            gl::TexParameterfv(gl::TEXTURE_2D, gl::TEXTURE_BORDER_COLOR, &transparent as _);
        }

        texture
    }

    pub fn color(extent: Extent2D) -> Self {
        Self::attachment(extent, gl::RGB, gl::UNSIGNED_BYTE)
    }

    pub fn depth(extent: Extent2D) -> Self {
        Self::attachment(extent, gl::DEPTH_COMPONENT, gl::UNSIGNED_SHORT)
    }

    /// Creates a one pixel texture with the RGBA color passed as argument
    pub fn pixel(data: Color) -> Self {
        let mut texture = Self::new();
        texture.upload(gl::RGBA, 1, 1, data.as_slice());
        texture
    }

    /// Loads a PNG image from a path and returns a new texture
    pub fn open<P: AsRef<Path>>(path: P) -> Texture {
        let str_path = path.as_ref().to_str().unwrap().to_string();
        let message = format!("Failed to open: {}", str_path);
        let decoder = png::Decoder::new(File::open(path).expect(&message));
        let (info, mut reader) = decoder.read_info().expect("Failed reading png info");
        let mut data: Vec<u8> = vec![0; info.buffer_size()];
        reader
            .next_frame(data.as_mut_slice())
            .expect("Failed to read png frame");

        let mut texture = Texture::new();
        texture.path = Some(str_path);
        texture.upload(
            to_gl_format(info.color_type),
            info.width,
            info.height,
            &data,
        );
        texture
    }

    pub fn bind(&self) {
        unsafe { gl::BindTexture(gl::TEXTURE_2D, self.handle) };
    }

    pub fn upload<T>(&mut self, format: gl::types::GLenum, width: u32, height: u32, data: &[T]) {
        self.extent.width = width;
        self.extent.height = height;

        self.bind();

        unsafe {
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                format as i32,
                width as i32,
                height as i32,
                0,
                format,
                gl::UNSIGNED_BYTE,
                &data[0] as *const T as _,
            );

            gl::GenerateMipmap(gl::TEXTURE_2D);

            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
        };
    }
}

impl Drop for Texture {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteTextures(1, &self.handle);
        }
    }
}

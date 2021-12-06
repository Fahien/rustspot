// Copyright © 2021
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use std::{
    error::Error,
    fs::File,
    path::{Path, PathBuf},
};

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

fn to_gl_renderable_format(format: gl::types::GLenum) -> gl::types::GLenum {
    match format {
        gl::RGB => gl::RGB8,
        gl::RGBA => gl::RGBA8,
        gl::DEPTH_COMPONENT => gl::DEPTH_COMPONENT16,
        _ => format,
    }
}

pub struct TextureBuilder<'a> {
    id: u32,
    format: gl::types::GLenum,
    extent: Extent2D,
    component: gl::types::GLenum,
    samples: u32,

    data: Option<&'a [u8]>,

    // Data loaded from file
    owned_data: Option<Vec<u8>>,
    path: Option<PathBuf>,
}

fn load_data<P: AsRef<Path>>(
    path: P,
) -> Result<(Extent2D, gl::types::GLenum, Vec<u8>), Box<dyn Error>> {
    let decoder = png::Decoder::new(File::open(&path)?);
    let (info, mut reader) = decoder.read_info()?;

    let mut data: Vec<u8> = vec![0; info.buffer_size()];
    let mut timer = Timer::new();
    reader.next_frame(data.as_mut_slice())?;

    let extent = Extent2D::new(info.width, info.height);
    let format = to_gl_format(info.color_type);

    println!(
        "Image {} ({:?}) leaded in {}",
        path.as_ref().to_string_lossy(),
        info.color_type,
        timer.get_delta().as_secs_f32()
    );
    Ok((extent, format, data))
}

impl<'a> TextureBuilder<'a> {
    pub fn new() -> Self {
        Self {
            id: 0,
            format: gl::RGBA,
            extent: Extent2D::new(1, 1),
            component: gl::UNSIGNED_BYTE,
            samples: 1,
            data: None,
            owned_data: None,
            path: None,
        }
    }

    pub fn id(mut self, id: u32) -> Self {
        self.id = id;
        self
    }

    pub fn format(mut self, format: gl::types::GLenum) -> Self {
        self.format = format;
        self
    }

    pub fn extent(mut self, extent: Extent2D) -> Self {
        self.extent = extent;
        self
    }

    pub fn component(mut self, component: gl::types::GLenum) -> Self {
        self.component = component;
        self
    }

    pub fn samples(mut self, samples: u32) -> Self {
        assert!(samples > 0);
        self.samples = samples;
        self
    }

    pub fn data(mut self, data: &'a [u8]) -> Self {
        self.data = Some(data);
        self
    }

    // As soon as this method is called, data from path is loaded in main memory
    pub fn path<P: AsRef<Path>>(mut self, path: P) -> Self {
        let (extent, format, data) = load_data(&path).expect(&format!(
            "Failed to load image from {}",
            path.as_ref().to_string_lossy()
        ));
        self.owned_data = Some(data);
        self.extent = extent;
        self.format = format;
        self.path = Some(path.as_ref().into());

        self
    }

    pub fn build(self) -> Result<Texture, Box<dyn Error>> {
        let mut ret = Texture::new(self.format, self.extent, self.component, self.samples);
        ret.id = self.id;

        ret.bind();

        if self.data.is_some() {
            ret.upload(self.data);
        } else if self.owned_data.is_some() {
            ret.path = self.path;
            ret.upload(Some(self.owned_data.as_ref().unwrap()));
        } else {
            ret.attachment();
        }

        ret.unbind();

        Ok(ret)
    }
}

pub struct Texture {
    pub handle: u32,
    pub id: u32,
    pub target: gl::types::GLenum,
    format: gl::types::GLenum,
    pub extent: Extent2D,
    component: gl::types::GLenum,
    pub samples: u32,
    pub path: Option<PathBuf>,
}

impl Texture {
    pub fn builder<'a>() -> TextureBuilder<'a> {
        TextureBuilder::new()
    }

    fn samples_as_target(samples: u32) -> gl::types::GLenum {
        if samples > 1 {
            gl::TEXTURE_2D_MULTISAMPLE
        } else {
            gl::TEXTURE_2D
        }
    }

    pub fn new(
        format: gl::types::GLenum,
        extent: Extent2D,
        component: gl::types::GLenum,
        samples: u32,
    ) -> Texture {
        let mut handle: u32 = 0;
        unsafe { gl::GenTextures(1, &mut handle) };

        Texture {
            handle,
            id: 0,
            target: Self::samples_as_target(samples),
            format,
            extent,
            component,
            samples,
            path: None,
        }
    }

    pub fn attachment(&mut self) {
        if self.samples == 1 {
            self.upload::<u8>(None);

            unsafe {
                // Clamping to border is important for the shadowmap
                gl::TexParameteri(self.target, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_BORDER as i32);
                gl::TexParameteri(self.target, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_BORDER as i32);

                gl::TexParameteri(self.target, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
                gl::TexParameteri(self.target, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);

                let transparent: [f32; 4] = [1.0, 1.0, 1.0, 0.0];
                gl::TexParameterfv(self.target, gl::TEXTURE_BORDER_COLOR, &transparent as _);
            }
        } else {
            let internal_format = to_gl_renderable_format(self.format);

            unsafe {
                if cfg!(feature = "gles") {
                    gl::TexStorage2DMultisample(
                        self.target,
                        self.samples as _,
                        internal_format,
                        self.extent.width as _,
                        self.extent.height as _,
                        gl::TRUE,
                    );
                } else {
                    gl::TexImage2DMultisample(
                        self.target,
                        self.samples as _,
                        internal_format,
                        self.extent.width as _,
                        self.extent.height as _,
                        gl::TRUE,
                    );
                }
            }
        }
    }

    pub fn color(extent: Extent2D, samples: u32) -> Self {
        Self::builder()
            .extent(extent)
            .samples(samples)
            .format(gl::RGB)
            .build()
            .unwrap()
    }

    pub fn depth(extent: Extent2D, samples: u32) -> Self {
        Self::builder()
            .extent(extent)
            .samples(samples)
            .format(gl::DEPTH_COMPONENT)
            .component(gl::UNSIGNED_SHORT)
            .build()
            .unwrap()
    }

    /// Creates a one pixel texture with the RGBA color passed as argument
    pub fn pixel(data: Color) -> Self {
        Self::builder().data(data.as_slice()).build().unwrap()
    }

    pub fn bind(&self) {
        unsafe {
            gl::BindTexture(self.target, self.handle);
        }
    }

    pub fn unbind(&self) {
        unsafe {
            gl::BindTexture(self.target, 0);
        }
    }

    fn upload<T>(&mut self, data: Option<&[T]>) {
        let data = if let Some(data) = data {
            &data[0] as *const T as _
        } else {
            std::ptr::null()
        };

        unsafe {
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                self.format as i32,
                self.extent.width as i32,
                self.extent.height as i32,
                0,
                self.format,
                self.component,
                data,
            );

            gl::GenerateMipmap(gl::TEXTURE_2D);

            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);
            gl::TexParameteri(
                gl::TEXTURE_2D,
                gl::TEXTURE_MIN_FILTER,
                gl::LINEAR_MIPMAP_LINEAR as i32,
            );
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
        }
    }
}

impl Drop for Texture {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteTextures(1, &self.handle);
        }
    }
}

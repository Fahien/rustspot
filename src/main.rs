// Copyright © 2020
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use std::ffi::CString;
use std::fs::File;
use std::io::Read;
use std::time::Instant;

use go2::*;
use nalgebra as na;

mod gfx;
mod util;
use gfx::*;
use util::*;

fn main() {
    // Initialize display, context, presenter, and gl symbols
    let display = Display::new().expect("Failed creating display");

    let attr = ContextAttributes {
        major: 3,
        minor: 2,
        red_bits: 8,
        green_bits: 8,
        blue_bits: 8,
        alpha_bits: 8,
        depth_bits: 24,
        stencil_bits: 0,
    };

    let context = Context::new(&display, 480, 320, &attr).expect("Failed creating context");
    context.make_current();

    let presenter = Presenter::new(&display, drm_sys::fourcc::DRM_FORMAT_RGB565, 0xFF080808)
        .expect("Failed creating presenter");

    unsafe {
        gl::load_with(|symbol| {
            eglGetProcAddress(CString::new(symbol).unwrap().as_ptr()) as *const _
        });
    };

    // Shaders
    let program = create_program("vert.glsl", "frag.glsl");

    // Create a primitive quad
    let mesh = Mesh::new(vec![Primitive::quad()]);

    // Use texture as a material for the mesh
    let texture = get_texture("res/img/fahien.png");

    let camera = Camera::perspective();
    let mut camera_node = Node::new();
    camera_node
        .model
        .append_translation_mut(&na::Translation3::new(0.0, 0.0, -1.0));

    let mut step = 0.5;
    let mut prev = Instant::now();

    let mut red = 0.0;

    let mut node = Node::new();

    loop {
        // Calculate delta time
        let now = Instant::now();
        let delta = now - prev;
        prev = now;

        // Update logic
        red += step * delta.as_secs_f32();
        if red > 1.0 || red < 0.0 {
            step = -step;
        }

        let rot = na::UnitQuaternion::from_axis_angle(&na::Vector3::y_axis(), delta.as_secs_f32());
        node.model.append_rotation_mut(&rot);

        // Render something
        unsafe {
            gl::ClearColor(red, 0.5, 1.0, 0.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

            program.enable();

            camera.bind(&camera_node);
            node.bind();
            texture.bind();
            mesh.bind();
            mesh.draw();
        }

        // Present to the screen
        context.swap_buffers();
        let surface = context.surface_lock();
        presenter.post(surface, 0, 0, 480, 320, 0, 0, 320, 480, 3);
        context.surface_unlock(surface);
        std::thread::sleep(std::time::Duration::from_millis(15));
    }
}

fn create_program(vert_path: &str, frag_path: &str) -> ShaderProgram {
    let mut vert_src = Vec::<u8>::new();
    let mut frag_src = Vec::<u8>::new();
    File::open(format!("res/shader/{}", vert_path))
        .expect("Failed to open vertex file")
        .read_to_end(&mut vert_src)
        .expect("Failed reading vertex file");
    File::open(format!("res/shader/{}", frag_path))
        .expect("Failed to open fragment file")
        .read_to_end(&mut frag_src)
        .expect("Failed reading fragment file");

    let vert = Shader::new(gl::VERTEX_SHADER, &vert_src).expect("Failed creating shader");
    let frag = Shader::new(gl::FRAGMENT_SHADER, &frag_src).expect("Failed creating shader");

    ShaderProgram::new(vert, frag)
}

/// Loads a PNG image from a path and returns a new texture
fn get_texture(path: &str) -> Texture {
    let decoder = png::Decoder::new(File::open(path).expect("Failed to open png"));
    let (info, mut reader) = decoder.read_info().expect("Failed reading png info");
    println!("Png {}\n{:?}", path, info);
    let mut data: Vec<u8> = vec![0; info.buffer_size()];
    reader
        .next_frame(data.as_mut_slice())
        .expect("Failed to read png frame");

    let texture = Texture::new();
    texture.bind();
    texture.upload(info.width, info.height, &data);
    unsafe {
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
    }

    texture
}

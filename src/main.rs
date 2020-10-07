// Copyright © 2020
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use std::ffi::CString;
use std::time::Instant;

use go2::*;

mod gfx;
use gfx::*;

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
    let vert_src = r#"#version 320 es
        layout (location = 0) in vec3 in_pos;
        void main() {
            gl_Position = vec4(in_pos, 1.0);
        }
    "#;

    let frag_src = r#"#version 320 es
        out mediump vec4 out_color;
        void main() {
            out_color = vec4(0.5, 0.1, 0.2, 1.0);
        }
    "#;

    let vert = Shader::new(gl::VERTEX_SHADER, vert_src).expect("Failed creating shader");
    let frag = Shader::new(gl::FRAGMENT_SHADER, frag_src).expect("Failed creating shader");
    let program = ShaderProgram::new(vert, frag);

    // Create a mesh with two triangles
    let vertices: Vec<f32> = vec![
        0.5, 0.5, 0.0, 0.5, -0.5, 0.0, -0.5, -0.5, 0.0, -0.5, 0.5, 0.0,
    ];
    let indices: Vec<u32> = vec![0, 1, 3, 1, 2, 3];
    let mesh = MeshRes::new(&vertices, &indices);

    let mut step = 0.5;
    let mut prev = Instant::now();

    let mut red = 0.0;

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

        // Render something
        unsafe {
            gl::ClearColor(red, 0.5, 1.0, 0.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

            program.enable();
            mesh.bind();
            gl::DrawElements(
                gl::TRIANGLES,
                indices.len() as gl::types::GLsizei,
                gl::UNSIGNED_INT,
                0 as *const std::ffi::c_void,
            );
        }

        // Present to the screen
        context.swap_buffers();
        let surface = context.surface_lock();
        presenter.post(surface, 0, 0, 480, 320, 0, 0, 320, 480, 3);
        context.surface_unlock(surface);
        std::thread::sleep(std::time::Duration::from_millis(15));
    }
}

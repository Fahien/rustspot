// Copyright © 2020
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use std::fs::File;
use std::io::Read;

use go2::*;
use nalgebra as na;

use rustspot::{gfx::*, util::*};

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

    let mut renderer = Renderer::new();

    let mut gui = imgui::Context::create();
    let mut gui_res = GuiRes::new(&mut gui.fonts());

    // Shaders
    let program = create_program("vert.glsl", "frag.glsl");

    // Create a primitive quad
    let mut mesh = Mesh::new(vec![Primitive::quad()]);
    mesh.name = String::from("quad");

    // Store mesh in a vector
    let mut meshes = Pack::new();
    let mesh = meshes.push(mesh);

    // Use texture as a material for the mesh
    let texture = get_texture("res/img/assets.png");

    let mut nodes = Pack::new();

    let camera = Camera::perspective();

    let mut camera_node = Node::new();
    camera_node.name = String::from("camera");
    camera_node
        .model
        .append_translation_mut(&na::Translation3::new(0.0, 0.0, -1.0));
    let camera_node = nodes.push(camera_node);

    let mut timer = Timer::new();

    let mut step = 0.5;
    let mut red = 0.0;

    let mut root = Node::new();
    root.name = String::from("root");

    let mut left = Node::new();
    left.name = String::from("left");
    left.model
        .append_translation_mut(&na::Translation3::new(-0.5, 0.0, 0.0));
    left.mesh = mesh;
    root.children.push(nodes.push(left));

    let mut right = Node::new();
    right.name = String::from("right");
    right
        .model
        .append_translation_mut(&na::Translation3::new(0.5, 0.0, 0.0));
    right.mesh = mesh;
    let right = nodes.push(right);
    root.children.push(right);

    let root = nodes.push(root);

    loop {
        // Calculate delta time
        let delta = timer.get_delta();

        // Update GUI
        let ui = gui.io_mut();
        ui.update_delta_time(delta);
        ui.display_size = [480.0, 320.0];

        // Update logic
        red += step * delta.as_secs_f32();
        if red > 1.0 || red < 0.0 {
            step = -step;
        }

        let rot = na::UnitQuaternion::from_axis_angle(&na::Vector3::y_axis(), delta.as_secs_f32());
        nodes
            .get_mut(&root)
            .unwrap()
            .model
            .append_rotation_mut(&rot);

        // Render something
        unsafe {
            gl::Enable(gl::BLEND);
            gl::BlendEquation(gl::FUNC_ADD);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
            gl::Disable(gl::CULL_FACE);
            gl::Disable(gl::DEPTH_TEST);
            gl::Disable(gl::SCISSOR_TEST);

            gl::ClearColor(red, 0.5, 1.0, 0.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }

        program.enable();

        camera.bind(nodes.get(&camera_node).unwrap());

        texture.bind();

        renderer.draw(&nodes, &root, &na::Isometry3::identity());
        renderer.present(&meshes, &nodes);

        // Render GUI
        let ui = gui.frame();

        // Draw gui here before drawing it
        imgui::Window::new(imgui::im_str!("RustSpot"))
            .size([300.0, 60.0], imgui::Condition::FirstUseEver)
            .build(&ui, || {
                ui.text("Hello world!");
            });

        gui_res.draw(ui);

        // Present to the screen
        context.swap_buffers();
        let surface = context.surface_lock();
        presenter.post(surface, 0, 0, 480, 320, 0, 0, 320, 480, 3);
        context.surface_unlock(surface);
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

    let mut texture = Texture::new();
    texture.upload(info.width, info.height, &data);
    texture
}

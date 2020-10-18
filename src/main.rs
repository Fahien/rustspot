// Copyright © 2020
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use nalgebra as na;

use rustspot::{gfx::*, util::*};

fn main() {
    let sdl = sdl2::init().expect("Failed to initialize SDL2");
    let mut events = sdl.event_pump().expect("Failed to initialize SDL2 events");

    let mut gfx = Gfx::new(&sdl);
    let gl_version = gfx.get_gl_version();
    println!("OpenGL v{}.{}", gl_version.0, gl_version.1);

    let mut gui = imgui::Context::create();
    let mut gui_res = GuiRes::new(&mut gui.fonts());

    let mut model = Model::new();

    // Shaders
    let program_handle = model.programs.push(ShaderProgram::open(
        "res/shader/vert.glsl",
        "res/shader/frag.glsl",
    ));

    let texture = model.textures.push(Texture::open("res/img/lena.png"));

    // Create a material with the previous texture
    let material = model.materials.push(Material::new(texture));

    // Create a primitive quad with the previous material
    let primitive = model.primitives.push(Primitive::quad(material));

    // Create a mesh with a primitive quad
    let mut mesh = Mesh::new(vec![primitive]);
    mesh.name = String::from("quad");
    let mesh = model.meshes.push(mesh);

    let camera = Camera::perspective();

    let mut camera_node = Node::new();
    camera_node.name = String::from("camera");
    camera_node
        .model
        .append_translation_mut(&na::Translation3::new(0.0, 0.0, -1.0));
    let camera_node = model.nodes.push(camera_node);

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
    root.children.push(model.nodes.push(left));

    let mut right = Node::new();
    right.name = String::from("right");
    right
        .model
        .append_translation_mut(&na::Translation3::new(0.5, 0.0, 0.0));
    right.mesh = mesh;
    let right = model.nodes.push(right);
    root.children.push(right);

    let root = model.nodes.push(root);

    'gameloop: loop {
        // Handle SDL2 events
        for event in events.poll_iter() {
            match event {
                sdl2::event::Event::Quit { .. } => break 'gameloop,
                _ => println!("{:?}", event),
            }
        }

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
        model.nodes
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

        let program = program_handle.get(&model.programs).unwrap();

        camera.bind(&program, model.nodes.get(&camera_node).unwrap());

        gfx.renderer.draw(
            &model,
            &root,
            &na::Isometry3::identity(),
        );
        gfx.renderer
            .present(&model);

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
        gfx.swap_buffers();
    }
}

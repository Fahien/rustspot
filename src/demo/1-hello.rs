// Copyright © 2020
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use nalgebra as na;

use rustspot::*;

fn main() {
    let mut spot = Spot::new();

    let mut gui = imgui::Context::create();
    let mut gui_res = GuiRes::new(spot.gfx.video.profile, &mut gui.fonts());

    let (mut model, root) = create_model(spot.gfx.video.profile);

    let mut timer = Timer::new();

    let mut step = 0.5;
    let mut red = 0.0;

    'gameloop: loop {
        // Handle SDL2 events
        for event in spot.events.poll_iter() {
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

        let rot =
            na::UnitQuaternion::from_axis_angle(&na::Vector3::y_axis(), delta.as_secs_f32() / 2.0);
        model.nodes.get_mut(&root).unwrap().trs.rotate(&rot);

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

        spot.gfx
            .renderer
            .draw(&model, &root, &na::Matrix4::identity());
        spot.gfx.renderer.present(&model);

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
        spot.gfx.swap_buffers();
    }
}

fn create_model(profile: sdl2::video::GLProfile) -> (Model, Handle<Node>) {
    let mut model = Model::new();

    // Shaders
    model.programs.push(ShaderProgram::open(
        profile,
        "res/shader/vert.glsl",
        "res/shader/frag.glsl",
    ));

    let fancy_shader = model.programs.push(ShaderProgram::open(
        profile,
        "res/shader/fancy_vert.glsl",
        "res/shader/fancy_frag.glsl",
    ));

    let texture = model.textures.push(Texture::open("res/img/lena.png"));

    // Create a material with the previous texture
    let material = model.materials.push(Material::new(texture));

    // Create a fancy material
    let mut fancy_material = Material::new(texture);
    fancy_material.shader = fancy_shader;
    let fancy_material = model.materials.push(fancy_material);

    // Create a primitive quad with the previous material
    let primitive = model.primitives.push(Primitive::quad(material));
    let fancy_primitive = model.primitives.push(Primitive::quad(fancy_material));

    // Create a mesh with a primitive quad
    let mut mesh = Mesh::new(vec![primitive]);
    mesh.name = String::from("quad");
    let mesh = model.meshes.push(mesh);

    // Create a fancy mesh
    let fancy_mesh = model.meshes.push(Mesh::new(vec![fancy_primitive]));

    let mut root = Node::new();
    root.name = String::from("root");

    let camera = model.cameras.push(Camera::perspective());

    let mut camera_node = Node::new();
    camera_node.name = String::from("camera");
    camera_node.camera = camera;
    camera_node.trs.translate(0.0, 0.0, 2.5);
    let camera_node = model.nodes.push(camera_node);
    root.children.push(camera_node);

    let mut top_left = Node::new();
    top_left.name = String::from("top_left");
    top_left.trs.translate(-0.5, 0.5, 0.0);
    top_left.mesh = mesh;
    root.children.push(model.nodes.push(top_left));

    let mut top_right = Node::new();
    top_right.name = String::from("top_right");
    top_right.trs.translate(0.5, 0.5, 0.0);
    top_right.mesh = fancy_mesh;
    let top_right = model.nodes.push(top_right);
    root.children.push(top_right);

    let mut bottom_right = Node::new();
    bottom_right.name = String::from("bottom_right");
    bottom_right.trs.translate(0.5, -0.5, 0.0);
    bottom_right.mesh = fancy_mesh;
    let bottom_right = model.nodes.push(bottom_right);
    root.children.push(bottom_right);

    let mut bottom_left = Node::new();
    bottom_left.name = String::from("bottom_left");
    bottom_left.trs.translate(-0.5, -0.5, 0.0);
    bottom_left.mesh = fancy_mesh;
    let bottom_left = model.nodes.push(bottom_left);
    root.children.push(bottom_left);

    let root = model.nodes.push(root);

    (model, root)
}

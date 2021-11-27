// Copyright © 2020
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use nalgebra as na;

use rustspot::*;

fn main() {
    let mut spot = Spot::builder().build();

    let (mut model, root) = create_model();

    let mut step = 0.5;
    let mut red = 0.0;

    'gameloop: loop {
        // Handle SDL2 events
        for event in spot.events.poll_iter() {
            spot.input.handle(&event);

            match event {
                sdl2::event::Event::Quit { .. } => break 'gameloop,
                _ => println!("{:?}", event),
            }
        }

        let delta = spot.update();

        // Update logic
        red += step * delta.as_secs_f32();
        if red > 1.0 || red < 0.0 {
            step = -step;
        }

        let rot =
            na::UnitQuaternion::from_axis_angle(&na::Vector3::y_axis(), delta.as_secs_f32() / 2.0);
        model.nodes.get_mut(root).unwrap().trs.rotate(&rot);

        spot.gfx
            .renderer
            .draw(&model, root, &na::Matrix4::identity());

        let frame = spot.gfx.next_frame();

        spot.gfx
            .renderer
            .render_geometry(&model, &frame.default_framebuffer);

        // Render GUI
        let ui = spot.gfx.gui.frame();

        // Draw gui here before drawing it
        imgui::Window::new(imgui::im_str!("RustSpot"))
            .size([300.0, 60.0], imgui::Condition::FirstUseEver)
            .build(&ui, || {
                ui.text("Hello world!");
            });

        spot.gfx.renderer.render_gui(ui, &frame.default_framebuffer);

        // Present to the screen
        spot.gfx.present(frame);

        spot.input.reset();
    }
}

fn create_model() -> (Model, Handle<Node>) {
    let mut model = Model::new();

    let texture = model.textures.push(Texture::open("res/img/lena.png"));

    // Create a material with the previous texture
    let material = model.materials.push(Material::new(texture));

    // Create a fancy material
    let mut fancy_material = Material::new(texture);
    fancy_material.shader = Shaders::FANCY;
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

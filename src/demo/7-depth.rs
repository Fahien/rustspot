// Copyright © 2021
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use nalgebra as na;

use rustspot::*;

fn main() {
    let width = 480;
    let height = 320;
    let mut spot = Spot::builder().width(width).height(height).build();

    let (mut model, root) = create_model();

    'gameloop: loop {
        // Handle SDL2 events
        for event in spot.events.poll_iter() {
            match event {
                sdl2::event::Event::Quit { .. } => break 'gameloop,
                _ => println!("{:?}", event),
            }
        }

        let delta = spot.update();

        let rot =
            na::UnitQuaternion::from_axis_angle(&na::Vector3::y_axis(), delta.as_secs_f32() / 2.0);
        model.nodes.get_mut(&root).unwrap().trs.rotate(&rot);

        // Render something
        spot.gfx
            .renderer
            .draw(&model, &root, &na::Matrix4::identity());

        let frame = spot.gfx.next_frame();
        spot.gfx
            .renderer
            .render_geometry(&model, &frame.geometry_buffer);

        // Render GUI
        let ui = spot.gfx.gui.frame();

        // Draw gui here before drawing it
        imgui::Window::new(imgui::im_str!("RustSpot"))
            .position([60.0, 60.0], imgui::Condition::FirstUseEver)
            .size([300.0, 60.0], imgui::Condition::FirstUseEver)
            .build(&ui, || {
                ui.text("Hello world!");
            });

        spot.gfx
            .renderer
            .blit_depth(&frame.geometry_buffer, &frame.default_framebuffer);

        spot.gfx.renderer.render_gui(ui, &frame.default_framebuffer);

        // Present to the screen
        spot.gfx.present(frame);
    }
}

fn create_model() -> (Model, Handle<Node>) {
    let mut model = Model::new();

    let texture = model.textures.push(Texture::open("res/img/lena.png"));

    // Create a material with the previous texture
    let material = model.materials.push(Material::new(texture));

    // Create a primitive quad with the previous material
    let primitive = model.primitives.push(Primitive::quad(material));

    // Create a mesh with a primitive quad
    let mut mesh = Mesh::new(vec![primitive]);
    mesh.name = String::from("quad");
    let mesh = model.meshes.push(mesh);

    let mut root = Node::new();
    root.name = String::from("root");

    let camera = model.cameras.push(Camera::perspective());

    let mut camera_node = Node::new();
    camera_node.name = String::from("camera");
    camera_node.camera = camera;
    camera_node.trs.translate(0.0, 0.0, 1.8);
    let camera_node = model.nodes.push(camera_node);
    root.children.push(camera_node);

    let mut quad = Node::new();
    quad.name = String::from("quad node");
    quad.mesh = mesh;
    root.children.push(model.nodes.push(quad));

    let root = model.nodes.push(root);

    (model, root)
}

// Copyright © 2020
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use nalgebra as na;

use rustspot::*;

fn main() {
    let mut spot = Spot::builder().build();

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
        imgui::Window::new(imgui::im_str!("Objects"))
            .size([300.0, 180.0], imgui::Condition::FirstUseEver)
            .build(&ui, || {
                ui.text(imgui::im_str!("materials: {}", model.materials.len()));
                ui.text(imgui::im_str!("primitives: {}", model.primitives.len()));
                ui.text(imgui::im_str!("meshes: {}", model.meshes.len()));
                ui.text(imgui::im_str!("nodes: {}", model.nodes.len()));
            });

        spot.gfx.renderer.render_gui(ui, &frame.default_framebuffer);

        // Present to the screen
        spot.gfx.present(frame);
    }
}

fn create_model() -> (Model, Handle<Node>) {
    let mut model = Model::new();

    let color_textures = vec![
        model
            .textures
            .push(Texture::pixel(Color::rgba(233, 225, 78, 255))), // yellow
        model
            .textures
            .push(Texture::pixel(Color::rgba(170, 221, 84, 255))), // green
        model
            .textures
            .push(Texture::pixel(Color::rgba(145, 209, 125, 255))),
        model
            .textures
            .push(Texture::pixel(Color::rgba(106, 174, 185, 255))), // cyan
        model
            .textures
            .push(Texture::pixel(Color::rgba(87, 137, 210, 255))), // blue
        model
            .textures
            .push(Texture::pixel(Color::rgba(103, 114, 194, 255))),
        model
            .textures
            .push(Texture::pixel(Color::rgba(110, 95, 162, 255))), // purple
        model
            .textures
            .push(Texture::pixel(Color::rgba(128, 102, 149, 255))),
        model
            .textures
            .push(Texture::pixel(Color::rgba(183, 105, 119, 255))), // red
        model
            .textures
            .push(Texture::pixel(Color::rgba(212, 103, 98, 255))),
        model
            .textures
            .push(Texture::pixel(Color::rgba(224, 138, 3, 255))), // orange
        model
            .textures
            .push(Texture::pixel(Color::rgba(236, 195, 79, 255))),
    ];

    // Create a material with the previous texture
    let mut materials = vec![];
    for texture in color_textures {
        materials.push(
            model.materials.push(
                Material::builder()
                    .texture(texture)
                    .shader(Shaders::UNLIT)
                    .build(),
            ),
        );
    }

    // Create a primitive quad with the previous material
    let mut primitives = vec![];
    for material in materials {
        primitives.push(model.primitives.push(Primitive::quad(material)));
    }

    // Create a mesh with a primitive quad
    let mut meshes = vec![];
    for primitive in primitives {
        meshes.push(model.meshes.push(Mesh::new(vec![primitive])));
    }

    // Nodes
    let mut root = Node::new();
    root.name = String::from("root");

    let camera = model.cameras.push(Camera::perspective());

    let mut camera_node = Node::new();
    camera_node.name = String::from("camera");
    camera_node.camera = camera;
    camera_node.trs.translate(0.0, 0.0, 8.0);
    let camera_node = model.nodes.push(camera_node);
    root.children.push(camera_node);

    // 12 columns
    for i in -6..6 {
        let mut node = Node::new();

        node.name = format!("column{}", i);
        node.trs.translate(i as f32, 0.0, 0.0);
        node.trs.scale(1.0, 8.0, 1.0);
        node.mesh = meshes[(i + 6) as usize];

        root.children.push(model.nodes.push(node));
    }

    let root = model.nodes.push(root);

    (model, root)
}

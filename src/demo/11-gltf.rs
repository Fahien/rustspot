// Copyright © 2021
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use std::error::Error;

use clap::Arg;
use nalgebra as na;

use rustspot::*;

fn main() -> Result<(), Box<dyn Error>> {
    let spot_builder = Spot::builder().arg(
        Arg::with_name("file")
            .short("f")
            .default_value("res/model/box/box.gltf"),
    );
    let (mut spot, matches) = spot_builder.build_with_matches();

    // Load gltf
    let file_path = matches.value_of("file").unwrap();
    let mut model = Model::builder(file_path)?.build()?;
    let mut terrain = Terrain::new(&mut model);
    terrain.set_scale(&mut model, 64.0);
    create_light(&mut model);
    let camera = create_camera(&mut model);

    let root = Handle::new(0);

    model.nodes.get_mut(root).unwrap().children.push(camera);
    //model
    //    .nodes
    //    .get_mut(root)
    //    .unwrap()
    //    .children
    //    .push(terrain.root);

    spot.gfx.renderer.sky.enabled = true;

    'gameloop: loop {
        // Handle SDL2 events
        for event in spot.events.poll_iter() {
            spot.input.handle(&event);

            match event {
                sdl2::event::Event::Quit { .. } => break 'gameloop,
                sdl2::event::Event::MouseMotion { xrel, yrel, .. } => {
                    let extent = spot.gfx.video.extent;

                    let node = model.nodes.get_mut(camera).unwrap();
                    let right = na::Unit::new_normalize(node.trs.get_right());
                    let y_rotation = na::UnitQuaternion::from_axis_angle(
                        &right,
                        4.0 * yrel as f32 / extent.height as f32,
                    );
                    node.trs.rotate(&y_rotation);

                    let z_rotation = na::UnitQuaternion::from_axis_angle(
                        &na::Vector3::y_axis(),
                        4.0 * -xrel as f32 / extent.width as f32,
                    );
                    node.trs.rotate(&z_rotation);
                }
                sdl2::event::Event::MouseWheel { y, .. } => {
                    let node = model.nodes.get_mut(camera).unwrap();
                    let forward = node.trs.get_forward().scale(0.125 * y as f32);
                    node.trs.translate(forward.x, forward.y, forward.z);
                }
                _ => println!("{:?}", event),
            }
        }

        let _delta = spot.update();

        spot.gfx
            .renderer
            .draw(&model, root, &na::Matrix4::identity());

        let frame = spot.gfx.next_frame();

        spot.gfx
            .renderer
            .render_shadow(&model, &frame.shadow_buffer);

        spot.gfx
            .renderer
            .draw(&model, root, &na::Matrix4::identity());

        spot.gfx
            .renderer
            .render_geometry(&model, &frame.geometry_buffer);

        spot.gfx
            .renderer
            .blit_color(&frame.geometry_buffer, &frame.default_framebuffer);

        // Render GUI
        let ui = spot.gfx.gui.frame();

        // Draw gui here before drawing it
        imgui::Window::new(imgui::im_str!("RustSpot"))
            .size([300.0, 60.0], imgui::Condition::FirstUseEver)
            .build(&ui, || {
                let root = &model.nodes[0];
                print_family(&ui, root, &model, "".to_string());
            });

        spot.gfx.renderer.render_gui(ui, &frame.default_framebuffer);

        // Present to the screen
        spot.gfx.present(frame);

        spot.input.reset()
    }

    Ok(())
}

fn print_family(ui: &imgui::Ui, node: &Node, model: &Model, indent: String) {
    ui.text(format!("{}{}: {}", indent, node.name, node.id));
    for &child in &node.children {
        let child = model.nodes.get(child).unwrap();
        let mut next_indent = indent.clone();
        next_indent.push_str(" ");
        print_family(ui, child, model, next_indent);
    }
}

fn create_light(model: &mut Model) {
    let light = model.directional_lights.push(DirectionalLight::color(
        244.0 / 255.0,
        233.0 / 255.0,
        205.0 / 255.0,
    ));

    let mut light_node = Node::builder()
        .id(model.nodes.len() as u32)
        .name("Light".to_string())
        .directional_light(light)
        .build();
    light_node.trs.rotate(&na::UnitQuaternion::from_axis_angle(
        &na::Vector3::x_axis(),
        -(0.2 + std::f32::consts::FRAC_PI_4 + std::f32::consts::FRAC_PI_8),
    ));
    light_node.trs.translate(2.0, 40.0, 16.0);

    let light_node = model.nodes.push(light_node);
    model
        .nodes
        .get_mut(Handle::new(0))
        .unwrap()
        .children
        .push(light_node);
}

fn create_camera(model: &mut Model) -> Handle<Node> {
    let camera = model.cameras.push(Camera::perspective());
    let mut camera_node = Node::builder()
        .id(model.nodes.len() as u32)
        .name("Camera".to_string())
        .camera(camera)
        .build();

    // camera_node.trs.rotate(&na::UnitQuaternion::from_axis_angle(
    //     &na::Vector3::x_axis(),
    //     -(0.2 + std::f32::consts::FRAC_PI_4 + std::f32::consts::FRAC_PI_8),
    // ));
    camera_node.trs.translate(0.0, 0.35, 0.0);
    model.nodes.push(camera_node)
}

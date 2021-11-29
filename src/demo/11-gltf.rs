// Copyright © 2021
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use std::error::Error;

use nalgebra as na;

use rustspot::*;

fn main() -> Result<(), Box<dyn Error>> {
    let mut spot = Spot::builder().width(480 * 2).height(320 * 2).build();
    let extent = spot.gfx.video.extent;

    // Load gltf
    let mut model = Model::builder("res/model/duck/duck.gltf")?.build()?;
    create_light(&mut model);
    let camera = create_camera(&mut model);

    let root = Handle::new(0);

    model.nodes.get_mut(root).unwrap().children.push(camera);

    'gameloop: loop {
        // Handle SDL2 events
        for event in spot.events.poll_iter() {
            spot.input.handle(&event);

            match event {
                sdl2::event::Event::Quit { .. } => break 'gameloop,
                sdl2::event::Event::MouseMotion { xrel, yrel, .. } => {
                    let node = model.nodes.get_mut(camera).unwrap();
                    let y_rotation = na::UnitQuaternion::from_axis_angle(
                        &na::Vector3::x_axis(),
                        yrel as f32 / extent.height as f32,
                    );
                    let z_rotation = na::UnitQuaternion::from_axis_angle(
                        &na::Vector3::y_axis(),
                        -xrel as f32 / extent.width as f32,
                    );
                    let rotation = y_rotation * z_rotation;
                    node.trs.rotate(&rotation);
                }
                sdl2::event::Event::MouseWheel { y, .. } => {
                    let node = model.nodes.get_mut(camera).unwrap();
                    let forward = node.trs.get_forward().scale(y as f32);
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
            .render_geometry(&model, &frame.default_framebuffer);

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
    let light = model
        .directional_lights
        .push(DirectionalLight::color(1.0, 1.0, 1.0));

    let mut light_node = Node::builder()
        .id(model.nodes.len() as u32)
        .name("Light".to_string())
        .directional_light(light)
        .build();
    light_node.trs.translate(2.0, 0.0, 8.0);
    light_node.trs.rotate(&na::UnitQuaternion::from_axis_angle(
        &na::Vector3::x_axis(),
        -std::f32::consts::FRAC_PI_8,
    ));

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
    camera_node.trs.rotate(&na::UnitQuaternion::from_axis_angle(
        &na::Vector3::x_axis(),
        -std::f32::consts::FRAC_PI_4,
    ));
    camera_node.trs.translate(0.5, 3.0, 2.0);
    model.nodes.push(camera_node)
}

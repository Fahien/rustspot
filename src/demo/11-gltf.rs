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
    let gltf_node = Handle::new(2);
    let mut terrain = Terrain::new(&mut model);
    terrain.set_scale(&mut model, 64.0);
    create_light(&mut model);
    let (camera, camera_node) = create_camera(&mut model);

    let root = Handle::new(0);

    model
        .nodes
        .get_mut(root)
        .unwrap()
        .children
        .push(camera_node);
    //model
    //    .nodes
    //    .get_mut(root)
    //    .unwrap()
    //    .children
    //    .push(terrain.root);

    spot.gfx.renderer.sky.enabled = true;

    let mut override_shader = spot.gfx.renderer.override_shader.clone();
    let mut occlusion_variant = PbrOcclusionVariant::Default;
    let mut metallic_roughness_variant = PbrMetallicRoughnessVariant::Default;
    let mut normal_variant = PbrNormalVariant::Default;
    let mut shadow_variant = PbrShadowVariant::Texture;

    'gameloop: loop {
        spot.gfx.renderer.override_shader = override_shader.clone();

        let delta = spot.update();

        // Handle SDL2 events
        for event in spot.events.poll_iter() {
            let extent = spot.gfx.video.get_drawable_extent();

            // Update camera
            {
                let camera = model.cameras.get_mut(camera).unwrap();
                *camera = Camera::perspective(extent.width as f32, extent.height as f32);
            }

            spot.input.handle(&event);

            match event {
                sdl2::event::Event::Quit { .. } => break 'gameloop,
                sdl2::event::Event::MouseMotion {
                    xrel,
                    yrel,
                    mousestate,
                    ..
                } => {
                    let node = model.nodes.get_mut(gltf_node).unwrap();

                    if mousestate.is_mouse_button_pressed(sdl2::mouse::MouseButton::Right) {
                        rotate_node(delta.as_secs_f32(), node, xrel as f32, yrel as f32);
                    } else if mousestate.is_mouse_button_pressed(sdl2::mouse::MouseButton::Middle) {
                        let x = delta.as_secs_f32() * 0.25 * xrel as f32;
                        let y = delta.as_secs_f32() * 0.25 * -yrel as f32;
                        node.trs.translate(x, y, 0.0);
                    }
                }
                sdl2::event::Event::MouseWheel { x, y, .. } => {
                    let node = model.nodes.get_mut(camera_node).unwrap();
                    let forward = node.trs.get_forward().scale(0.125 * y as f32);
                    node.trs.translate(forward.x, forward.y, forward.z);

                    rotate_node(delta.as_secs_f32(), node, x as f32, 0.0);
                }
                sdl2::event::Event::KeyDown {
                    keycode: Some(sdl2::keyboard::Keycode::Up),
                    ..
                } => {
                    spot.gfx.renderer.override_shader =
                        if let Some(override_shader) = spot.gfx.renderer.override_shader {
                            override_shader.next()
                        } else {
                            Some(Shaders::first())
                        };
                }
                sdl2::event::Event::KeyDown {
                    keycode: Some(sdl2::keyboard::Keycode::Down),
                    ..
                } => {
                    spot.gfx.renderer.override_shader =
                        if let Some(override_shader) = spot.gfx.renderer.override_shader {
                            override_shader.prev()
                        } else {
                            Some(Shaders::last())
                        };
                }
                _ => println!("{:?}", event),
            }
        }

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

                ui.separator();

                let shader = if override_shader.is_some() {
                    override_shader.as_ref().unwrap().as_str()
                } else {
                    "None"
                };
                ui.text(format!("Shader variant: {}", shader));

                ui.separator();

                let option = "Occlusion";
                ui.text(option);
                let mut selected_variant = occlusion_variant;
                for variant in PbrOcclusionVariant::all() {
                    if ui.radio_button(
                        &imgui::im_str!("{}::{}", option, variant.as_str()),
                        &mut selected_variant,
                        variant,
                    ) {
                        occlusion_variant = selected_variant;
                    }
                }

                ui.separator();
                let option = "MetallicRoughness";
                ui.text(option);
                let mut selected_variant = metallic_roughness_variant;
                for variant in PbrMetallicRoughnessVariant::all() {
                    if ui.radio_button(
                        &imgui::im_str!("{}::{}", option, variant.as_str()),
                        &mut selected_variant,
                        variant,
                    ) {
                        metallic_roughness_variant = selected_variant;
                    }
                }

                ui.separator();
                let option = "Normal";
                ui.text(option);
                let mut selected_variant = normal_variant;
                for variant in PbrNormalVariant::all() {
                    if ui.radio_button(
                        &imgui::im_str!("{}::{}", option, variant.as_str()),
                        &mut selected_variant,
                        variant,
                    ) {
                        normal_variant = selected_variant;
                    }
                }

                ui.separator();
                let option = "Shadow";
                ui.text(option);
                let mut selected_variant = shadow_variant;
                for variant in PbrShadowVariant::all() {
                    if ui.radio_button(
                        &imgui::im_str!("{}::{}", option, variant.as_str()),
                        &mut selected_variant,
                        variant,
                    ) {
                        shadow_variant = selected_variant;
                    }
                }

                override_shader.replace(
                    PBR_VARIANTS[occlusion_variant as usize][metallic_roughness_variant as usize]
                        [normal_variant as usize][shadow_variant as usize],
                );
            });

        spot.gfx.renderer.render_gui(ui, &frame.default_framebuffer);

        // Present to the screen
        spot.gfx.present(frame);

        spot.input.reset()
    }

    Ok(())
}

fn rotate_node(delta: f32, node: &mut Node, x: f32, y: f32) {
    let right = na::Unit::new_normalize(node.trs.get_right());
    let y_rotation = na::UnitQuaternion::from_axis_angle(&right, 4.0 * y * delta);
    node.trs.rotate(&y_rotation);

    let z_rotation = na::UnitQuaternion::from_axis_angle(&na::Vector3::y_axis(), 4.0 * x * delta);
    node.trs.rotate(&z_rotation);
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
    light_node.trs.translate(2.0, 3.0, 1.0);

    let light_node = model.nodes.push(light_node);
    model
        .nodes
        .get_mut(Handle::new(0))
        .unwrap()
        .children
        .push(light_node);
}

fn create_camera(model: &mut Model) -> (Handle<Camera>, Handle<Node>) {
    let camera = model.cameras.push(Camera::perspective(480.0, 320.0));
    let mut camera_node = Node::builder()
        .id(model.nodes.len() as u32)
        .name("Camera".to_string())
        .camera(camera)
        .build();

    // camera_node.trs.rotate(&na::UnitQuaternion::from_axis_angle(
    //     &na::Vector3::x_axis(),
    //     -(0.2 + std::f32::consts::FRAC_PI_4 + std::f32::consts::FRAC_PI_8),
    // ));
    camera_node.trs.translate(0.0, 0.5, 0.0);
    let node = model.nodes.push(camera_node);

    (camera, node)
}

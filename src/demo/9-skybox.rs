// Copyright © 2021
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use nalgebra as na;

use rustspot::*;

mod model;

fn main() {
    let scale = 1;
    let width = scale * 480;
    let height = scale * 320;
    let mut spot = Spot::builder().width(width).height(height).build();
    spot.gfx.renderer.sky.enabled = true;

    let (mut model, root) = create_model(spot.gfx.video.profile);

    let mut joysticks = vec![];

    'gameloop: loop {
        let delta = spot.update();

        // Handle SDL2 events
        for event in spot.events.poll_iter() {
            match event {
                sdl2::event::Event::Quit { .. } => break 'gameloop,
                sdl2::event::Event::MouseMotion { xrel, yrel, .. } => {
                    for node in model.nodes.iter_mut() {
                        if node.camera.valid() {
                        let y_rotation = na::UnitQuaternion::from_axis_angle(
                                &na::Vector3::x_axis(),
                                yrel as f32 / height as f32,
                            );
                            let z_rotation = na::UnitQuaternion::from_axis_angle(
                                &na::Unit::new_normalize(node.trs.get_forward()),
                                xrel as f32 / width as f32,
                            );
                            let rotation = y_rotation * z_rotation;
                            node.trs.rotate(&rotation);
                            break;
                        }
                    }
                }
                sdl2::event::Event::MouseWheel { y, .. } => {
                    for node in model.nodes.iter_mut() {
                        if node.camera.valid() {
                            let forward = node.trs.get_forward().scale(y as f32);
                            node.trs.translate(forward.x, forward.y, forward.z);
                            break;
                        }
                    }
                }
                sdl2::event::Event::JoyAxisMotion {
                    axis_idx, value, ..
                } => {
                    if axis_idx == 0 || axis_idx == 1 {
                        for node in model.nodes.iter_mut() {
                            if node.camera.valid() {
                                let axis = if axis_idx == 0 {
                                    na::Vector3::y_axis()
                                } else {
                                    na::Vector3::x_axis()
                                };
                                let angle =
                                    -(value as f32 / (32768.0 / 2.0)) as f32 * delta.as_secs_f32();
                                let rotation = na::UnitQuaternion::from_axis_angle(&axis, angle);
                                node.trs.rotate(&rotation);
                                break;
                            }
                        }
                    }
                }
                sdl2::event::Event::JoyDeviceAdded { which, .. } => {
                    let joystick = spot
                        .joystick
                        .open(which)
                        .expect("Failed to open controller");
                    joysticks.push(joystick);
                }
                _ => println!("{:?}", event),
            }
        }

        let rot =
            na::UnitQuaternion::from_axis_angle(&na::Vector3::y_axis(), delta.as_secs_f32() / 16.0);
        model.nodes.get_mut(&root).unwrap().trs.rotate(&rot);

        spot.gfx
            .renderer
            .draw(&model, &root, &na::Matrix4::identity());

        let frame = spot.gfx.next_frame();
        spot.gfx
            .renderer
            .render_shadow(&model, &frame.shadow_buffer);

        spot.gfx
            .renderer
            .draw(&model, &root, &na::Matrix4::identity());

        spot.gfx
            .renderer
            .render_geometry(&model, &frame.geometry_buffer);

        // Draw a simple triangle which cover the whole screen
        spot.gfx
            .renderer
            .blit_color(&frame.geometry_buffer, &frame.default_framebuffer);

        // Present to the screen
        spot.gfx.present(frame);
    }
}

fn create_model(profile: sdl2::video::GLProfile) -> (Model, Handle<Node>) {
    let mut model = Model::new();

    // Shaders
    model.programs.push(ShaderProgram::open(
        profile,
        "res/shader/light-shadow-vert.glsl",
        "res/shader/light-shadow-frag.glsl",
    ));

    let root = model::create_structure_scene(&mut model);

    (model, root)
}

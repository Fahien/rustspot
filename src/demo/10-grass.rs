// Copyright © 2021
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use nalgebra as na;

use rustspot::*;

mod grass;
use grass::*;

fn main() {
    let scale = 2;
    let width = scale * 480;
    let height = scale * 320;
    let mut spot = Spot::builder()
        .width(width)
        .height(height)
        .offscreen_width(width)
        .offscreen_height(height)
        .build();
    spot.gfx.renderer.sky.enabled = true;

    let mut grass = Grass::new();

    let mut joysticks = vec![];

    'gameloop: loop {
        let delta = spot.update();

        // Handle SDL2 events
        for event in spot.events.poll_iter() {
            match event {
                sdl2::event::Event::Quit { .. } => break 'gameloop,
                sdl2::event::Event::MouseMotion { xrel, yrel, .. } => {
                    let node = grass.model.nodes.get_mut(&grass.camera).unwrap();
                    let y_rotation = na::UnitQuaternion::from_axis_angle(
                        &na::Vector3::x_axis(),
                        yrel as f32 / height as f32,
                    );
                    let z_rotation = na::UnitQuaternion::from_axis_angle(
                        &na::Vector3::y_axis(),
                        -xrel as f32 / width as f32,
                    );
                    let rotation = y_rotation * z_rotation;
                    node.trs.rotate(&rotation);
                }
                sdl2::event::Event::MouseWheel { y, .. } => {
                    let node = grass.model.nodes.get_mut(&grass.camera).unwrap();
                    let forward = node.trs.get_forward().scale(y as f32);
                    node.trs.translate(forward.x, forward.y, forward.z);
                }
                sdl2::event::Event::JoyAxisMotion {
                    axis_idx, value, ..
                } => {
                    if axis_idx == 0 || axis_idx == 1 {
                        let node = grass.model.nodes.get_mut(&grass.camera).unwrap();
                        let axis = if axis_idx == 0 {
                            na::Vector3::y_axis()
                        } else {
                            na::Vector3::x_axis()
                        };
                        let angle = -(value as f32 / (32768.0 / 2.0)) as f32 * delta.as_secs_f32();
                        let rotation = na::UnitQuaternion::from_axis_angle(&axis, angle);
                        node.trs.rotate(&rotation);
                    }
                }
                sdl2::event::Event::JoyDeviceAdded { which, .. } => {
                    let joystick = spot
                        .joystick
                        .open(which)
                        .expect("Failed to open controller");
                    joysticks.push(joystick);
                }
                sdl2::event::Event::KeyDown {
                    keycode: Some(code),
                    ..
                } => {
                    use sdl2::keyboard::Keycode;
                    let scale = temple.terrain.get_scale() * if code == Keycode::Up {
                        2.0
                    } else if code == Keycode::Down {
                        0.5
                    } else {
                        1.0
                    };

                    temple.terrain.set_scale(&mut temple.model, scale);

                    let blades_per_unit = temple.terrain.get_instances_per_unit() as f32 * if code == Keycode::Right {
                        2.0
                    } else if code == Keycode::Left {
                        0.5
                    } else {
                        1.0
                    };

                    temple.terrain.set_instance_per_unit(&mut temple.model, blades_per_unit as u32);
                }
                _ => println!("{:?}", event),
            }
        }

        spot.gfx
            .renderer
            .draw(&grass.model, &grass.root, &na::Matrix4::identity());

        let frame = spot.gfx.next_frame();
        spot.gfx
            .renderer
            .render_shadow(&grass.model, &frame.shadow_buffer);

        spot.gfx
            .renderer
            .draw(&grass.model, &grass.root, &na::Matrix4::identity());

        spot.gfx
            .renderer
            .render_geometry(&grass.model, &frame.geometry_buffer);

        // Draw a simple triangle which cover the whole screen
        spot.gfx
            .renderer
            .blit_color(&frame.geometry_buffer, &frame.default_framebuffer);

        // Start a new GUI frame
        let ui = spot.gfx.gui.frame();

        // Build GUI here before drawing it
        imgui::Window::new(imgui::im_str!("Terrain"))
            .size([300.0, 180.0], imgui::Condition::FirstUseEver)
            .build(&ui, || {
                ui.text(imgui::im_str!("scale: {}", temple.terrain.get_scale()));
                ui.text(imgui::im_str!("blades per unit: {}", temple.terrain.get_instances_per_unit()));
                ui.text(imgui::im_str!("blades: {}", temple.terrain.get_instance_count()));
            });

        spot.gfx.renderer.draw_gui(ui);

        // Present to the screen
        spot.gfx.present(frame);
    }
}

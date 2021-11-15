// Copyright © 2021
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use nalgebra as na;

use rustspot::*;

mod model;

fn main() {
    let width = 480;
    let height = 320;
    let mut spot = Spot::builder().width(width).height(height).build();

    let (mut model, root) = create_model(spot.gfx.video.profile);

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
    let mut model = Model::new(profile);
    let root = model::create_structure_scene(&mut model);
    (model, root)
}
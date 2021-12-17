// Copyright © 2021
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use nalgebra as na;

use rustspot::*;

mod model;

#[derive(Clone, Copy, PartialEq, Eq)]
enum RenderSource {
    // Render the default source
    Default,

    // Render the shadowmap
    Shadowmap,
}

fn main() {
    let mut spot = Spot::builder().build();

    let (mut model, root) = create_model();

    let mut render_source = RenderSource::Default;

    'gameloop: loop {
        // Handle SDL2 events
        for event in spot.events.poll_iter() {
            spot.input.handle(&event);

            match event {
                sdl2::event::Event::Quit { .. } => break 'gameloop,
                _ => (),
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
            .render_shadow(&model, &frame.shadow_buffer);

        match render_source {
            RenderSource::Default => {
                spot.gfx
                    .renderer
                    .draw(&model, root, &na::Matrix4::identity());

                spot.gfx
                    .renderer
                    .render_geometry(&model, &frame.geometry_buffer);

                spot.gfx
                    .renderer
                    .blit_color(&frame.geometry_buffer, &frame.default_framebuffer);
            }

            RenderSource::Shadowmap => {
                spot.gfx
                    .renderer
                    .blit_depth(&frame.shadow_buffer, &frame.default_framebuffer);
            }
        }

        // Render GUI
        let ui = spot.gfx.gui.frame();
        // Draw gui here before drawing it
        imgui::Window::new(imgui::im_str!("RustSpot"))
            .size([300.0, 60.0], imgui::Condition::FirstUseEver)
            .build(&ui, || {
                ui.text("Render source");
                let mut value = render_source;
                if ui.radio_button(imgui::im_str!("Default"), &mut value, RenderSource::Default) {
                    render_source = value;
                }
                if ui.radio_button(
                    imgui::im_str!("Shadowmap"),
                    &mut value,
                    RenderSource::Shadowmap,
                ) {
                    render_source = value;
                }
            });
        spot.gfx.renderer.render_gui(ui, &frame.default_framebuffer);

        // Present to the screen
        spot.gfx.present(frame);

        // TODO greate a spot loop which automatically resets stuff?
        spot.input.reset();
    }
}

fn create_model() -> (Model, Handle<Node>) {
    let mut model = Model::new();
    let root = model::create_structure_scene(&mut model);
    (model, root)
}

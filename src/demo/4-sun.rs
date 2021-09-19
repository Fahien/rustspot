// Copyright © 2020
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use nalgebra as na;

use rustspot::*;

fn main() {
    let mut spot = Spot::new();

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
            na::UnitQuaternion::from_axis_angle(&na::Vector3::y_axis(), delta.as_secs_f32() / 2.0)
                * na::UnitQuaternion::from_axis_angle(
                    &na::Vector3::x_axis(),
                    delta.as_secs_f32() / 2.0,
                );
        model.nodes.get_mut(&root).unwrap().trs.rotate(&rot);

        // Render something
        unsafe {
            gl::Enable(gl::BLEND);
            gl::BlendEquation(gl::FUNC_ADD);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
            gl::Disable(gl::CULL_FACE);
            gl::Enable(gl::DEPTH_TEST);
            gl::Disable(gl::SCISSOR_TEST);

            gl::ClearColor(0.5, 0.5, 1.0, 0.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }

        spot.gfx
            .renderer
            .draw(&model, &root, &na::Matrix4::identity());
        spot.gfx.renderer.present(&model);

        // Present to the screen
        spot.gfx.swap_buffers();
    }
}

fn create_model(profile: sdl2::video::GLProfile) -> (Model, Handle<Node>) {
    let mut model = Model::new();

    // Shaders
    model.programs.push(ShaderProgram::open(
        profile,
        "res/shader/light-vert.glsl",
        "res/shader/light-frag.glsl",
    ));

    let color_textures = vec![
        model.textures.push(Texture::pixel(&[233, 225, 78, 255])), // yellow
        model.textures.push(Texture::pixel(&[170, 221, 84, 255])), // green
        model.textures.push(Texture::pixel(&[145, 209, 125, 255])),
        model.textures.push(Texture::pixel(&[106, 174, 185, 255])), // cyan
        model.textures.push(Texture::pixel(&[87, 137, 210, 255])),  // blue
        model.textures.push(Texture::pixel(&[103, 114, 194, 255])),
        model.textures.push(Texture::pixel(&[110, 95, 162, 255])), // purple
        model.textures.push(Texture::pixel(&[128, 102, 149, 255])),
        model.textures.push(Texture::pixel(&[183, 105, 119, 255])), // red
        model.textures.push(Texture::pixel(&[212, 103, 98, 255])),
        model.textures.push(Texture::pixel(&[224, 138, 3, 255])), // orange
        model.textures.push(Texture::pixel(&[236, 195, 79, 255])),
        model.textures.push(Texture::pixel(&[233, 225, 78, 255])), // yellow
    ];

    // Create a material with the previous texture
    let mut materials = vec![];
    for texture in color_textures {
        materials.push(model.materials.push(Material::new(texture)));
    }

    // Create a primitive quad with the previous material
    let mut primitives = vec![];
    for material in materials {
        primitives.push(model.primitives.push(Primitive::cube(material)));
    }

    // Create a mesh with a primitive quad
    let mut meshes = vec![];
    for primitive in primitives {
        meshes.push(model.meshes.push(Mesh::new(vec![primitive])));
    }

    // Nodes
    let mut root = Node::new();
    root.name = String::from("root");

    let light = model
        .directional_lights
        .push(DirectionalLight::color(1.0, 1.0, 1.0));
    let mut light_node = Node::new();
    light_node.trs.rotate(&na::UnitQuaternion::from_axis_angle(
        &na::Vector3::x_axis(),
        -std::f32::consts::FRAC_PI_4,
    ));
    light_node.directional_light = light;
    let light_node = model.nodes.push(light_node);
    root.children.push(light_node);

    let camera = model.cameras.push(Camera::perspective());

    let mut camera_node = Node::new();
    camera_node.name = String::from("camera");
    camera_node.camera = camera;
    camera_node.trs.translate(0.0, 0.0, 14.0);
    let camera_node = model.nodes.push(camera_node);
    root.children.push(camera_node);

    // 12 columns
    for i in 0..meshes.len() {
        let mut node = Node::new();

        node.name = format!("column{}", i);
        let x = (i as i32 - (meshes.len() as i32 / 2) as i32) as f32 + 0.0;
        node.trs.translate(x, 0.0, 0.0);
        let y_scale = -(i as i32 - (meshes.len() / 2) as i32).abs() + (meshes.len() as i32 / 2) + 1;
        node.trs.scale(1.0, y_scale as f32, 1.0);
        node.mesh = meshes[i as usize];

        root.children.push(model.nodes.push(node));
    }

    let root = model.nodes.push(root);

    (model, root)
}
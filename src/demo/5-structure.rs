// Copyright © 2020
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use nalgebra as na;

use rustspot::*;

fn main() {
    let mut spot = Spot::builder().build();

    let (mut model, root) = create_model(spot.gfx.video.profile);

    let mut step = 0.5;
    let mut red = 0.0;

    'gameloop: loop {
        // Handle SDL2 events
        for event in spot.events.poll_iter() {
            match event {
                sdl2::event::Event::Quit { .. } => break 'gameloop,
                _ => println!("{:?}", event),
            }
        }

        let delta = spot.update();

        // Update logic
        red += step * delta.as_secs_f32();
        if red > 1.0 || red < 0.0 {
            step = -step;
        }

        let rot =
            na::UnitQuaternion::from_axis_angle(&na::Vector3::y_axis(), delta.as_secs_f32() / 2.0);
        model.nodes.get_mut(&root).unwrap().trs.rotate(&rot);

        // Render something
        unsafe {
            gl::Enable(gl::BLEND);
            gl::BlendEquation(gl::FUNC_ADD);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
            gl::Enable(gl::CULL_FACE);
            gl::Enable(gl::DEPTH_TEST);
            gl::Disable(gl::SCISSOR_TEST);

            gl::ClearColor(0.3, 0.3, 0.3, 0.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }

        spot.gfx
            .renderer
            .draw(&model, &root, &na::Matrix4::identity());
        spot.gfx
            .renderer
            .present(&spot.gfx.default_framebuffer, &model);

        // Present to the screen
        spot.gfx.swap_buffers();
    }
}

fn create_face(model: &mut Model, mesh: Handle<Mesh>) -> Node {
    let thickness = 0.05;

    let mut left = Node::new();
    left.name = String::from("left");
    left.mesh = mesh;
    let y_scale = 1.0 - thickness;
    left.trs.scale(thickness, y_scale, thickness);
    left.trs.translate(-y_scale / 2.0, 0.0, y_scale / 2.0);

    let mut right = Node::new();
    right.name = String::from("right");
    right.mesh = mesh;
    right.trs.scale(thickness, y_scale, thickness);
    right.trs.translate(y_scale / 2.0, 0.0, y_scale / 2.0);

    let mut bottom = Node::new();
    bottom.name = String::from("bottom");
    bottom.mesh = mesh;
    bottom.trs.translate(0.0, -0.5, y_scale / 2.0);
    bottom.trs.scale(1.0, thickness, thickness);

    let mut top = Node::new();
    top.name = String::from("top");
    top.mesh = mesh;
    top.trs.translate(0.0, 0.5, y_scale / 2.0);
    top.trs.scale(1.0, thickness, thickness);

    let mut hor = Node::new();
    hor.children.push(model.nodes.push(left));
    hor.children.push(model.nodes.push(right));

    let mut ver = Node::new();
    ver.children.push(model.nodes.push(bottom));
    ver.children.push(model.nodes.push(top));

    let mut face = Node::new();
    face.children.push(model.nodes.push(hor));
    face.children.push(model.nodes.push(ver));

    face
}

fn create_structure(model: &mut Model, mesh: Handle<Mesh>) -> Node {
    let front_face = create_face(model, mesh);
    let mut right_face = create_face(model, mesh);
    right_face.trs.rotate(&na::UnitQuaternion::from_axis_angle(
        &na::Vector3::y_axis(),
        std::f32::consts::FRAC_PI_2,
    ));
    let mut back_face = create_face(model, mesh);
    back_face.trs.rotate(&na::UnitQuaternion::from_axis_angle(
        &na::Vector3::y_axis(),
        std::f32::consts::PI,
    ));
    let mut left_face = create_face(model, mesh);
    left_face.trs.rotate(&na::UnitQuaternion::from_axis_angle(
        &na::Vector3::y_axis(),
        -std::f32::consts::FRAC_PI_2,
    ));

    let mut structure = Node::new();
    structure.children.push(model.nodes.push(front_face));
    structure.children.push(model.nodes.push(right_face));
    structure.children.push(model.nodes.push(back_face));
    structure.children.push(model.nodes.push(left_face));

    structure
}

fn create_model(profile: sdl2::video::GLProfile) -> (Model, Handle<Node>) {
    let mut model = Model::new();

    // Shaders
    model.programs.push(ShaderProgram::open(
        profile,
        "res/shader/light-vert.glsl",
        "res/shader/light-frag.glsl",
    ));

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
    camera_node.trs.translate(0.0, 0.4, 5.5);
    let camera_node = model.nodes.push(camera_node);
    root.children.push(camera_node);

    // Cyan material
    let texture = model.textures.push(Texture::pixel(&[160, 170, 180, 255]));
    let material = model.materials.push(Material::new(texture));
    let primitives = vec![model.primitives.push(Primitive::cube(material))];
    let mesh = model.meshes.push(Mesh::new(primitives));

    let mut floor = Node::new();
    floor.name = String::from("floor");
    floor.mesh = mesh;
    floor.trs.translate(0.0, -0.6, 0.0);
    floor.trs.scale(4.0, 0.1, 4.0);
    root.children.push(model.nodes.push(floor));

    // White material
    let texture = model.textures.push(Texture::pixel(&[255, 255, 255, 255]));
    let material = model.materials.push(Material::new(texture));
    let primitives = vec![model.primitives.push(Primitive::cube(material))];
    let mesh = model.meshes.push(Mesh::new(primitives));

    // Super structure
    let mut super_struct = Node::new();

    let structure = create_structure(&mut model, mesh);
    super_struct.children.push(model.nodes.push(structure));

    let offset = -0.95;

    let mut structure = create_structure(&mut model, mesh);
    structure.trs.translate(0.0, 0.0, offset);
    super_struct.children.push(model.nodes.push(structure));

    let mut structure = create_structure(&mut model, mesh);
    structure.trs.translate(offset, 0.0, offset);
    super_struct.children.push(model.nodes.push(structure));

    let mut structure = create_structure(&mut model, mesh);
    structure.trs.translate(offset, 0.0, 0.0);
    super_struct.children.push(model.nodes.push(structure));

    let y = 1.0;

    let mut structure = create_structure(&mut model, mesh);
    structure.trs.translate(0.0, y, offset);
    super_struct.children.push(model.nodes.push(structure));

    let mut structure = create_structure(&mut model, mesh);
    structure.trs.translate(offset, y, offset);
    super_struct.children.push(model.nodes.push(structure));

    let mut structure = create_structure(&mut model, mesh);
    structure.trs.translate(offset, y, 0.0);
    super_struct.children.push(model.nodes.push(structure));

    super_struct.trs.translate(0.5, 0.0, 0.5);

    root.children.push(model.nodes.push(super_struct));

    let root = model.nodes.push(root);

    (model, root)
}

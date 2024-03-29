// Copyright © 2021
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use crate::*;
use nalgebra as na;

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

pub fn create_structure_scene(model: &mut Model) -> Handle<Node> {
    let mut root = Node::new();
    root.name = String::from("root");

    let light = model
        .directional_lights
        .push(DirectionalLight::color(1.0, 1.0, 1.0));
    let mut light_node = Node::new();
    light_node.trs.translate(2.0, 0.0, 4.0);
    light_node.trs.rotate(&na::UnitQuaternion::from_axis_angle(
        &na::Vector3::x_axis(),
        -std::f32::consts::FRAC_PI_4,
    ));
    light_node.directional_light = light;
    let light_node = model.nodes.push(light_node);
    root.children.push(light_node);

    let camera = model.cameras.push(Camera::perspective(480.0, 320.0));

    let mut camera_node = Node::new();
    camera_node.name = String::from("camera");
    camera_node.camera = camera;
    camera_node.trs.rotate(&na::UnitQuaternion::from_axis_angle(
        &na::Vector3::x_axis(),
        -0.56,
    ));
    camera_node.trs.translate(0.0, 3.0, 5.5);
    let camera_node = model.nodes.push(camera_node);
    root.children.push(camera_node);

    // Cyan material
    let texture = model
        .textures
        .push(Texture::pixel(Color::rgba(160, 170, 180, 255)));
    let material = Material::builder()
        .texture(texture)
        .shader(Shaders::PbrOcclusionDefaultMetallicRoughnessDefaultNormalDefaultShadowTexture)
        .build();
    let material = model.materials.push(material);
    let primitives = vec![model.primitives.push(Primitive::cube(material))];
    let mesh = model.meshes.push(Mesh::new(primitives));

    let mut floor = Node::new();
    floor.name = String::from("floor");
    floor.mesh = mesh;
    floor.trs.translate(0.0, -0.6, 0.0);
    floor.trs.scale(16.0, 0.1, 16.0);
    root.children.push(model.nodes.push(floor));

    // White material
    let texture = model
        .textures
        .push(Texture::pixel(Color::rgba(255, 255, 255, 255)));
    let material = Material::builder()
        .texture(texture)
        .shader(Shaders::PbrOcclusionDefaultMetallicRoughnessDefaultNormalDefaultShadowTexture)
        .build();
    let material = model.materials.push(material);
    let primitives = vec![model.primitives.push(Primitive::cube(material))];
    let mesh = model.meshes.push(Mesh::new(primitives));

    // Super structure
    let mut super_struct = Node::new();

    let structure = create_structure(model, mesh);
    super_struct.children.push(model.nodes.push(structure));

    let offset = -0.95;

    let mut structure = create_structure(model, mesh);
    structure.trs.translate(0.0, 0.0, offset);
    super_struct.children.push(model.nodes.push(structure));

    let mut structure = create_structure(model, mesh);
    structure.trs.translate(offset, 0.0, offset);
    super_struct.children.push(model.nodes.push(structure));

    let mut structure = create_structure(model, mesh);
    structure.trs.translate(offset, 0.0, 0.0);
    super_struct.children.push(model.nodes.push(structure));

    let y = 1.0;

    let mut structure = create_structure(model, mesh);
    structure.trs.translate(0.0, y, offset);
    super_struct.children.push(model.nodes.push(structure));

    let mut structure = create_structure(model, mesh);
    structure.trs.translate(offset, y, offset);
    super_struct.children.push(model.nodes.push(structure));

    let mut structure = create_structure(model, mesh);
    structure.trs.translate(offset, y, 0.0);
    super_struct.children.push(model.nodes.push(structure));

    super_struct.trs.translate(0.5, 0.0, 0.5);

    root.children.push(model.nodes.push(super_struct));

    model.nodes.push(root)
}

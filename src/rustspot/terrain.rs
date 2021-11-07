// Copyright © 2021
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use super::*;
use nalgebra as na;

pub struct Terrain {
    pub node: Handle<Node>,
}

impl Terrain {
    fn create_grass_blade(model: &mut Model) -> Handle<Node> {
        // Grass Shaders
        let grass_shader = model.programs.push(ShaderProgram::open(
            model.profile,
            "res/shader/light-grass-vert.glsl",
            "res/shader/light-grass-frag.glsl",
        ));

        let texture = model.textures.push(Texture::pixel(&[31, 100, 32, 255]));
        let mut material = Material::new(texture);
        material.shader = grass_shader;

        let material = model.materials.push(material);
        let mut primitive = Primitive::triangle(material);
        primitive.instance_count = 4096;
        let primitives = vec![model.primitives.push(primitive)];
        let mesh = model.meshes.push(Mesh::new(primitives));

        let mut blade = Node::new();
        blade.name = String::from("blade");
        blade.mesh = mesh;
        blade.trs.scale(0.125, 1.0, 1.0);

        model.nodes.push(blade)
    }

    fn create_ground(model: &mut Model) -> Handle<Node> {
        // Ground shader
        let ground_shader = model.programs.push(ShaderProgram::open(
            model.profile,
            "res/shader/light-vert.glsl",
            "res/shader/light-grass-frag.glsl",
        ));

        // Ground material
        let texture = model.textures.push(Texture::pixel(&[31, 100, 32, 255]));
        let mut material = Material::new(texture);
        material.shader = ground_shader;

        let material = model.materials.push(material);

        let primitives = vec![model.primitives.push(Primitive::quad(material))];
        let mesh = model.meshes.push(Mesh::new(primitives));

        let mut plane = Node::new();
        plane.name = String::from("plane");
        plane.mesh = mesh;
        plane.trs.scale(18.0, 18.0, 18.0);
        plane.trs.rotate(&na::UnitQuaternion::from_axis_angle(
            &na::Vector3::x_axis(),
            -std::f32::consts::FRAC_PI_2,
        ));
        let plane = model.nodes.push(plane);

        let mut ground = Node::new();
        ground.name = String::from("ground");
        ground.children.push(plane);
        ground.children.push(Self::create_grass_blade(model));

        model.nodes.push(ground)
    }

    pub fn new(model: &mut Model) -> Self {
        Self {
            node: Self::create_ground(model),
        }
    }
}

// Copyright © 2021
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use crate::*;
use nalgebra as na;
use sdl2::video::GLProfile;

pub struct Grass {
    pub camera: Handle<Node>,
    pub root: Handle<Node>,
    pub model: Model,
}

impl Grass {
    fn create_camera(model: &mut Model) -> Handle<Node> {
        let camera = model.cameras.push(Camera::perspective());

        let mut camera_node = Node::new();
        camera_node.name = String::from("camera");
        camera_node.camera = camera;
        camera_node.trs.rotate(&na::UnitQuaternion::from_axis_angle(
            &na::Vector3::x_axis(),
            -std::f32::consts::FRAC_PI_2,
        ));
        camera_node.trs.translate(0.0, 20.0, 0.0);

        model.nodes.push(camera_node)
    }

    fn create_light(model: &mut Model) -> Handle<Node> {
        let light = model
            .directional_lights
            .push(DirectionalLight::color(1.0, 1.0, 1.0));

        let mut light_node = Node::new();
        light_node.directional_light = light;
        light_node.trs.translate(2.0, 0.0, 8.0);
        light_node.trs.rotate(&na::UnitQuaternion::from_axis_angle(
            &na::Vector3::x_axis(),
            -std::f32::consts::FRAC_PI_4,
        ));

        model.nodes.push(light_node)
    }

    fn create_grass_blade(model: &mut Model, grass_shader: Handle<ShaderProgram>) -> Handle<Node> {
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
            "res/shader/light.vert.glsl",
            "res/shader/light-grass.frag.glsl",
        ));

        // Grass Shaders
        let grass_shader = model.programs.push(ShaderProgram::open(
            model.profile,
            "res/shader/light-grass.vert.glsl",
            "res/shader/light-grass.frag.glsl",
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
        ground
            .children
            .push(Self::create_grass_blade(model, grass_shader));

        model.nodes.push(ground)
    }

    pub fn new(profile: GLProfile) -> Self {
        let mut model = Model::new(profile);

        // Shaders
        model.programs.push(ShaderProgram::open(
            profile,
            "res/shader/light-shadow.vert.glsl",
            "res/shader/light-shadow.frag.glsl",
        ));

        let mut root = Node::new();

        let camera = Self::create_camera(&mut model);
        root.children.push(camera);

        root.children.push(Self::create_light(&mut model));
        root.children.push(Self::create_ground(&mut model));

        let root = model.nodes.push(root);

        Self {
            camera,
            root,
            model,
        }
    }
}

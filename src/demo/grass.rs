// Copyright © 2021
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use crate::*;
use nalgebra as na;
use sdl2::video::GLProfile;

pub struct Grass {
    pub camera: Handle<Node>,
    pub root: Handle<Node>,
    field: Terrain,
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

    pub fn new(profile: GLProfile) -> Self {
        let mut model = Model::new(profile);
        let field = Terrain::new(&mut model);

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
        root.children.push(field.node);

        let root = model.nodes.push(root);

        Self {
            camera,
            root,
            field,
            model,
        }
    }
}

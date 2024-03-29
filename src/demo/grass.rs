// Copyright © 2021
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use crate::*;
use nalgebra as na;

pub struct Grass {
    pub camera: Handle<Node>,
    pub root: Handle<Node>,
    pub terrain: Terrain,
    pub model: Model,
}

impl Grass {
    fn create_camera(model: &mut Model) -> Handle<Node> {
        let camera = model.cameras.push(Camera::perspective(480.0, 320.0));

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

    pub fn new() -> Self {
        let mut model = Model::new();
        let terrain = Terrain::new(&mut model);

        let mut root = Node::new();

        let camera = Self::create_camera(&mut model);
        root.children.push(camera);

        root.children.push(Self::create_light(&mut model));
        root.children.push(terrain.root);

        let root = model.nodes.push(root);

        Self {
            camera,
            root,
            terrain,
            model,
        }
    }
}

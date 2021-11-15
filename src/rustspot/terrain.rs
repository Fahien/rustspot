// Copyright © 2021
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use super::*;
use nalgebra as na;

const MARGIN: f32 = 2.0;
const BLADE_COUNT: u32 = 16;

pub struct Terrain {
    pub plane: Handle<Node>,
    pub grass: Handle<Node>,
    pub root: Handle<Node>,
    scale: f32,
    blades_per_unit: u32,
}

impl Terrain {
    fn create_grass_blade(model: &mut Model) -> Handle<Node> {
        let texture = model.textures.push(Texture::pixel(&[31, 100, 32, 255]));
        let mut material = Material::new(texture);
        material.shader = Shaders::LIGHTGRASS;

        let material = model.materials.push(material);
        let mut primitive = Primitive::triangle(material);
        primitive.instance_count = BLADE_COUNT;
        let primitives = vec![model.primitives.push(primitive)];
        let mesh = model.meshes.push(Mesh::new(primitives));

        let mut blade = Node::new();
        blade.name = String::from("blade");
        blade.mesh = mesh;
        blade.trs.scale(0.125, 1.0, 1.0);

        model.nodes.push(blade)
    }

    fn create_plane(model: &mut Model) -> Handle<Node> {
        // Plane material
        let texture = model.textures.push(Texture::pixel(&[31, 100, 32, 255]));
        let mut material = Material::new(texture);
        material.shader = Shaders::LIGHT;

        let material = model.materials.push(material);

        let primitives = vec![model.primitives.push(Primitive::quad(material))];
        let mesh = model.meshes.push(Mesh::new(primitives));

        let mut plane = Node::new();
        plane.name = String::from("plane");
        plane.mesh = mesh;
        plane.trs.scale(1.0 + MARGIN, 1.0 + MARGIN, 1.0 + MARGIN);
        plane.trs.rotate(&na::UnitQuaternion::from_axis_angle(
            &na::Vector3::x_axis(),
            -std::f32::consts::FRAC_PI_2,
        ));

        model.nodes.push(plane)
    }

    fn create_ground(model: &mut Model, plane: Handle<Node>, grass: Handle<Node>) -> Handle<Node> {
        let mut ground = Node::new();
        ground.name = String::from("ground");
        ground.children.push(plane);
        ground.children.push(grass);

        model.nodes.push(ground)
    }

    pub fn new(model: &mut Model) -> Self {
        let plane = Self::create_plane(model);
        let grass = Self::create_grass_blade(model);

        Self {
            plane,
            grass,
            root: Self::create_ground(model, plane, grass),
            scale: 1.0,
            blades_per_unit: 16,
        }
    }

    fn update_instance_count(&mut self, model: &mut Model) {
        let grass = model.nodes.get_mut(&self.grass).unwrap();
        let mesh = model.meshes.get_mut(&grass.mesh).unwrap();
        let primitive = model.primitives.get_mut(&mesh.primitives[0]).unwrap();
        primitive.instance_count = self.get_blade_count();
    }

    pub fn set_scale(&mut self, model: &mut Model, scale: f32) {
        self.scale = scale;

        let plane = model.nodes.get_mut(&self.plane).unwrap();
        plane
            .trs
            .set_scale(scale + MARGIN, scale + MARGIN, scale + MARGIN);

        self.update_instance_count(model);
    }

    pub fn get_scale(&self) -> f32 {
        self.scale
    }

    pub fn set_blade_per_unit(&mut self, model: &mut Model, blades_per_unit: u32) {
        self.blades_per_unit = blades_per_unit;
        self.update_instance_count(model);
    }

    pub fn get_blades_per_unit(&self) -> u32 {
        self.blades_per_unit
    }

    pub fn get_blade_count(&self) -> u32 {
        self.blades_per_unit * (self.scale as u32).pow(2)
    }
}

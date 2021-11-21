// Copyright © 2021
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use super::*;
use na::ComplexField;
use nalgebra as na;
use noise::{NoiseFn, Perlin};
use rayon::prelude::*;

const INSTANCE_MAX: u32 = 4096 * 4096;

pub struct Terrain {
    pub plane: Handle<Node>,
    pub grass: Handle<Node>,
    pub root: Handle<Node>,
    // Can I calculate this from instances per unit?
    scale: f32,
    instances_per_unit: u32,
}

impl Terrain {
    /// Create transform matrices for the instances
    fn create_transforms(&mut self) -> Vec<na::Matrix4<f32>> {
        let spread = 4.0 / self.instances_per_unit as f32;
        // That is to center the instance I guess
        let instance_offset = na::Vector3::new(spread / 2.0, 0.0, spread / 2.0);

        let instance_count = self.get_instance_count();
        let stride = (instance_count as f32).sqrt() as u32;
        // Used to put center of grid in origin
        let cell_offset = -na::Vector3::new(stride as f32 / 2.0, 0.0, stride as f32 / 2.0);

        let perlin = Perlin::new();

        let matrices = (0..instance_count)
            .into_par_iter()
            .map(|i| {
                let column = i % stride;
                let row = i / stride;

                // [0.0, 1.0]
                let random_x = perlin.get([228.24 * (i as f64), 654.56 * (i as f64)]);
                let random_z = perlin.get([310.85 * (i as f64), 142.98 * (i as f64)]);

                let random_weight = 1.0;
                // This is to slightly offset the instance from its cell position, just for a bit of natural chaos
                let random_offset =
                    random_weight * na::Vector3::new(random_x as f32, 0.0, random_z as f32);

                let translation = spread
                    * (na::Vector3::new(column as f32, 0.0, row as f32) + cell_offset)
                    + random_offset
                    + instance_offset;

                na::Matrix4::identity().append_translation(&translation)
            })
            .collect();

        matrices
    }

    fn create_grass_blade(model: &mut Model) -> Handle<Node> {
        let texture = model.textures.push(Texture::pixel(&[31, 100, 32, 255]));
        let mut material = Material::new(texture);
        material.shader = Shaders::LIGHTGRASS;

        let material = model.materials.push(material);
        let primitive = Primitive::triangle(material);
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
        plane.trs.scale(3.0, 3.0, 3.0);
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

        let mut ret = Self {
            plane,
            grass,
            root: Self::create_ground(model, plane, grass),
            scale: 1.0,
            instances_per_unit: 16,
        };
        ret.update_instance_count(model);
        ret
    }

    fn update_instance_count(&mut self, model: &mut Model) {
        let transforms = self.create_transforms();
        let grass = model.nodes.get_mut(&self.grass).unwrap();
        grass.transforms = transforms;
    }

    fn update_plane_scale(&mut self, model: &mut Model) {
        let plane = model.nodes.get_mut(&self.plane).unwrap();
        let margin = 2.0;
        plane.trs.set_scale(
            self.scale + margin,
            self.scale + margin,
            self.scale + margin,
        );
    }

    pub fn set_scale(&mut self, model: &mut Model, scale: f32) {
        let new_instance_count = Self::instance_count(scale as u32, self.instances_per_unit);
        if new_instance_count > INSTANCE_MAX {
            return ();
        }

        self.scale = scale;
        self.update_instance_count(model);
        self.update_plane_scale(model);
    }

    pub fn get_scale(&self) -> f32 {
        self.scale
    }

    pub fn set_instance_per_unit(&mut self, model: &mut Model, instances_per_unit: u32) {
        let instances_per_unit = std::cmp::max(1, instances_per_unit);
        let new_instance_count = Self::instance_count(self.scale as u32, instances_per_unit);
        if new_instance_count > INSTANCE_MAX {
            return ();
        }

        self.instances_per_unit = instances_per_unit;
        self.update_instance_count(model);
        self.update_plane_scale(model);
    }

    pub fn get_instances_per_unit(&self) -> u32 {
        self.instances_per_unit
    }

    pub fn instance_count(scale: u32, instances_per_unit: u32) -> u32 {
        std::cmp::max(1, instances_per_unit * scale.pow(2))
    }

    pub fn get_instance_count(&self) -> u32 {
        Self::instance_count(self.scale as u32, self.instances_per_unit)
    }
}

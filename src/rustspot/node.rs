// Copyright © 2021
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use nalgebra as na;

use crate::*;

pub struct NodeBuilder {
    pub id: u32,
    pub name: String,
    pub translation: na::Translation3<f32>,
    pub rotation: na::UnitQuaternion<f32>,
    pub scale: na::Vector3<f32>,
    pub matrix: na::Matrix4<f32>,
    pub children: Vec<Handle<Node>>,
}

impl NodeBuilder {
    pub fn new() -> Self {
        Self {
            id: 0,
            name: "Unknown".to_string(),
            translation: na::Translation3::new(0.0, 0.0, 0.0),
            rotation: na::UnitQuaternion::default(),
            scale: na::Vector3::new(1.0, 1.0, 1.0),
            matrix: na::Matrix4::identity(),
            children: vec![],
        }
    }

    pub fn id(mut self, id: u32) -> Self {
        self.id = id;
        self
    }

    pub fn name(mut self, name: String) -> Self {
        self.name = name;
        self
    }

    pub fn translation(mut self, translation: na::Translation3<f32>) -> Self {
        self.translation = translation;
        self
    }

    pub fn rotation(mut self, rotation: na::UnitQuaternion<f32>) -> Self {
        self.rotation = rotation;
        self
    }

    pub fn scale(mut self, scale: na::Vector3<f32>) -> Self {
        self.scale = scale;
        self
    }

    pub fn matrix(mut self, matrix: na::Matrix4<f32>) -> Self {
        self.matrix = matrix;
        self
    }

    pub fn children(mut self, children: Vec<Handle<Node>>) -> Self {
        self.children = children;
        self
    }

    pub fn build(self) -> Node {
        let mut node = Node::new();
        node.id = self.id;
        node.name = self.name;
        node.trs.set_scale(self.scale.x, self.scale.y, self.scale.z);
        node.trs.rotate(&self.rotation);
        node.trs
        .translate(self.translation.x, self.translation.y, self.translation.z);

        node.children = self.children;
        node
    }
}

#[derive(Clone)]
pub struct Node {
    pub id: u32,
    pub name: String,
    pub trs: Trs,
    pub mesh: Handle<Mesh>,
    /// Transform matrices when it needs to draw instanced meshes.
    pub transforms: Vec<na::Matrix4<f32>>,
    pub directional_light: Handle<DirectionalLight>,
    pub point_light: Handle<PointLight>,
    pub camera: Handle<Camera>,
    pub children: Vec<Handle<Node>>,
}

impl Node {
    pub fn builder() -> NodeBuilder {
        NodeBuilder::new()
    }

    pub fn new() -> Self {
        Node {
            id: 0,
            name: String::new(),
            trs: Trs::new(),
            mesh: Handle::none(),
            transforms: vec![],
            directional_light: Handle::none(),
            point_light: Handle::none(),
            camera: Handle::none(),
            children: vec![],
        }
    }

    pub fn bind(&self, program: &ShaderProgram, transform: &na::Matrix4<f32>) {
        let intr = transform
            .remove_column(3)
            .remove_row(3)
            .try_inverse()
            .unwrap();
        unsafe {
            gl::UniformMatrix4fv(program.loc.model, 1, gl::FALSE, transform.as_ptr());
            if program.loc.model_intr >= 0 {
                gl::UniformMatrix3fv(program.loc.model_intr, 1, gl::TRUE, intr.as_ptr());
            }
        }
    }
}

impl std::fmt::Debug for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Node {}", self.name)
    }
}

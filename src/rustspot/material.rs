// Copyright © 2021
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use crate::*;

pub struct MaterialBuilder {
    shader: Shaders,
    texture: Option<Handle<Texture>>,
    normals: Option<Handle<Texture>>,

    metallic: f32,
    roughness: f32,
}

impl MaterialBuilder {
    pub fn new() -> Self {
        Self {
            shader: Shaders::DEFAULT,
            texture: None,
            normals: None,
            metallic: 1.0,
            roughness: 1.0,
        }
    }

    pub fn shader(mut self, shader: Shaders) -> Self {
        self.shader = shader;
        self
    }

    pub fn texture(mut self, texture: Handle<Texture>) -> Self {
        self.texture = Some(texture);
        self
    }

    pub fn normals(mut self, normals: Handle<Texture>) -> Self {
        self.normals = Some(normals);
        self
    }

    pub fn metallic(mut self, metallic: f32) -> Self {
        self.metallic = metallic;
        self
    }

    pub fn roughness(mut self, roughness: f32) -> Self {
        self.roughness = roughness;
        self
    }

    pub fn build(self) -> Material {
        let mut material = Material::new();
        material.shader = self.shader;
        material.texture = self.texture;
        material.metallic = self.metallic;
        material.roughness = self.roughness;
        material
    }
}

pub struct Material {
    pub shader: Shaders,
    pub color: Color,
    pub texture: Option<Handle<Texture>>,
    pub normals: Option<Handle<Texture>>,

    // PBR factors
    pub metallic: f32,
    pub roughness: f32,
}

impl Material {
    pub fn builder() -> MaterialBuilder {
        MaterialBuilder::new()
    }

    pub fn new() -> Self {
        Self {
            shader: Shaders::DEFAULT,
            color: Color::new(),
            texture: None,
            normals: None,
            metallic: 1.0,
            roughness: 1.0,
        }
    }
}

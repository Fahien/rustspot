// Copyright © 2021
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use std::collections::HashMap;

use crate::*;

pub struct MaterialBuilder {
    shader: Shaders,
    texture: Option<Handle<Texture>>,
    normals: Option<Handle<Texture>>,
}

impl MaterialBuilder {
    pub fn new() -> Self {
        Self {
            shader: Shaders::DEFAULT,
            texture: None,
            normals: None,
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

    pub fn build(self) -> Material {
        let mut material = Material::new();
        material.shader = self.shader;
        material.texture = self.texture;
        material
    }
}

pub struct Material {
    pub shader: Shaders,
    pub color: Color,
    pub texture: Option<Handle<Texture>>,
    pub normals: Option<Handle<Texture>>,
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
        }
    }

    pub fn bind(&self, textures: &Pack<Texture>, colors: &HashMap<Color, Texture>) {
        // Bind albedo map
        if let Some(texture_handle) = self.texture {
            textures.get(texture_handle).unwrap().bind();
        } else {
            colors.get(&self.color).unwrap().bind();
        }

        // Bind normal map
        if let Some(normals_handle) = self.normals {
            unsafe {
                gl::ActiveTexture(gl::TEXTURE0 + 2);
            }
            textures.get(normals_handle).unwrap().bind();
            unsafe {
                gl::ActiveTexture(gl::TEXTURE0);
            }
        }
    }
}

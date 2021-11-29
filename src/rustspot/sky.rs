// Copyright © 2021
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use super::*;

struct SkyLoc {
    horizon: i32,
    zenit: i32,
}

impl SkyLoc {
    fn new(shader: &ShaderProgram) -> Self {
        let horizon = shader.get_uniform_location("horizon");
        let zenit = shader.get_uniform_location("zenit");
        Self { horizon, zenit }
    }

    fn set_horizon(&self, color: &[f32; 3]) {
        unsafe { gl::Uniform3fv(self.horizon, 1, color.as_ptr()) }
    }

    fn set_zenit(&self, color: &[f32; 3]) {
        unsafe { gl::Uniform3fv(self.zenit, 1, color.as_ptr()) }
    }

    fn set_color(&self, color: &SkyColor) {
        self.set_horizon(&color.horizon);
        self.set_zenit(&color.zenit);
    }
}

#[repr(C)]
pub struct SkyColor {
    horizon: [f32; 3],
    zenit: [f32; 3],
}

impl SkyColor {
    pub fn new(horizon: [f32; 3], zenit: [f32; 3]) -> Self {
        Self { horizon, zenit }
    }
}

pub struct Sky {
    colors: Vec<SkyColor>,
    primitive: Primitive,
    pub enabled: bool,
}

impl Sky {
    pub fn new() -> Sky {
        let colors = vec![SkyColor::new(
            [254.0 / 255.0, 254.0 / 255.0, 202.0 / 255.0],
            [98.0 / 255.0, 203.0 / 255.0, 251.0 / 255.0],
        )];

        let primitive = Primitive::quad(Handle::none());

        Sky {
            colors,
            primitive,
            enabled: false,
        }
    }

    pub fn draw(&self, shader: &SkyShader, camera: &Node) {
        unsafe {
            gl::Disable(gl::CULL_FACE);
            gl::DepthFunc(gl::LEQUAL);
        }

        shader.bind();

        unsafe {
            gl::Uniform3fv(shader.loc.horizon, 1, self.colors[0].horizon.as_ptr());
            gl::Uniform3fv(shader.loc.zenit, 1, self.colors[0].zenit.as_ptr());
        }

        let transform = camera.trs.get_matrix();
        shader.bind_node(&camera, &transform);
        self.primitive.bind();
        self.primitive.draw();
    }
}

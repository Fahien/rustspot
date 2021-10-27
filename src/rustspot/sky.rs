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
    shader: ShaderProgram,
    loc: SkyLoc,
    colors: Vec<SkyColor>,
    primitive: Primitive,
    pub enabled: bool,
}

impl Sky {
    pub fn new(profile: sdl2::video::GLProfile) -> Sky {
        let shader = ShaderProgram::open(
            profile,
            "res/shader/sky.vert.glsl",
            "res/shader/sky.frag.glsl",
        );

        let loc = SkyLoc::new(&shader);

        let colors = vec![SkyColor::new(
            [254.0 / 255.0, 254.0 / 255.0, 202.0 / 255.0],
            [98.0 / 255.0, 203.0 / 255.0, 251.0 / 255.0],
        )];

        let primitive = Primitive::quad(Handle::none());

        Sky {
            shader,
            loc,
            colors,
            primitive,
            enabled: false,
        }
    }

    pub fn draw(&self, camera: &Node) {
        unsafe {
            gl::Disable(gl::CULL_FACE);
            gl::DepthFunc(gl::LEQUAL);
        }

        self.shader.enable();
        self.loc.set_color(&self.colors[0]);

        let transform = camera.trs.get_matrix();
        camera.bind(&self.shader, &transform);
        self.primitive.bind();
        self.primitive.draw();
    }
}

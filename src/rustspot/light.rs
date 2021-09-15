use crate::*;

pub struct DirectionalLight {
    pub color: [f32; 3],
}

impl DirectionalLight {
    pub fn new() -> Self {
        Self {
            color: [1.0, 1.0, 1.0],
        }
    }

    pub fn color(r: f32, g: f32, b: f32) -> Self {
        Self { color: [r, g, b] }
    }

    pub fn bind(&self, program: &ShaderProgram, node: &Node) {
        let direction = node.trs.get_forward();

        unsafe {
            gl::Uniform3fv(program.loc.light_color, 1, &self.color as *const f32);
            gl::Uniform3fv(
                program.loc.light_direction,
                1,
                direction.as_ptr() as *const f32,
            );
        }
    }
}

pub struct PointLight {
    pub color: [f32; 3],
}

impl PointLight {
    pub fn new() -> Self {
        Self {
            color: [1.0, 1.0, 1.0],
        }
    }
}

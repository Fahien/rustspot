use crate::*;
use nalgebra as na;
use std::any::Any;
use std::{ffi::CString, fs::File, io::Read, path::Path};

pub struct Shader {
    handle: u32,
}

impl Shader {
    pub fn new(shader_type: gl::types::GLenum, src: &[u8]) -> Option<Shader> {
        unsafe {
            let version = if cfg!(feature = "gles") {
                "#version 320 es\n"
            } else {
                "#version 330 core\n"
            };

            let handle = gl::CreateShader(shader_type);

            let c_version = CString::new(version).unwrap();
            let c_src = CString::new(src).unwrap();

            let src_vec = vec![c_version.as_ptr(), c_src.as_ptr()];
            let lengths: Vec<gl::types::GLint> = vec![version.len() as i32, src.len() as i32];
            gl::ShaderSource(handle, 2, src_vec.as_ptr(), lengths.as_ptr());
            gl::CompileShader(handle);

            // Check error compiling shader
            let mut success = gl::FALSE as gl::types::GLint;
            let length = 512;
            let mut log = Vec::with_capacity(length);
            log.set_len(length as usize - 1);
            gl::GetShaderiv(handle, gl::COMPILE_STATUS, &mut success);

            if success != gl::TRUE as gl::types::GLint {
                let mut ilen = length as i32;
                gl::GetShaderInfoLog(
                    handle,
                    511,
                    &mut ilen as *mut i32,
                    log.as_mut_ptr() as *mut gl::types::GLchar,
                );
                log.set_len(ilen as usize);
                let message = CString::from(log);
                println!("Compilation failed: {}", message.to_str().unwrap());
                None
            } else {
                Some(Shader { handle })
            }
        }
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        unsafe { gl::DeleteShader(self.handle) };
    }
}

/// Uniform location handles. Handles can be negative if not found.
pub struct Loc {
    pub instance_count: i32,
    pub time: i32,
    pub extent: i32,
    pub node_id: i32,
    pub model: i32,
    pub view: i32,
    pub proj: i32,
    /// Model inverse transpose
    pub model_intr: i32,
    pub light_space: i32,
    pub tex_sampler: i32,
    pub shadow_sampler: i32,
    pub light_color: i32,
    pub light_direction: i32,
}

impl Loc {
    fn get_uniform_location(program_handle: u32, name: &str) -> i32 {
        let name = CString::new(name).expect("Failed converting Rust name to C string");
        unsafe { gl::GetUniformLocation(program_handle, name.as_ptr()) }
    }

    pub fn new(program_handle: u32) -> Loc {
        let instance_count = Loc::get_uniform_location(program_handle, "instance_count");
        let time = Loc::get_uniform_location(program_handle, "time");
        let extent = Loc::get_uniform_location(program_handle, "extent");
        let node_id = Loc::get_uniform_location(program_handle, "node_id");
        let model = Loc::get_uniform_location(program_handle, "model");
        let view = Loc::get_uniform_location(program_handle, "view");
        let proj = Loc::get_uniform_location(program_handle, "proj");
        let model_intr = Loc::get_uniform_location(program_handle, "model_intr");
        let light_space = Loc::get_uniform_location(program_handle, "light_space");
        let tex_sampler = Loc::get_uniform_location(program_handle, "tex_sampler");
        let shadow_sampler = Loc::get_uniform_location(program_handle, "shadow_sampler");
        let light_color = Loc::get_uniform_location(program_handle, "directional_light.color");
        let light_direction =
            Loc::get_uniform_location(program_handle, "directional_light.direction");

        Self {
            instance_count,
            time,
            extent,
            node_id,
            model,
            view,
            proj,
            model_intr,
            light_space,
            tex_sampler,
            shadow_sampler,
            light_color,
            light_direction,
        }
    }
}

pub struct ShaderProgram {
    handle: u32,
    pub loc: Loc,
}

impl ShaderProgram {
    pub fn new(vert: Shader, frag: Shader) -> ShaderProgram {
        let handle = unsafe { gl::CreateProgram() };

        unsafe {
            gl::AttachShader(handle, vert.handle);
            gl::AttachShader(handle, frag.handle);
            gl::LinkProgram(handle);
        }

        let loc = Loc::new(handle);

        ShaderProgram { handle, loc }
    }

    /// Returns a new shader program by loading vertex and fragment shaders files
    pub fn open<P: AsRef<Path>>(vert: P, frag: P) -> ShaderProgram {
        let mut vert_src = Vec::<u8>::new();
        let mut frag_src = Vec::<u8>::new();

        let vert_str = vert.as_ref().to_string_lossy().to_string();
        let frag_str = frag.as_ref().to_string_lossy().to_string();

        File::open(vert)
            .expect(&format!("Failed to open vertex file {}", vert_str))
            .read_to_end(&mut vert_src)
            .expect("Failed reading vertex file");
        File::open(frag)
            .expect(&format!("Failed to open fragment file {}", frag_str))
            .read_to_end(&mut frag_src)
            .expect("Failed reading fragment file");

        let vert = Shader::new(gl::VERTEX_SHADER, &vert_src)
            .expect(&format!("Failed creating shader {}", vert_str));
        let frag = Shader::new(gl::FRAGMENT_SHADER, &frag_src)
            .expect(&format!("Failed creating shader {}", frag_str));

        ShaderProgram::new(vert, frag)
    }

    pub fn get_uniform_location(&self, name: &str) -> i32 {
        Loc::get_uniform_location(self.handle, name)
    }

    pub fn enable(&self) {
        unsafe { gl::UseProgram(self.handle) };
    }
}

impl Drop for ShaderProgram {
    fn drop(&mut self) {
        unsafe { gl::DeleteProgram(self.handle) };
    }
}

pub trait CustomShader {
    fn as_any(&self) -> &dyn Any;

    fn bind(&self);
    fn bind_time(&self, delta: f32) {}
    fn bind_extent(&self, width: f32, height: f32) {}
    fn bind_sun(&self, light_color: &[f32; 3], light_node: &Node, light_space: &na::Matrix4<f32>) {}
    fn bind_shadow(&self, shadow_map: u32) {}
    fn bind_camera(&self, camera: &Camera, camera_node: &Node) {}
    fn bind_primitive(&self, primitive: &Primitive) {}
    fn bind_node(&self, node: &Node, transform: &na::Matrix4<f32>) {}

    fn draw(&self, node: &Node, primitive: &Primitive);
}

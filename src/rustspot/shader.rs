use std::{ffi::CString, fs::File, io::Read, path::Path};

pub struct Shader {
    handle: u32,
}

impl Shader {
    pub fn new(
        profile: sdl2::video::GLProfile,
        shader_type: gl::types::GLenum,
        src: &[u8],
    ) -> Option<Shader> {
        unsafe {
            let version = if profile == sdl2::video::GLProfile::Core {
                "#version 330 core\n"
            } else {
                "#version 320 es\n"
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
    pub node_id: i32,
    pub model: i32,
    pub view: i32,
    pub proj: i32,
    /// Model inverse transpose
    pub model_intr: i32,
    pub tex_sampler: i32,
    pub light_color: i32,
    pub light_direction: i32,
}

impl Loc {
    fn get_uniform_location(program_handle: u32, name: &str) -> i32 {
        let name = CString::new(name).expect("Failed converting Rust name to C string");
        unsafe { gl::GetUniformLocation(program_handle, name.as_ptr()) }
    }

    pub fn new(program_handle: u32) -> Loc {
        let node_id = Loc::get_uniform_location(program_handle, "node_id");
        let model = Loc::get_uniform_location(program_handle, "model");
        let view = Loc::get_uniform_location(program_handle, "view");
        let proj = Loc::get_uniform_location(program_handle, "proj");
        let model_intr = Loc::get_uniform_location(program_handle, "model_intr");
        let tex_sampler = Loc::get_uniform_location(program_handle, "tex_sampler");
        let light_color = Loc::get_uniform_location(program_handle, "directional_light.color");
        let light_direction =
            Loc::get_uniform_location(program_handle, "directional_light.direction");

        Self {
            node_id,
            model,
            view,
            proj,
            model_intr,
            tex_sampler,
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
    pub fn open<P: AsRef<Path>>(
        profile: sdl2::video::GLProfile,
        vert: P,
        frag: P,
    ) -> ShaderProgram {
        let mut vert_src = Vec::<u8>::new();
        let mut frag_src = Vec::<u8>::new();

        File::open(vert)
            .expect("Failed to open vertex file")
            .read_to_end(&mut vert_src)
            .expect("Failed reading vertex file");
        File::open(frag)
            .expect("Failed to open fragment file")
            .read_to_end(&mut frag_src)
            .expect("Failed reading fragment file");

        let vert =
            Shader::new(profile, gl::VERTEX_SHADER, &vert_src).expect("Failed creating shader");
        let frag =
            Shader::new(profile, gl::FRAGMENT_SHADER, &frag_src).expect("Failed creating shader");

        ShaderProgram::new(vert, frag)
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

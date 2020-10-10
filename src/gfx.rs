use std::ffi::CString;

use nalgebra as na;

pub struct Shader {
    handle: u32,
}

impl Shader {
    pub fn new(shader_type: gl::types::GLenum, src: &[u8]) -> Option<Shader> {
        unsafe {
            let handle = gl::CreateShader(shader_type);
            let c_src = CString::new(src).unwrap();
            gl::ShaderSource(handle, 1, &c_src.as_ptr(), std::ptr::null());
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

pub struct ShaderProgram {
    handle: u32,
}

impl ShaderProgram {
    pub fn new(vert: Shader, frag: Shader) -> ShaderProgram {
        unsafe {
            let handle = gl::CreateProgram();
            gl::AttachShader(handle, vert.handle);
            gl::AttachShader(handle, frag.handle);
            gl::LinkProgram(handle);
            ShaderProgram { handle }
        }
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

pub struct Vertex {
    pub position: [f32; 3],
    pub color: [f32; 3],
    pub tex_coords: [f32; 2],
}

pub struct Texture {
    handle: u32,
}

impl Texture {
    pub fn new() -> Texture {
        let mut handle: u32 = 0;
        unsafe { gl::GenTextures(1, &mut handle) };
        Texture { handle }
    }

    pub fn bind(&self) {
        unsafe { gl::BindTexture(gl::TEXTURE_2D, self.handle) };
    }

    pub fn upload<T>(&self, width: u32, height: u32, data: &[T]) {
        unsafe {
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::SRGB as i32,
                width as i32,
                height as i32,
                0,
                gl::SRGB,
                gl::UNSIGNED_BYTE,
                &data[0] as *const T as *const libc::c_void,
            );
            gl::GenerateMipmap(gl::TEXTURE_2D);
        };
    }
}

impl Drop for Texture {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteTextures(1, &self.handle);
        }
    }
}

struct Vbo {
    handle: u32,
}

impl Vbo {
    fn new() -> Vbo {
        let mut handle = 0;
        unsafe { gl::GenBuffers(1, &mut handle) };
        Vbo { handle }
    }

    fn bind(&self) {
        unsafe { gl::BindBuffer(gl::ARRAY_BUFFER, self.handle) };
    }

    fn upload<T>(&self, vertices: &Vec<T>) {
        unsafe {
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (vertices.len() * std::mem::size_of::<T>()) as isize,
                vertices.as_ptr() as *const libc::c_void,
                gl::STATIC_DRAW,
            )
        };
    }
}

impl Drop for Vbo {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteBuffers(1, &self.handle);
        }
    }
}

struct Ebo {
    handle: u32,
}

impl Ebo {
    fn new() -> Ebo {
        let mut handle = 0;
        unsafe { gl::GenBuffers(1, &mut handle) };
        Ebo { handle }
    }

    fn bind(&self) {
        unsafe { gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.handle) };
    }

    fn upload(&self, indices: &Vec<u32>) {
        unsafe {
            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                (indices.len() * std::mem::size_of::<u32>()) as isize,
                indices.as_ptr() as *const libc::c_void,
                gl::STATIC_DRAW,
            )
        };
    }
}

impl Drop for Ebo {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteBuffers(1, &self.handle);
        }
    }
}

struct Vao {
    handle: u32,
}

impl Vao {
    fn new() -> Vao {
        let mut handle = 0;
        unsafe { gl::GenVertexArrays(1, &mut handle) };
        Vao { handle }
    }

    fn bind(&self) {
        unsafe { gl::BindVertexArray(self.handle) };
    }
}

impl Drop for Vao {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteVertexArrays(1, &self.handle);
        }
    }
}

/// Geometry to be rendered with a given material
pub struct Primitive {
    vertices: Vec<Vertex>,
    indices: Vec<u32>,
}

impl Primitive {
    /// Returns a new primitive quad with side length 1 centered at the origin
    pub fn quad() -> Self {
        let vertices = vec![
            Vertex {
                position: [-0.5, -0.5, 0.0],
                color: [1.0, 1.0, 1.0],
                tex_coords: [0.0, 0.0],
            },
            Vertex {
                position: [0.5, -0.5, 0.0],
                color: [1.0, 1.0, 1.0],
                tex_coords: [1.0, 0.0],
            },
            Vertex {
                position: [0.5, 0.5, 0.0],
                color: [1.0, 1.0, 1.0],
                tex_coords: [1.0, 1.0],
            },
            Vertex {
                position: [-0.5, 0.5, 0.0],
                color: [1.0, 1.0, 1.0],
                tex_coords: [0.0, 1.0],
            },
        ];
        let indices = vec![0, 1, 2, 2, 3, 0];

        Self { vertices, indices }
    }

    pub fn draw(&self) {
        unsafe {
            gl::DrawElements(
                gl::TRIANGLES,
                self.indices.len() as _,
                gl::UNSIGNED_INT,
                0 as _,
            );
        }
    }
}

pub struct MeshRes {
    _vbo: Vbo,
    _ebo: Ebo,
    vao: Vao,
}

impl MeshRes {
    pub fn new(primitive: &Primitive) -> Self {
        let vbo = Vbo::new();
        let ebo = Ebo::new();
        let vao = Vao::new();

        vao.bind();
        vbo.bind();
        vbo.upload(&primitive.vertices);
        ebo.bind();
        ebo.upload(&primitive.indices);

        // These should follow Vao, Vbo, Ebo
        unsafe {
            // Position
            gl::VertexAttribPointer(
                0,
                3,
                gl::FLOAT,
                gl::FALSE,
                8 * std::mem::size_of::<f32>() as i32,
                0 as _,
            );
            gl::EnableVertexAttribArray(0);

            // Color
            gl::VertexAttribPointer(
                1,
                3,
                gl::FLOAT,
                gl::FALSE,
                8 * std::mem::size_of::<f32>() as i32,
                (3 * std::mem::size_of::<f32>()) as _,
            );
            gl::EnableVertexAttribArray(1);

            // Texture coordinates
            gl::VertexAttribPointer(
                2,
                2,
                gl::FLOAT,
                gl::FALSE,
                8 * std::mem::size_of::<f32>() as i32,
                (6 * std::mem::size_of::<f32>()) as _,
            );
            gl::EnableVertexAttribArray(2)
        }

        Self {
            _vbo: vbo,
            _ebo: ebo,
            vao,
        }
    }

    pub fn bind(&self) {
        self.vao.bind();
    }
}

/// A node can refer to a camera to apply a transform to place it in the scene
pub struct Camera {
    proj: na::Matrix4<f32>,
}

impl Camera {
    pub fn orthogonal() -> Camera {
        let (w, h) = (4.8, 3.2);
        let proj = na::Orthographic3::new(-w / 2.0, w / 2.0, -h / 2.0, h / 2.0, 0.1, 100.0);
        Camera {
            proj: proj.to_homogeneous(),
        }
    }

    pub fn perspective() -> Camera {
        let (w, h) = (480.0, 320.0);
        let proj = na::Perspective3::new(w / h, 3.14 / 4.0, 0.1, 100.0);
        Camera {
            proj: proj.to_homogeneous(),
        }
    }

    pub fn bind(&self, view: &Node) {
        unsafe {
            gl::UniformMatrix4fv(
                1, // view location
                1,
                gl::FALSE,
                view.model.inverse().to_homogeneous().as_ptr(),
            );

            gl::UniformMatrix4fv(
                2, // proj location
                1,
                gl::FALSE,
                self.proj.as_ptr(),
            );
        }
    }
}

pub struct Node {
    pub name: String,
    pub model: na::Isometry3<f32>,
}

impl Node {
    pub fn new() -> Self {
        Node {
            name: String::new(),
            model: na::Isometry3::identity(),
        }
    }

    pub fn bind(&self) {
        unsafe {
            gl::UniformMatrix4fv(
                0, // model location
                1,
                gl::FALSE,
                self.model.to_homogeneous().as_ptr(),
            );
        }
    }
}

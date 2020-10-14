use std::collections::HashMap;
use std::ffi::CString;

use nalgebra as na;

use crate::util::*;

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

    pub fn upload<T>(&mut self, width: u32, height: u32, data: &[T]) {
        self.bind();

        unsafe {
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGBA as i32,
                width as i32,
                height as i32,
                0,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                &data[0] as *const T as _,
            );

            gl::GenerateMipmap(gl::TEXTURE_2D);

            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
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

    fn upload<T>(&mut self, vertices: &Vec<T>) {
        self.bind();
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

    fn upload(&mut self, indices: &Vec<u32>) {
        self.bind();
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
    _vertices: Vec<Vertex>,
    indices: Vec<u32>,

    // Res could be computed on the fly, but we would need to hash both vertices and indices,
    // therefore we store it here and it is responsibility of the scene builder to avoid an
    // explosion of primitive resources at run-time.
    res: MeshRes,
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

        let res = MeshRes::from(&vertices, &indices);

        Self {
            _vertices: vertices,
            indices,
            res,
        }
    }

    pub fn bind(&self) {
        self.res.bind();
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

/// A mesh is an array of primitives to be rendered. A node can contain
/// one mesh, and a node's transform places the mesh in the scene
pub struct Mesh {
    pub name: String,
    primitives: Vec<Primitive>,
}

impl Mesh {
    pub fn new(primitives: Vec<Primitive>) -> Self {
        Self {
            name: String::new(),
            primitives,
        }
    }

    pub fn bind(&self) {
        for primitive in self.primitives.iter() {
            primitive.bind();
        }
    }

    pub fn draw(&self) {
        for primitive in self.primitives.iter() {
            primitive.draw();
        }
    }
}

pub struct MeshRes {
    vbo: Vbo,
    ebo: Ebo,
    vao: Vao,
}

impl MeshRes {
    pub fn new() -> Self {
        let vbo = Vbo::new();
        let ebo = Ebo::new();
        let vao = Vao::new();

        Self { vbo, ebo, vao }
    }

    pub fn from(vertices: &Vec<Vertex>, indices: &Vec<u32>) -> Self {
        let mut res = MeshRes::new();

        res.vao.bind();
        res.vbo.upload(&vertices);
        res.ebo.upload(&indices);

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

        res
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
    pub mesh: Handle<Mesh>,
    pub children: Vec<Handle<Node>>,
}

impl Node {
    pub fn new() -> Self {
        Node {
            name: String::new(),
            model: na::Isometry3::identity(),
            mesh: Handle::none(),
            children: vec![],
        }
    }

    fn bind(&self, transform: &na::Matrix4<f32>) {
        unsafe {
            gl::UniformMatrix4fv(
                0, // model location
                1,
                gl::FALSE,
                transform.as_ptr(),
            );
        }
    }

    /// This is going to draw a node
    pub fn draw(&self, meshes: &Pack<Mesh>, transform: &na::Matrix4<f32>) {
        // If node has a mesh, bind this transform and draw elements
        if self.mesh.valid() {
            self.bind(transform);
            // This is not going to bind any mesh resource
            // As we expect them to be already bound
            meshes[self.mesh.id].draw();
        }
    }
}

impl std::fmt::Debug for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Node {}", self.name)
    }
}

fn gl_err_to_string(err: gl::types::GLenum) -> &'static str {
    match err {
        gl::INVALID_ENUM => "Invalid enum",
        gl::INVALID_VALUE => "Invalid value",
        gl::INVALID_OPERATION => "Invalid operation",
        gl::STACK_OVERFLOW => "Stack overflow",
        gl::STACK_UNDERFLOW => "Stack underflow",
        gl::OUT_OF_MEMORY => "Out of memory",
        gl::INVALID_FRAMEBUFFER_OPERATION => "Invalid framebuffer operation",
        gl::CONTEXT_LOST => "Context lost",
        _ => "Unknown",
    }
}

/// Useful function to check for graphics errors
pub fn gl_check() {
    let err = unsafe { gl::GetError() };
    if err != gl::NO_ERROR {
        panic!("GlError {}", gl_err_to_string(err));
    }
}

pub struct Renderer {
    /// List of mesh handles to draw with nodes referring to them.
    /// Together with nodes, we store their transform matrix computed during the scene graph traversal.
    meshes: HashMap<usize, HashMap<usize, na::Matrix4<f32>>>,
}

impl Renderer {
    pub fn new() -> Renderer {
        let (mut major, mut minor) = (0, 0);
        unsafe {
            gl::load_with(|symbol| {
                go2::eglGetProcAddress(CString::new(symbol).unwrap().as_ptr()) as *const _
            });

            gl::GetIntegerv(gl::MAJOR_VERSION, &mut major);
            gl::GetIntegerv(gl::MINOR_VERSION, &mut minor);
        }
        println!("OpenGL v{}.{}", major, minor);

        Renderer {
            meshes: HashMap::new(),
        }
    }

    /// Draw does not render immediately, instead it creates a list of mesh resources.
    /// At the same time it computes transform matrices for each node to be bound later on.
    pub fn draw(
        &mut self,
        nodes: &Pack<Node>,
        node: &Handle<Node>,
        transform: &na::Isometry3<f32>,
    ) {
        // Precompute transform matrix
        let temp_transform = transform * nodes[node.id].model;

        // Here we add this to a list of nodes that should be rendered
        let mesh = &nodes[node.id].mesh;
        if mesh.valid() {
            if let Some(mesh_nodes) = self.meshes.get_mut(&mesh.id) {
                if let None = mesh_nodes.get(&node.id) {
                    // Add this nodes to the list of nodes associated to this mesh
                    mesh_nodes.insert(node.id, temp_transform.to_homogeneous());
                }
            } else {
                // Create a new entry in the mesh resources
                let mut mesh_nodes = HashMap::new();
                mesh_nodes.insert(node.id, temp_transform.to_homogeneous());
                self.meshes.insert(mesh.id, mesh_nodes);
            }
        }

        // And all its children recursively
        for child in nodes[node.id].children.iter() {
            self.draw(nodes, child, &temp_transform);
        }
    }

    /// This should be called after drawing everything to trigger the actual GL rendering.
    pub fn present(&mut self, meshes: &Pack<Mesh>, nodes: &Pack<Node>) {
        // Rendering should follow this approach
        // foreach prog in programs:
        //   bind(prog)
        //   foreach mat in p.materials:
        //     bind(mat)
        //     foreach mesh in mat.meshes:
        //       bind(mesh)
        //       foreach node in mesh.nodes:
        //         draw(nodes) -> draw(mesh) -> draw(primitives)
        for (mesh_id, node_res) in self.meshes.iter() {
            meshes[*mesh_id].bind();

            for (node_id, transform) in node_res.iter() {
                nodes[*node_id].draw(meshes, &transform);
            }
        }

        self.meshes.clear();
    }
}

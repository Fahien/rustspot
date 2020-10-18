use std::collections::HashMap;
use std::ffi::CString;
use std::fs::File;
use std::io::Read;
use std::path::Path;

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

    /// Returns a new shader program by loading vertex and fragment shaders files
    pub fn open<P: AsRef<Path>>(vert: P, frag: P) -> ShaderProgram {
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

        let vert = Shader::new(gl::VERTEX_SHADER, &vert_src).expect("Failed creating shader");
        let frag = Shader::new(gl::FRAGMENT_SHADER, &frag_src).expect("Failed creating shader");

        ShaderProgram::new(vert, frag)
    }

    pub fn enable(&self) {
        unsafe { gl::UseProgram(self.handle) };
    }

    pub fn get_uniform_location(&self, name: &str) -> Result<i32, &str> {
        let name = CString::new(name).expect("Failed converting Rust name to C string");
        let location = unsafe { gl::GetUniformLocation(self.handle, name.as_ptr()) };
        if location == -1 {
            return Err("Failed to get uniform location");
        }
        Ok(location)
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

    /// Loads a PNG image from a path and returns a new texture
    pub fn open<P: AsRef<Path>>(path: P) -> Texture {
        let str_path = path.as_ref().to_str().unwrap();
        let message = format!("Failed to open: {}", str_path);
        let decoder = png::Decoder::new(File::open(path).expect(&message));
        let (info, mut reader) = decoder.read_info().expect("Failed reading png info");
        let mut data: Vec<u8> = vec![0; info.buffer_size()];
        reader
            .next_frame(data.as_mut_slice())
            .expect("Failed to read png frame");

        let mut texture = Texture::new();
        texture.upload(info.width, info.height, &data);
        texture
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

pub struct Material {
    shader: Handle<ShaderProgram>,
    texture: Handle<Texture>,
}

impl Material {
    pub fn new(texture: Handle<Texture>) -> Self {
        Self {
            shader: Handle::new(0),
            texture,
        }
    }

    pub fn bind(&self, textures: &Pack<Texture>) {
        if let Some(texture) = textures.get(&self.texture) {
            texture.bind();
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
    material: Handle<Material>,

    // Res could be computed on the fly, but we would need to hash both vertices and indices,
    // therefore we store it here and it is responsibility of the scene builder to avoid an
    // explosion of primitive resources at run-time.
    res: MeshRes,
}

impl Primitive {
    /// Returns a new primitive quad with side length 1 centered at the origin
    pub fn quad(material: Handle<Material>) -> Self {
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
            material,
            res,
        }
    }

    /// This function is going to bind only this primitive's VAO. We do not bind the
    /// primitives' material here because we expect the renderer has already bound it.
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
    primitives: Vec<Handle<Primitive>>,
}

impl Mesh {
    pub fn new(primitives: Vec<Handle<Primitive>>) -> Self {
        Self {
            name: String::new(),
            primitives,
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

    pub fn bind(&self, program: &ShaderProgram, view: &Node) {
        program.enable();
        let view_loc = program
            .get_uniform_location("view")
            .expect("Failed to get view uniform location");
        let proj_loc = program
            .get_uniform_location("proj")
            .expect("Failed to get proj uniform location");

        unsafe {
            gl::UniformMatrix4fv(
                view_loc,
                1,
                gl::FALSE,
                view.model.inverse().to_homogeneous().as_ptr(),
            );

            gl::UniformMatrix4fv(proj_loc, 1, gl::FALSE, self.proj.as_ptr());
        }
    }
}

pub struct Node {
    pub name: String,
    pub model: na::Isometry3<f32>,
    pub mesh: Handle<Mesh>,
    pub camera: Handle<Camera>,
    pub children: Vec<Handle<Node>>,
}

impl Node {
    pub fn new() -> Self {
        Node {
            name: String::new(),
            model: na::Isometry3::identity(),
            mesh: Handle::none(),
            camera: Handle::none(),
            children: vec![],
        }
    }

    fn bind(&self, program: &ShaderProgram, transform: &na::Matrix4<f32>) {
        let model_loc = program
            .get_uniform_location("model")
            .expect("Failed to get model uniform location");
        unsafe {
            gl::UniformMatrix4fv(model_loc, 1, gl::FALSE, transform.as_ptr());
        }
    }
}

impl std::fmt::Debug for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Node {}", self.name)
    }
}

pub struct Model {
    pub programs: Pack<ShaderProgram>,
    pub textures: Pack<Texture>,
    pub materials: Pack<Material>,
    pub primitives: Pack<Primitive>,
    pub meshes: Pack<Mesh>,
    pub nodes: Pack<Node>,
    pub cameras: Pack<Camera>,
}

impl Model {
    pub fn new() -> Self {
        Self {
            programs: Pack::new(),
            textures: Pack::new(),
            materials: Pack::new(),
            primitives: Pack::new(),
            meshes: Pack::new(),
            nodes: Pack::new(),
            cameras: Pack::new(),
        }
    }
}

pub struct GuiRes {
    _font_texture: Texture,
    program: ShaderProgram,
    mesh_res: MeshRes,
}

impl GuiRes {
    pub fn new(fonts: &mut imgui::FontAtlasRefMut) -> Self {
        // Font
        let mut font_texture = Texture::new();
        let texture = fonts.build_rgba32_texture();
        font_texture.upload(texture.width, texture.height, texture.data);
        fonts.tex_id = (font_texture.handle as usize).into();

        // Shaders
        let vert_source = r#"#version 320 es

        layout (location = 0) in vec2 in_position;
        layout (location = 1) in vec2 in_tex_coords;
        layout (location = 2) in vec4 in_color;

        uniform mat4 proj;

        out vec2 tex_coords;
        out vec4 color;

        void main()
        {
            tex_coords = in_tex_coords;
            color = in_color;
            gl_Position = proj * vec4(in_position, 0.0, 1.0);
        }
        "#;

        let frag_source = r#"#version 320 es
        precision mediump float;

        in vec2 tex_coords;
        in vec4 color;

        uniform sampler2D tex_sampler;

        out vec4 out_color;

        void main()
        {
            out_color = color * texture(tex_sampler, tex_coords);
        }
        "#;

        let vert = Shader::new(gl::VERTEX_SHADER, vert_source.as_bytes())
            .expect("Failed to create imgui vertex shader");
        let frag = Shader::new(gl::FRAGMENT_SHADER, frag_source.as_bytes())
            .expect("Failed to create imgui fragment shader");

        let program = ShaderProgram::new(vert, frag);

        // Mesh resources
        let mesh_res = MeshRes::new();

        mesh_res.vao.bind();
        mesh_res.vbo.bind();
        mesh_res.ebo.bind();

        let stride = std::mem::size_of::<imgui::DrawVert>() as i32;

        unsafe {
            // Position
            gl::VertexAttribPointer(0, 2, gl::FLOAT, gl::FALSE, stride, 0 as _);
            gl::EnableVertexAttribArray(0);

            // Texture coordinates
            gl::VertexAttribPointer(
                1,
                2,
                gl::FLOAT,
                gl::FALSE,
                stride,
                (2 * std::mem::size_of::<f32>()) as _,
            );
            gl::EnableVertexAttribArray(1);

            // Color
            gl::VertexAttribPointer(
                2,
                4,
                gl::UNSIGNED_BYTE,
                gl::TRUE,
                stride,
                (4 * std::mem::size_of::<f32>()) as _,
            );
            gl::EnableVertexAttribArray(2);
        }

        GuiRes {
            _font_texture: font_texture,
            program,
            mesh_res,
        }
    }

    pub fn draw(&mut self, ui: imgui::Ui) {
        let [width, height] = ui.io().display_size;
        let [scale_w, scale_h] = ui.io().display_framebuffer_scale;
        let fb_width = width * scale_w;
        let fb_height = height * scale_h;

        unsafe {
            gl::Enable(gl::BLEND);
            gl::BlendEquation(gl::FUNC_ADD);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
            gl::Disable(gl::CULL_FACE);
            gl::Disable(gl::DEPTH_TEST);
            gl::Enable(gl::SCISSOR_TEST);
            // There is no glPolygonMode in GLES3.2
            // gl::PolygonMode(gl::FRONT_AND_BACK, gl::FILL);
            gl::Viewport(0, 0, fb_width as _, fb_height as _);
        }

        let data = ui.render();

        let matrix = [
            [2.0 / width as f32, 0.0, 0.0, 0.0],
            [0.0, 2.0 / -(height as f32), 0.0, 0.0],
            [0.0, 0.0, -1.0, 0.0],
            [-1.0, 1.0, 0.0, 1.0],
        ];

        self.program.enable();

        let proj_loc = self
            .program
            .get_uniform_location("proj")
            .expect("Failed to get proj location");
        let tex_sampler_loc = self
            .program
            .get_uniform_location("tex_sampler")
            .expect("Failed to get tex_sampler location");
        unsafe {
            gl::UniformMatrix4fv(proj_loc, 1, gl::FALSE, matrix.as_ptr() as _);
            gl::Uniform1i(tex_sampler_loc, 0);
        }

        for draw_list in data.draw_lists() {
            let vtx_buffer = draw_list.vtx_buffer();
            let idx_buffer = draw_list.idx_buffer();

            self.mesh_res.vao.bind();

            self.mesh_res.vbo.bind();
            unsafe {
                gl::BufferData(
                    gl::ARRAY_BUFFER,
                    (vtx_buffer.len() * std::mem::size_of::<imgui::DrawVert>()) as _,
                    vtx_buffer.as_ptr() as _,
                    gl::STREAM_DRAW,
                );
            }

            self.mesh_res.ebo.bind();
            unsafe {
                gl::BufferData(
                    gl::ELEMENT_ARRAY_BUFFER,
                    (idx_buffer.len() * std::mem::size_of::<imgui::DrawIdx>()) as _,
                    idx_buffer.as_ptr() as _,
                    gl::STREAM_DRAW,
                );
            }

            for cmd in draw_list.commands() {
                match cmd {
                    imgui::DrawCmd::Elements {
                        count,
                        cmd_params:
                            imgui::DrawCmdParams {
                                clip_rect: [x, y, z, w],
                                texture_id,
                                idx_offset,
                                ..
                            },
                    } => {
                        unsafe {
                            gl::BindTexture(gl::TEXTURE_2D, texture_id.id() as _);
                            gl::Scissor(
                                (x * scale_w) as gl::types::GLint,
                                (fb_height - w * scale_h) as gl::types::GLint,
                                ((z - x) * scale_w) as gl::types::GLint,
                                ((w - y) * scale_h) as gl::types::GLint,
                            );
                        }

                        let idx_size = if std::mem::size_of::<imgui::DrawIdx>() == 2 {
                            gl::UNSIGNED_SHORT
                        } else {
                            gl::UNSIGNED_INT
                        };

                        unsafe {
                            gl::DrawElements(
                                gl::TRIANGLES,
                                count as _,
                                idx_size,
                                (idx_offset * std::mem::size_of::<imgui::DrawIdx>()) as _,
                            );
                        }
                    }
                    imgui::DrawCmd::ResetRenderState => {
                        unimplemented!("DrawCmd::ResetRenderState not implemented");
                    }
                    imgui::DrawCmd::RawCallback { .. } => {
                        unimplemented!("User callbacks not implemented");
                    }
                }
            }
        }
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
    /// List of shader handles to bind with materials referring to them.
    shaders: HashMap<usize, Vec<usize>>,

    /// List of camera handles to use while drawing the scene paired with the node to use
    cameras: Vec<(Handle<Camera>, Handle<Node>)>,

    /// List of material handles to bind with primitives referring to them.
    materials: HashMap<usize, Vec<usize>>,

    /// List of primitive handles to draw with nodes referring to them.
    /// Together with nodes, we store their transform matrix computed during the scene graph traversal.
    primitives: HashMap<usize, HashMap<usize, na::Matrix4<f32>>>,
}

impl Renderer {
    pub fn new() -> Renderer {
        Renderer {
            shaders: HashMap::new(),
            cameras: Vec::new(),
            materials: HashMap::new(),
            primitives: HashMap::new(),
        }
    }

    /// Draw does not render immediately, instead it creates a list of mesh resources.
    /// At the same time it computes transform matrices for each node to be bound later on.
    pub fn draw(
        &mut self,
        model: &Model,
        node_handle: &Handle<Node>,
        transform: &na::Isometry3<f32>,
    ) {
        // Precompute transform matrix
        let temp_transform = transform * model.nodes[node_handle.id].model;

        // The current node
        let node = model.nodes.get(&node_handle).unwrap();

        // Here we add this to a list of nodes that should be rendered
        let mesh = &node.mesh;
        if let Some(mesh) = model.meshes.get(&mesh) {
            for primitive_handle in mesh.primitives.iter() {
                let primitive = model.primitives.get(&primitive_handle).unwrap();
                let material = model.materials.get(&primitive.material).unwrap();

                // Store this association shader program, material
                let key = material.shader.id;
                if let Some(shader_materials) = self.shaders.get_mut(&key) {
                    // Add this material id to the value list of not already there
                    if !shader_materials.contains(&primitive.material.id) {
                        shader_materials.push(primitive.material.id);
                    }
                } else {
                    // Create a new entry (shader, material)
                    self.shaders.insert(key, vec![primitive.material.id]);
                }

                // Store this association material, primitive
                let key = primitive.material.id;
                // Check if an entry already exists
                if let Some(material_primitives) = self.materials.get_mut(&key) {
                    // Add this primitive id to the value list if not already there
                    if !material_primitives.contains(&primitive_handle.id) {
                        material_primitives.push(primitive_handle.id);
                    }
                } else {
                    // Create a new entry (material, primitives)
                    self.materials.insert(key, vec![primitive_handle.id]);
                }

                // Get those nodes referring to this primitive
                if let Some(primitive_nodes) = self.primitives.get_mut(&primitive_handle.id) {
                    // Add this nodes to the list of nodes associated to this primitive if not already there
                    if !primitive_nodes.contains_key(&node_handle.id) {
                        primitive_nodes.insert(node_handle.id, temp_transform.to_homogeneous());
                    }
                } else {
                    // Create a new entry in the primitive resources
                    let mut primitive_nodes = HashMap::new();
                    primitive_nodes.insert(node_handle.id, temp_transform.to_homogeneous());
                    self.primitives.insert(primitive_handle.id, primitive_nodes);
                }
            }
        }

        // Here we check if the current node has a camera, just add it
        if model.cameras.get(&node.camera).is_some() {
            self.cameras.push((node.camera, *node_handle));
        }

        // And all its children recursively
        for child in node.children.iter() {
            self.draw(model, child, &temp_transform);
        }
    }

    /// This should be called after drawing everything to trigger the actual GL rendering.
    pub fn present(&mut self, model: &Model) {
        // Rendering should follow this approach
        // foreach prog in programs:
        //   bind(prog)
        //   foreach mat in p.materials:
        //     bind(mat)
        //     foreach prim in mat.primitives:
        //       bind(prim)
        //       foreach node in prim.nodes:
        //         bind(node) -> draw(prim)

        // Need to bind programs one at a time
        for (shader_id, material_ids) in self.shaders.iter() {
            let shader = &model.programs[*shader_id];
            shader.enable();

            // Draw the scene from all the points of view
            for (camera_handle, camera_node_handle) in self.cameras.iter() {
                let camera = model.cameras.get(&camera_handle).unwrap();
                let camera_node = model.nodes.get(&camera_node_handle).unwrap();
                camera.bind(shader, camera_node);

                // Need to bind materials for a group of primitives that use the same one
                for material_id in material_ids.iter() {
                    let primitive_ids = &self.materials[material_id];

                    let material = &model.materials[*material_id];
                    material.bind(&model.textures);

                    for primitive_id in primitive_ids.iter() {
                        let primitive = &model.primitives[*primitive_id];
                        assert!(primitive.material.valid());

                        // Bind the primitive, bind the nodes using that primitive, draw the primitive.
                        primitive.bind();
                        let node_res = &self.primitives[primitive_id];
                        for (node_id, transform) in node_res.iter() {
                            model.nodes[*node_id].bind(shader, &transform);
                            primitive.draw();
                        }
                    }
                }
            }
        }

        self.shaders.clear();
        self.cameras.clear();
        self.materials.clear();
        self.primitives.clear();
    }
}

pub struct Video {
    system: sdl2::VideoSubsystem,
    window: sdl2::video::Window,
    gl: sdl2::video::GLContext,
}

impl Video {
    fn new(sdl: &sdl2::Sdl) -> Self {
        let system = sdl.video().expect("Failed initializing video");

        let attr = system.gl_attr();
        attr.set_context_profile(sdl2::video::GLProfile::GLES);
        attr.set_context_version(3, 2);

        let window = system
            // TODO: pass these as parameters
            .window("Test", 480, 320)
            .opengl()
            .position_centered()
            .build()
            .expect("Failed building window");

        let gl = window
            .gl_create_context()
            .expect("Failed creating GL context");

        gl::load_with(|symbol| system.gl_get_proc_address(symbol) as *const _);

        Self { system, window, gl }
    }
}

pub struct Gfx {
    pub video: Video,
    pub renderer: Renderer,
}

impl Gfx {
    pub fn new(sdl: &sdl2::Sdl) -> Self {
        let video = Video::new(sdl);
        let renderer = Renderer::new();
        Self { video, renderer }
    }

    pub fn get_gl_version(&self) -> (i32, i32) {
        let (mut major, mut minor) = (0, 0);
        unsafe {
            gl::GetIntegerv(gl::MAJOR_VERSION, &mut major);
            gl::GetIntegerv(gl::MINOR_VERSION, &mut minor);
        }
        (major, minor)
    }

    pub fn swap_buffers(&self) {
        self.video.window.gl_swap_window();
    }
}

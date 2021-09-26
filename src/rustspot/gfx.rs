use std::collections::HashMap;
use std::fs::File;
use std::path::Path;

use nalgebra as na;

use crate::*;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Vertex {
    pub position: [f32; 3],
    pub color: [f32; 3],
    pub tex_coords: [f32; 2],
    pub normal: [f32; 3],
}

impl Vertex {
    pub fn new() -> Self {
        Self {
            position: [0.0, 0.0, 0.0],
            color: [1.0, 1.0, 1.0],
            tex_coords: [0.0, 0.0],
            normal: [0.0, 0.0, 1.0],
        }
    }
}

pub struct Texture {
    pub handle: u32,
    pub extent: Extent2D,
}

impl Texture {
    pub fn new() -> Texture {
        let mut handle: u32 = 0;
        unsafe { gl::GenTextures(1, &mut handle) };
        Texture {
            handle,
            extent: Extent2D::new(0, 0),
        }
    }

    pub fn attachment(
        extent: Extent2D,
        format: gl::types::GLenum,
        type_: gl::types::GLenum,
    ) -> Self {
        let mut texture = Self::new();
        texture.extent = extent;
        texture.bind();

        unsafe {
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                format as _,
                extent.width as _,
                extent.height as _,
                0,
                format,
                type_,
                std::ptr::null(),
            );

            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
        }

        texture
    }

    pub fn color(extent: Extent2D) -> Self {
        Self::attachment(extent, gl::RGB, gl::UNSIGNED_BYTE)
    }

    pub fn depth(extent: Extent2D) -> Self {
        Self::attachment(extent, gl::DEPTH_COMPONENT, gl::UNSIGNED_SHORT)
    }

    /// Creates a one pixel texture with the RGBA color passed as argument
    pub fn pixel(data: &[u8; 4]) -> Self {
        let mut texture = Self::new();
        texture.upload(1, 1, data);
        texture
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
        self.extent.width = width;
        self.extent.height = height;

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
    pub shader: Handle<ShaderProgram>,
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

pub struct Vbo {
    handle: u32,
}

impl Vbo {
    fn new() -> Vbo {
        let mut handle = 0;
        unsafe { gl::GenBuffers(1, &mut handle) };
        Vbo { handle }
    }

    pub fn bind(&self) {
        unsafe { gl::BindBuffer(gl::ARRAY_BUFFER, self.handle) };
    }

    fn upload<T>(&mut self, vertices: &[T]) {
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

pub struct Ebo {
    handle: u32,
}

impl Ebo {
    fn new() -> Ebo {
        let mut handle = 0;
        unsafe { gl::GenBuffers(1, &mut handle) };
        Ebo { handle }
    }

    pub fn bind(&self) {
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

pub struct Vao {
    handle: u32,
}

impl Vao {
    fn new() -> Vao {
        let mut handle = 0;
        unsafe { gl::GenVertexArrays(1, &mut handle) };
        Vao { handle }
    }

    pub fn bind(&self) {
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
                normal: [0.0, 0.0, 1.0],
            },
            Vertex {
                position: [0.5, -0.5, 0.0],
                color: [1.0, 1.0, 1.0],
                tex_coords: [1.0, 0.0],
                normal: [0.0, 0.0, 1.0],
            },
            Vertex {
                position: [0.5, 0.5, 0.0],
                color: [1.0, 1.0, 1.0],
                tex_coords: [1.0, 1.0],
                normal: [0.0, 0.0, 1.0],
            },
            Vertex {
                position: [-0.5, 0.5, 0.0],
                color: [1.0, 1.0, 1.0],
                tex_coords: [0.0, 1.0],
                normal: [0.0, 0.0, 1.0],
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

    pub fn cube(material: Handle<Material>) -> Self {
        let mut vertices = vec![Vertex::new(); 24];

        let (tex_width, tex_height) = (4.0, 4.0);

        // Front
        vertices[0].position = [-0.5, -0.5, 0.5];
        vertices[0].tex_coords = [1.0 / tex_width, 1.0 / tex_height];
        vertices[0].normal = [0.0, 0.0, 1.0];
        vertices[1].position = [0.5, -0.5, 0.5];
        vertices[1].tex_coords = [2.0 / tex_width, 1.0 / tex_height];
        vertices[1].normal = [0.0, 0.0, 1.0];
        vertices[2].position = [0.5, 0.5, 0.5];
        vertices[2].tex_coords = [2.0 / tex_width, 2.0 / tex_height];
        vertices[2].normal = [0.0, 0.0, 1.0];
        vertices[3].position = [-0.5, 0.5, 0.5];
        vertices[3].tex_coords = [1.0 / tex_width, 2.0 / tex_height];
        vertices[3].normal = [0.0, 0.0, 1.0];

        // Right
        vertices[4].position = [0.5, -0.5, 0.5];
        vertices[4].normal = [1.0, 0.0, 0.0];
        vertices[4].tex_coords = [2.0 / tex_width, 1.0 / tex_height];
        vertices[5].position = [0.5, -0.5, -0.5];
        vertices[5].normal = [1.0, 0.0, 0.0];
        vertices[5].tex_coords = [3.0 / tex_width, 1.0 / tex_height];
        vertices[6].position = [0.5, 0.5, -0.5];
        vertices[6].normal = [1.0, 0.0, 0.0];
        vertices[6].tex_coords = [3.0 / tex_width, 2.0 / tex_height];
        vertices[7].position = [0.5, 0.5, 0.5];
        vertices[7].normal = [1.0, 0.0, 0.0];
        vertices[7].tex_coords = [2.0 / tex_width, 2.0 / tex_height];

        // Back
        vertices[8].position = [0.5, -0.5, -0.5];
        vertices[8].normal = [0.0, 0.0, -1.0];
        vertices[8].tex_coords = [3.0 / tex_width, 1.0 / tex_height];
        vertices[9].position = [-0.5, -0.5, -0.5];
        vertices[9].normal = [0.0, 0.0, -1.0];
        vertices[9].tex_coords = [4.0 / tex_width, 1.0 / tex_height];
        vertices[10].position = [-0.5, 0.5, -0.5];
        vertices[10].normal = [0.0, 0.0, -1.0];
        vertices[10].tex_coords = [4.0 / tex_width, 2.0 / tex_height];
        vertices[11].position = [0.5, 0.5, -0.5];
        vertices[11].normal = [0.0, 0.0, -1.0];
        vertices[11].tex_coords = [3.0 / tex_width, 2.0 / tex_height];

        // Left
        vertices[12].position = [-0.5, -0.5, -0.5];
        vertices[12].normal = [-1.0, 0.0, 0.0];
        vertices[12].tex_coords = [0.0, 1.0 / tex_height];
        vertices[13].position = [-0.5, -0.5, 0.5];
        vertices[13].normal = [-1.0, 0.0, 0.0];
        vertices[13].tex_coords = [1.0 / tex_width, 1.0 / tex_height];
        vertices[14].position = [-0.5, 0.5, 0.5];
        vertices[14].normal = [-1.0, 0.0, 0.0];
        vertices[14].tex_coords = [1.0 / tex_width, 2.0 / tex_height];
        vertices[15].position = [-0.5, 0.5, -0.5];
        vertices[15].normal = [-1.0, 0.0, 0.0];
        vertices[15].tex_coords = [0.0, 2.0 / tex_height];

        // Top
        vertices[16].position = [-0.5, 0.5, 0.5];
        vertices[16].normal = [0.0, 1.0, 0.0];
        vertices[16].tex_coords = [1.0 / tex_width, 2.0 / tex_height];
        vertices[17].position = [0.5, 0.5, 0.5];
        vertices[17].normal = [0.0, 1.0, 0.0];
        vertices[17].tex_coords = [2.0 / tex_width, 2.0 / tex_height];
        vertices[18].position = [0.5, 0.5, -0.5];
        vertices[18].normal = [0.0, 1.0, 0.0];
        vertices[18].tex_coords = [2.0 / tex_width, 3.0 / tex_height];
        vertices[19].position = [-0.5, 0.5, -0.5];
        vertices[19].normal = [0.0, 1.0, 0.0];
        vertices[19].tex_coords = [1.0 / tex_width, 3.0 / tex_height];

        // Bottom
        vertices[20].position = [-0.5, -0.5, -0.5];
        vertices[20].normal = [0.0, -1.0, 0.0];
        vertices[20].tex_coords = [1.0 / tex_width, 0.0];
        vertices[21].position = [0.5, -0.5, -0.5];
        vertices[21].normal = [0.0, -1.0, 0.0];
        vertices[21].tex_coords = [2.0 / tex_width, 0.0];
        vertices[22].position = [0.5, -0.5, 0.5];
        vertices[22].normal = [0.0, -1.0, 0.0];
        vertices[22].tex_coords = [2.0 / tex_width, 1.0 / tex_height];
        vertices[23].position = [-0.5, -0.5, 0.5];
        vertices[23].normal = [0.0, -1.0, 0.0];
        vertices[23].tex_coords = [1.0 / tex_width, 1.0 / tex_height];

        let indices = vec![
            0, 1, 2, 0, 2, 3, // front face
            4, 5, 6, 4, 6, 7, // right
            8, 9, 10, 8, 10, 11, // back
            12, 13, 14, 12, 14, 15, // left
            16, 17, 18, 16, 18, 19, // top
            20, 21, 22, 20, 22, 23, // bottom
        ];

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
    pub vbo: Vbo,
    pub ebo: Ebo,
    pub vao: Vao,
}

impl MeshRes {
    pub fn new() -> Self {
        let vbo = Vbo::new();
        let ebo = Ebo::new();
        let vao = Vao::new();

        Self { vbo, ebo, vao }
    }

    pub fn from(vertices: &[Vertex], indices: &Vec<u32>) -> Self {
        let mut res = MeshRes::new();

        res.vao.bind();
        res.vbo.upload(&vertices);
        res.ebo.upload(&indices);

        let stride = std::mem::size_of::<Vertex>() as i32;

        // These should follow Vao, Vbo, Ebo
        unsafe {
            // Position
            gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, stride, 0 as _);
            gl::EnableVertexAttribArray(0);

            // Color
            gl::VertexAttribPointer(
                1,
                3,
                gl::FLOAT,
                gl::FALSE,
                stride,
                (3 * std::mem::size_of::<f32>()) as _,
            );
            gl::EnableVertexAttribArray(1);

            // Texture coordinates
            gl::VertexAttribPointer(
                2,
                2,
                gl::FLOAT,
                gl::FALSE,
                stride,
                (6 * std::mem::size_of::<f32>()) as _,
            );
            gl::EnableVertexAttribArray(2);

            // Normal
            gl::VertexAttribPointer(
                3,
                3,
                gl::FLOAT,
                gl::TRUE,
                stride,
                (8 * std::mem::size_of::<f32>()) as _,
            );
            gl::EnableVertexAttribArray(3);
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

        let view = view.trs.isometry.inverse().to_homogeneous();
        unsafe {
            gl::UniformMatrix4fv(program.loc.view, 1, gl::FALSE, view.as_ptr());
            gl::UniformMatrix4fv(program.loc.proj, 1, gl::FALSE, self.proj.as_ptr());
        }
    }
}

#[derive(Clone)]
pub struct Trs {
    isometry: na::Isometry3<f32>,
    scale: na::Vector3<f32>,
}

impl Trs {
    pub fn new() -> Self {
        Self {
            isometry: na::Isometry3::identity(),
            scale: na::Vector3::new(1.0, 1.0, 1.0),
        }
    }

    pub fn get_matrix(&self) -> na::Matrix4<f32> {
        self.isometry
            .to_homogeneous()
            .prepend_nonuniform_scaling(&self.scale)
    }

    pub fn rotate(&mut self, rotation: &na::Unit<na::Quaternion<f32>>) {
        self.isometry.append_rotation_mut(&rotation);
    }

    pub fn translate(&mut self, x: f32, y: f32, z: f32) {
        self.isometry
            .append_translation_mut(&na::Translation3::new(x, y, z));
    }

    pub fn scale(&mut self, x: f32, y: f32, z: f32) {
        self.scale.x *= x;
        self.scale.y *= y;
        self.scale.z *= z;
    }

    pub fn get_forward(&self) -> na::Vector3<f32> {
        self.isometry
            .inverse()
            .inverse_transform_vector(&-na::Vector3::z())
            .normalize()
    }
}

#[derive(Clone)]
pub struct Node {
    pub name: String,
    pub trs: Trs,
    pub mesh: Handle<Mesh>,
    pub directional_light: Handle<DirectionalLight>,
    pub point_light: Handle<PointLight>,
    pub camera: Handle<Camera>,
    pub children: Vec<Handle<Node>>,
}

impl Node {
    pub fn new() -> Self {
        Node {
            name: String::new(),
            trs: Trs::new(),
            mesh: Handle::none(),
            directional_light: Handle::none(),
            point_light: Handle::none(),
            camera: Handle::none(),
            children: vec![],
        }
    }

    fn bind(&self, program: &ShaderProgram, transform: &na::Matrix4<f32>) {
        let intr = transform
            .remove_column(3)
            .remove_row(3)
            .try_inverse()
            .unwrap();
        unsafe {
            gl::UniformMatrix4fv(program.loc.model, 1, gl::FALSE, transform.as_ptr());
            if program.loc.model_intr >= 0 {
                gl::UniformMatrix3fv(program.loc.model_intr, 1, gl::TRUE, intr.as_ptr());
            }
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
    pub directional_lights: Pack<DirectionalLight>,
    pub point_lights: Pack<PointLight>,
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
            directional_lights: Pack::new(),
            point_lights: Pack::new(),
            cameras: Pack::new(),
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
    gui_res: GuiRes,

    /// List of shader handles to bind with materials referring to them.
    shaders: HashMap<usize, Vec<usize>>,

    /// Node with the directional light to use for rendering
    directional_light: Handle<Node>,

    /// List of point light handles to use while drawing the scene paired with the node to use
    point_lights: Vec<Handle<Node>>,

    /// List of camera handles to use while drawing the scene paired with the node to use
    cameras: Vec<(Handle<Camera>, Handle<Node>)>,

    /// List of material handles to bind with primitives referring to them.
    materials: HashMap<usize, Vec<usize>>,

    /// List of primitive handles to draw with nodes referring to them.
    /// Together with nodes, we store their transform matrix computed during the scene graph traversal.
    primitives: HashMap<usize, HashMap<usize, na::Matrix4<f32>>>,
}

impl Renderer {
    pub fn new(profile: sdl2::video::GLProfile, fonts: &mut imgui::FontAtlasRefMut) -> Renderer {
        Renderer {
            gui_res: GuiRes::new(profile, fonts),
            shaders: HashMap::new(),
            directional_light: Handle::none(),
            point_lights: Vec::new(),
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
        transform: &na::Matrix4<f32>,
    ) {
        // Precompute transform matrix
        let node = &model.nodes[node_handle.id];
        let temp_transform = transform * node.trs.get_matrix();

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
                        primitive_nodes.insert(node_handle.id, temp_transform);
                    }
                } else {
                    // Create a new entry in the primitive resources
                    let mut primitive_nodes = HashMap::new();
                    primitive_nodes.insert(node_handle.id, temp_transform);
                    self.primitives.insert(primitive_handle.id, primitive_nodes);
                }
            }
        }

        // Check if current node has a directional light and set it for rendering
        if model
            .directional_lights
            .get(&node.directional_light)
            .is_some()
        {
            self.directional_light = *node_handle;
        }

        // Check if current node has a point light and add it to the current list
        if model.point_lights.get(&node.point_light).is_some() {
            self.point_lights.push(*node_handle);
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

            // Bind directional light once for each shader
            if shader.loc.light_color >= 0 {
                let node = model.nodes.get(&self.directional_light).unwrap();
                let directional_light = model
                    .directional_lights
                    .get(&node.directional_light)
                    .unwrap();
                directional_light.bind(shader, node);
            };

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
                            if shader.loc.node_id >= 0 {
                                unsafe {
                                    gl::Uniform1i(shader.loc.node_id, *node_id as i32);
                                }
                            }

                            model.nodes[*node_id].bind(shader, &transform);
                            primitive.draw();
                        }
                    }
                }
            }
        }

        self.shaders.clear();
        self.point_lights.clear();
        self.cameras.clear();
        self.materials.clear();
        self.primitives.clear();
    }

    pub fn draw_gui(&mut self, ui: imgui::Ui) {
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

        self.gui_res.program.enable();

        unsafe {
            gl::UniformMatrix4fv(
                self.gui_res.program.loc.proj,
                1,
                gl::FALSE,
                matrix.as_ptr() as _,
            );
            gl::Uniform1i(self.gui_res.program.loc.tex_sampler, 0);
        }

        for draw_list in data.draw_lists() {
            let vtx_buffer = draw_list.vtx_buffer();
            let idx_buffer = draw_list.idx_buffer();

            self.gui_res.mesh_res.vao.bind();

            self.gui_res.mesh_res.vbo.bind();
            unsafe {
                gl::BufferData(
                    gl::ARRAY_BUFFER,
                    (vtx_buffer.len() * std::mem::size_of::<imgui::DrawVert>()) as _,
                    vtx_buffer.as_ptr() as _,
                    gl::STREAM_DRAW,
                );
            }

            self.gui_res.mesh_res.ebo.bind();
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

#[derive(Clone, Copy, PartialEq)]
pub struct Extent2D {
    pub width: u32,
    pub height: u32,
}

impl Extent2D {
    pub fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }
}

impl Default for Extent2D {
    fn default() -> Self {
        Self::new(0, 0)
    }
}

pub struct Video {
    pub extent: Extent2D,
    system: sdl2::VideoSubsystem,
    window: sdl2::video::Window,
    pub profile: sdl2::video::GLProfile,
    gl: sdl2::video::GLContext,
}

impl Video {
    fn new(sdl: &sdl2::Sdl, extent: Extent2D) -> Self {
        let system = sdl.video().expect("Failed initializing video");

        let attr = system.gl_attr();
        let mut profile = sdl2::video::GLProfile::GLES;
        attr.set_context_profile(profile);
        attr.set_context_version(3, 2);

        attr.set_multisample_buffers(1);
        attr.set_multisample_samples(2);

        let window = match system
            .window("Test", extent.width, extent.height)
            .opengl()
            .position_centered()
            .resizable()
            .build()
        {
            Ok(w) => w,
            Err(_) => {
                println!("Failed initializing GLES profile, trying Core");
                profile = sdl2::video::GLProfile::Core;
                attr.set_context_profile(profile);
                attr.set_context_version(3, 3);
                system
                    .window("Test", extent.width, extent.height)
                    .opengl()
                    .position_centered()
                    .build()
                    .expect("Failed building window")
            }
        };

        let gl = window
            .gl_create_context()
            .expect("Failed creating GL context");

        gl::load_with(|symbol| system.gl_get_proc_address(symbol) as *const _);

        Self {
            extent,
            system,
            window,
            profile,
            gl,
        }
    }
}

pub struct Gfx {
    pub video: Video,
    pub renderer: Renderer,
    pub gui: imgui::Context,
}

impl Gfx {
    pub fn new(sdl: &sdl2::Sdl, extent: Extent2D) -> Self {
        let video = Video::new(sdl, extent);
        let mut gui = imgui::Context::create();
        let renderer = Renderer::new(video.profile, &mut gui.fonts());
        Self {
            video,
            renderer,
            gui,
        }
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

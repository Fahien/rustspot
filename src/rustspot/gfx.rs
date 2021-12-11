use std::ffi::CStr;

use nalgebra as na;

use crate::*;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Vertex {
    pub position: [f32; 3],
    pub color: [f32; 3],
    pub tex_coords: [f32; 2],
    pub normal: na::Vector3<f32>,
    pub tangent: na::Vector3<f32>,
    pub bitangent: na::Vector3<f32>,
}

impl Vertex {
    pub fn new() -> Self {
        Self {
            position: [0.0, 0.0, 0.0],
            color: [1.0, 1.0, 1.0],
            tex_coords: [0.0, 0.0],
            normal: na::Vector3::z(),
            tangent: na::Vector3::zeros(),
            bitangent: na::Vector3::zeros()
        }
    }
}

#[derive(Copy, Clone, Hash, PartialEq, Eq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub fn new() -> Self {
        Self {
            r: 255,
            g: 255,
            b: 255,
            a: 255,
        }
    }

    pub fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub fn as_ptr(&self) -> *const u8 {
        &self.r
    }

    pub fn as_slice(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.as_ptr(), 4) }
    }
}

pub struct Vbo {
    handle: u32,
}

impl Vbo {
    pub fn new() -> Vbo {
        let mut handle = 0;
        unsafe { gl::GenBuffers(1, &mut handle) };
        Vbo { handle }
    }

    pub fn bind(&self) {
        unsafe { gl::BindBuffer(gl::ARRAY_BUFFER, self.handle) };
    }

    pub fn upload<T>(&mut self, vertices: &[T]) {
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
    pub fn new() -> Ebo {
        let mut handle = 0;
        unsafe { gl::GenBuffers(1, &mut handle) };
        Ebo { handle }
    }

    pub fn bind(&self) {
        unsafe { gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.handle) };
    }

    pub fn upload<T>(&mut self, indices: &Vec<T>) {
        self.bind();
        unsafe {
            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                (indices.len() * std::mem::size_of::<T>()) as isize,
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
    pub fn new() -> Vao {
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

/// A node can refer to a camera to apply a transform to place it in the scene
pub struct Camera {
    pub proj: na::Matrix4<f32>,
}

impl Camera {
    pub fn orthographic(width: u32, height: u32) -> Camera {
        let proj = na::Orthographic3::new(
            -(width as f32) / 2.0,
            width as f32 / 2.0,
            -(height as f32) / 2.0,
            (height as f32) / 2.0,
            0.1,
            100.0,
        );
        Camera {
            proj: proj.to_homogeneous(),
        }
    }

    pub fn perspective(width: f32, height: f32) -> Camera {
        let proj = na::Perspective3::new(width / height, 3.14 / 4.0, 0.1, 100.0);
        Camera {
            proj: proj.to_homogeneous(),
        }
    }

    pub fn bind(&self, program: &ShaderProgram, view: &Node) {
        program.enable();

        let view = view.trs.get_view();
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

    pub fn get_translation(&self) -> na::Vector3<f32> {
        na::Vector3::new(
            self.isometry.translation.x,
            self.isometry.translation.y,
            self.isometry.translation.z,
        )
    }

    pub fn get_matrix(&self) -> na::Matrix4<f32> {
        self.isometry
            .to_homogeneous()
            .prepend_nonuniform_scaling(&self.scale)
    }

    pub fn get_view(&self) -> na::Matrix4<f32> {
        self.isometry.inverse().to_homogeneous()
    }

    pub fn rotate(&mut self, rotation: &na::Unit<na::Quaternion<f32>>) {
        self.isometry.append_rotation_mut(&rotation);
    }

    pub fn translate(&mut self, x: f32, y: f32, z: f32) {
        self.isometry
            .append_translation_mut(&na::Translation3::new(x, y, z));
    }

    pub fn set_scale(&mut self, x: f32, y: f32, z: f32) {
        self.scale.x = x;
        self.scale.y = y;
        self.scale.z = z;
    }

    pub fn scale(&mut self, x: f32, y: f32, z: f32) {
        self.scale.x *= x;
        self.scale.y *= y;
        self.scale.z *= z;
    }

    pub fn get_forward(&self) -> na::Vector3<f32> {
        // Does this work?
        self.isometry
            .rotation
            .transform_vector(&-na::Vector3::z())
            .normalize()
    }

    pub fn get_right(&self) -> na::Vector3<f32> {
        self.isometry
            .rotation
            .transform_vector(&na::Vector3::x())
            .normalize()
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
    system: sdl2::VideoSubsystem,
    window: sdl2::video::Window,
    pub gl: sdl2::video::GLContext,
}

impl Video {
    fn get_context_profile() -> sdl2::video::GLProfile {
        if cfg!(feature = "gles") {
            sdl2::video::GLProfile::GLES
        } else {
            sdl2::video::GLProfile::Core
        }
    }

    fn get_context_version() -> (u8, u8) {
        if cfg!(feature = "gles") {
            (3, 2)
        } else {
            (3, 3)
        }
    }

    pub fn get_drawable_extent(&self) -> Extent2D {
        // Default framebuffer drawable size could be different than window size depending on DPI
        let (width, height) = self.window.drawable_size();
        Extent2D::new(width, height)
    }

    fn new(sdl: &sdl2::Sdl, extent: Extent2D) -> Self {
        let system = sdl.video().expect("Failed initializing video");

        let attr = system.gl_attr();
        attr.set_context_profile(Self::get_context_profile());
        let (major, minor) = Self::get_context_version();
        attr.set_context_version(major, minor);

        // We need these only if rendering directly onto default framebuffer
        // attr.set_multisample_buffers(1);
        // attr.set_multisample_samples(2);

        let window = match system
            .window("Test", extent.width, extent.height)
            .opengl()
            .allow_highdpi()
            .position_centered()
            .resizable()
            .build()
        {
            Ok(w) => w,
            Err(_) => {
                panic!("Failed initializing SDL window");
            }
        };

        let gl = window
            .gl_create_context()
            .expect("Failed creating GL context");

        gl::load_with(|symbol| system.gl_get_proc_address(symbol) as *const _);

        Self { system, window, gl }
    }
}

extern "system" fn debug_callback(
    _source: gl::types::GLenum,
    _type: gl::types::GLenum,
    _id: gl::types::GLenum,
    _severity: gl::types::GLenum,
    _length: gl::types::GLsizei,
    message: *const gl::types::GLchar,
    _user_param: *mut libc::c_void,
) {
    let msg = unsafe { CStr::from_ptr(message as _) };
    println!("{}", msg.to_str().unwrap());
}

pub struct Gfx {
    /// The frame can be taken by a client for drawing.
    /// Then it is returned for presenting to screen.
    frame: Option<Frame>,
    pub renderer: Renderer,
    pub gui: imgui::Context,

    pub video: Video,
}

impl Gfx {
    pub fn new(sdl: &sdl2::Sdl, extent: Extent2D, offscreen_extent: Extent2D) -> Self {
        let video = Video::new(sdl, extent);

        if !cfg!(target_os = "macos") {
            unsafe {
                gl::Enable(gl::DEBUG_OUTPUT);
                gl::DebugMessageCallback(Some(debug_callback), std::ptr::null());
            }
        }

        let gl_version = Self::get_gl_version();
        println!("OpenGL v{}.{}", gl_version.0, gl_version.1);

        let mut gui = imgui::Context::create();
        let renderer = Renderer::new(&mut gui.fonts());

        let extent = video.get_drawable_extent();
        let frame = Some(Frame::new(extent, offscreen_extent));

        Self {
            frame,
            renderer,
            gui,
            video,
        }
    }

    pub fn get_gl_version() -> (i32, i32) {
        let (mut major, mut minor) = (0, 0);
        unsafe {
            gl::GetIntegerv(gl::MAJOR_VERSION, &mut major);
            gl::GetIntegerv(gl::MINOR_VERSION, &mut minor);
        }
        (major, minor)
    }

    /// Borrow the frame
    pub fn get_frame(&self) -> &Frame {
        self.frame.as_ref().unwrap()
    }

    pub fn get_frame_mut(&mut self) -> &mut Frame {
        self.frame.as_mut().unwrap()
    }

    pub fn next_frame(&mut self) -> Frame {
        self.frame.take().unwrap()
    }

    pub fn present(&mut self, frame: Frame) {
        self.frame.replace(frame);
        self.video.window.gl_swap_window();
    }
}

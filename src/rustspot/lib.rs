pub mod shader;
pub use shader::*;

pub mod light;
pub use light::*;

pub mod gfx;
pub use gfx::*;

pub mod util;
pub use util::*;

pub struct Spot {
    pub sdl: sdl2::Sdl,
    pub events: sdl2::EventPump,
    pub gfx: Gfx,
}

impl Spot {
    pub fn new() -> Self {
        let sdl = sdl2::init().expect("Failed to initialize SDL2");
        let events = sdl.event_pump().expect("Failed to initialize SDL2 events");

        let gfx = Gfx::new(&sdl);
        let gl_version = gfx.get_gl_version();
        println!("OpenGL v{}.{}", gl_version.0, gl_version.1);

        Spot { sdl, events, gfx }
    }
}

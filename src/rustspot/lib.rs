pub mod shader;
use std::time::Duration;

pub use shader::*;

pub mod light;
pub use light::*;

pub mod gui;
pub use gui::*;

pub mod frame;
pub use frame::*;

pub mod gfx;
pub use gfx::*;

pub mod util;
pub use util::*;

pub struct SpotBuilder {
    extent: Extent2D,
}

impl SpotBuilder {
    pub fn new() -> Self {
        Self {
            extent: Extent2D::new(480, 320),
        }
    }

    pub fn width(mut self, width: u32) -> Self {
        self.extent.width = width;
        self
    }

    pub fn height(mut self, height: u32) -> Self {
        self.extent.height = height;
        self
    }

    pub fn build(self) -> Spot {
        Spot::new(self.extent)
    }
}
pub struct Spot {
    pub timer: Timer,
    pub gfx: Gfx,
    pub events: sdl2::EventPump,
    pub sdl: sdl2::Sdl,
}

impl Spot {
    pub fn builder() -> SpotBuilder {
        SpotBuilder::new()
    }

    pub fn new(extent: Extent2D) -> Self {
        let sdl = sdl2::init().expect("Failed to initialize SDL2");
        let events = sdl.event_pump().expect("Failed to initialize SDL2 events");

        let gfx = Gfx::new(&sdl, extent);
        let gl_version = gfx.get_gl_version();
        println!("OpenGL v{}.{}", gl_version.0, gl_version.1);

        let timer = Timer::new();

        Spot {
            gfx,
            events,
            sdl,
            timer,
        }
    }

    pub fn update(&mut self) -> Duration {
        let delta = self.timer.get_delta();

        // Update GUI
        let ui = self.gfx.gui.io_mut();
        ui.update_delta_time(delta);
        // TODO Should this update be here or somewhere else?
        ui.display_size = [
            self.gfx.video.extent.width as f32,
            self.gfx.video.extent.height as f32,
        ];

        delta
    }
}

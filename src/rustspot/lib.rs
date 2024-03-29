use std::time::Duration;

use clap::{App, Arg, ArgMatches};

pub mod shader;
pub use shader::*;

pub mod shaders;
pub use shaders::*;

pub mod light;
pub use light::*;

pub mod gui;
pub use gui::*;

pub mod frame;
pub use frame::*;

pub mod sky;
pub use sky::*;

pub mod terrain;
pub use terrain::*;

pub mod renderer;
pub use renderer::*;

pub mod texture;
pub use texture::*;

pub mod material;
pub use material::*;

pub mod mesh;
pub use mesh::*;

pub mod node;
pub use node::*;

pub mod input;
pub use input::*;

pub mod model;
pub use model::*;

pub mod gfx;
pub use gfx::*;

pub mod util;
pub use util::*;

pub struct SpotBuilder<'a, 'b> {
    extent: Extent2D,
    offscreen_extent: Extent2D,

    app: App<'a, 'b>,
}

impl<'a, 'b> SpotBuilder<'a, 'b> {
    fn parse_extent(matches: &ArgMatches, name: &str) -> Option<Extent2D> {
        if let Some(extent) = matches.value_of(name) {
            let mut extent = extent.split('x');
            let extent_width = extent
                .next()
                .unwrap()
                .parse()
                .expect("Failed to get extent width");
            let extent_height = extent
                .next()
                .unwrap()
                .parse()
                .expect("Failed to get extent height");
            Some(Extent2D::new(extent_width, extent_height))
        } else {
            None
        }
    }

    pub fn new() -> Self {
        let extent_arg = Arg::with_name("extent").short("e").default_value("480x320");
        let offscreen_extent_arg = Arg::with_name("offscreen-extent")
            .short("o")
            .default_value("480x320");

        let app = App::new("RustSpot")
            .version("0.1.0")
            .author("Antonio Caggiano <info@antoniocaggiano.eu>")
            .about("OpenGL renderer")
            .arg(extent_arg)
            .arg(offscreen_extent_arg);

        Self {
            extent: Extent2D::new(480, 320),
            offscreen_extent: Extent2D::new(480, 320),
            app,
        }
    }

    /// This can be used before calling `build_with_matches()` to add more user specific CLI args
    pub fn arg(mut self, arg: Arg<'a, 'b>) -> Self {
        self.app = self.app.arg(arg);
        self
    }

    pub fn extent(mut self, extent: Extent2D) -> Self {
        self.extent = extent;
        self
    }

    pub fn width(mut self, width: u32) -> Self {
        self.extent.width = width;
        self
    }

    pub fn height(mut self, height: u32) -> Self {
        self.extent.height = height;
        self
    }

    pub fn offscreen_extent(mut self, extent: Extent2D) -> Self {
        self.offscreen_extent = extent;
        self
    }

    pub fn offscreen_width(mut self, width: u32) -> Self {
        self.offscreen_extent.width = width;
        self
    }

    pub fn offscreen_height(mut self, height: u32) -> Self {
        self.offscreen_extent.height = height;
        self
    }

    pub fn build(self) -> Spot {
        let (spot, _) = self.build_with_matches();
        spot
    }

    pub fn build_with_matches(mut self) -> (Spot, ArgMatches<'a>) {
        let matches = self.app.get_matches();

        if let Some(extent) = Self::parse_extent(&matches, "extent") {
            self.extent = extent;
        }
        if let Some(offscreen_extent) = Self::parse_extent(&matches, "offscreen-extent") {
            self.offscreen_extent = offscreen_extent;
        }

        (Spot::new(self.extent, self.offscreen_extent), matches)
    }
}

pub struct Spot {
    pub input: Input,
    pub timer: Timer,
    pub gfx: Gfx,
    pub events: sdl2::EventPump,
    pub joystick: sdl2::JoystickSubsystem,
    pub sdl: sdl2::Sdl,
}

impl Spot {
    pub fn builder<'a, 'b>() -> SpotBuilder<'a, 'b> {
        SpotBuilder::new()
    }

    pub fn new(extent: Extent2D, offscreen_extent: Extent2D) -> Self {
        let sdl = sdl2::init().expect("Failed to initialize SDL2");
        let joystick = sdl
            .joystick()
            .expect("Failed to initialize SDL2 joystick subsystem");
        let events = sdl.event_pump().expect("Failed to initialize SDL2 events");

        let gfx = Gfx::new(&sdl, extent, offscreen_extent);

        let timer = Timer::new();

        let input = Input::new();

        Spot {
            input,
            gfx,
            events,
            joystick,
            sdl,
            timer,
        }
    }

    pub fn update(&mut self) -> Duration {
        let delta = self.timer.get_delta();
        self.gfx.update(delta, &self.input);
        delta
    }
}

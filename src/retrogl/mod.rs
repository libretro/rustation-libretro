//! PlayStation OpenGL 3.3 renderer playing nice with libretro

use libretro;
use gl;
use rustation::gpu::renderer::Renderer;

use self::dummy_state::DummyState;
use self::gl_state::GlState;

#[macro_use]
mod vertex;
mod error;
mod types;
mod buffer;
mod texture;
mod shader;
mod program;
mod dummy_state;
mod gl_state;

pub struct RetroGl {
    state: Box<State>,
}

impl RetroGl {
    pub fn new() -> Result<RetroGl, ()> {
        if !libretro::set_pixel_format(libretro::PixelFormat::Xrgb8888) {
            error!("Can't set pixel format");
            return Err(());
        }

        if !libretro::hw_context::init() {
            error!("Failed to init hardware context");
            return Err(());
        }

        let config = DrawConfig {
            xres: 1024,
            yres: 512,
            draw_area_top_left: (0, 0),
            draw_area_resolution: (0, 0),
            draw_offset: (0, 0),
        };

        Ok(RetroGl {
            // No context until `context_reset` is called
            state: Box::new(DummyState::from_config(config)),
        })
    }

    pub fn context_reset(&mut self) {
        info!("OpenGL context reset");

        // Should I call this at every reset? Does it matter?
        gl::load_with(|s| {
            libretro::hw_context::get_proc_address(s) as *const _
        });

        let config = self.state.draw_config().clone();

        match GlState::from_config(config) {
            Ok(s) => self.state = Box::new(s),
            Err(e) => panic!("Couldn't create RetroGL state: {:?}", e),
        }
    }

    pub fn context_destroy(&mut self) {
        let config = self.state.draw_config().clone();

        self.state = Box::new(DummyState::from_config(config));
    }

    pub fn xres(&self) -> u16 {
        self.state.draw_config().xres
    }

    pub fn yres(&self) -> u16 {
        self.state.draw_config().yres
    }

    pub fn render_frame<F>(&mut self, emulate: F)
        where F: FnOnce(&mut Renderer) {

        self.state.prepare_render();

        emulate(self.state.renderer_mut());

        self.state.finish();
    }
}

pub trait State: Renderer {
    fn draw_config(&self) -> &DrawConfig;

    fn prepare_render(&mut self);
    fn finish(&mut self);

    fn renderer_mut(&mut self) -> &mut Renderer;
}

#[derive(Clone)]
pub struct DrawConfig {
    xres: u16,
    yres: u16,
    draw_offset: (i16, i16),
    draw_area_top_left: (u16, u16),
    draw_area_resolution: (u16, u16),
}

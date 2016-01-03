//! PlayStation OpenGL 3.3 renderer playing nice with libretro

use libretro;
use gl;
use gl::types::GLuint;

use self::buffer::DrawBuffer;
use self::error::Error;
use self::shader::{Shader, ShaderType};
use self::program::Program;

mod error;
mod buffer;
mod vertex;
mod shader;
mod program;

pub struct RetroGl {
    /// Current horizontal resolution of the video output
    xres: u16,
    /// Current vertical resolution of the video output
    yres: u16,
    /// OpenGL state. None if the context is destroyed/not yet
    /// created.
    state: Option<State>,
}

impl RetroGl {
    pub fn new() -> Result<RetroGl, ()> {
        if !libretro::hw_context::init() {
            error!("Failed to init hardware context");
            return Err(());
        }

        Ok(RetroGl {
            xres: 640,
            yres: 480,
            // Wait until `context_reset` is called
            state: None,
        })
    }

    pub fn context_reset(&mut self) {
        info!("OpenGL context reset");

        // Should I call this at every reset? Does it matter?
        gl::load_with(|s| {
            libretro::hw_context::get_proc_address(s) as *const _
        });

        match State::new() {
            Ok(s) => self.state = Some(s),
            Err(e) => error!("Couldn't create RetroGL state: {:?}", e),
        }
    }

    pub fn context_destroy(&mut self) {
        self.state = None
    }

    pub fn xres(&self) -> u16 {
        self.xres
    }

    pub fn yres(&self) -> u16 {
        self.yres
    }

    pub fn state(&mut self) -> Option<&mut State> {
        self.state.as_mut()
    }
}

pub struct State {
    buffer: DrawBuffer<(f32, f32)>,
}

impl State {
    fn new() -> Result<State, Error> {

        info!("Building RetroGL state");

        let vs = try!(Shader::new(include_str!("shaders/vertex.glsl"),
                                  ShaderType::Vertex));

        let fs = try!(Shader::new(include_str!("shaders/fragment.glsl"),
                                  ShaderType::Fragment));

        let program = try!(Program::new(vs, fs));

        let buffer = try!(DrawBuffer::new(128, program));

        Ok(State {
            buffer: buffer,
        })
    }

    pub fn render_frame(&mut self) -> Result<(), Error> {

        let r = self.do_render_frame();

        // Cleanup OpenGL context before returning to the frontend
        unsafe {
            gl::UseProgram(0);
            gl::BindVertexArray(0);
            gl::BindFramebuffer(gl::DRAW_FRAMEBUFFER, 0);
        }

        r
    }

    fn do_render_frame(&mut self) -> Result<(), Error> {

        try!(self.buffer.push_slice(&[(0., 1.),
                                      (-1., -1.),
                                      (1., -1.),
                                      ]));

        // Bind the output framebuffer provided by the frontend
        let fbo = libretro::hw_context::get_current_framebuffer() as GLuint;

        unsafe {
            gl::BindFramebuffer(gl::DRAW_FRAMEBUFFER, fbo);
            gl::Viewport(0, 0, 640, 480);
        }

        unsafe {
            gl::ClearColor(0.3, 0.4, 0.5, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }

        try!(self.buffer.draw_triangles());

        self.buffer.clear()
    }
}
// STD Dependencies -----------------------------------------------------------
use std::time::Duration;


// External Dependencies ------------------------------------------------------
use glutin;
use glutin_window::GlutinWindow;
use opengl_graphics::OpenGL;
use piston::window::{Size, Window, WindowSettings, OpenGLWindow};
use piston::input::RenderArgs;
use piston::event_loop::{Events, WindowEvents};
use clock_ticks;


// Renderer Abstraction -------------------------------------------------------
pub struct Renderer {
    window: GlutinWindow,
    updates_per_second: u64,
    width: f64,
    height: f64,
    t: u64,
    u: f64
}

impl Renderer {

    pub fn new(
        title: &str,
        width: u32,
        height: u32,
        updates_per_second: u64

    ) -> Renderer {

        // Create Window
        let opengl = OpenGL::V3_2;
        let window: GlutinWindow = WindowSettings::new(
                title,
                [width, height]
            )
            .opengl(opengl)
            .samples(8)
            .vsync(false)
            .exit_on_esc(true)
            .build()
            .unwrap();

        // Hide Cursor
        window.window.set_cursor_state(glutin::CursorState::Hide).ok();

        Renderer {
            window: window,
            updates_per_second: updates_per_second,
            width: width as f64,
            height: height as f64,
            t: clock_ticks::precise_time_ms(),
            u: 0.0
        }

    }

    // Events -----------------------------------------------------------------
    pub fn events(&self) -> WindowEvents {
        self.window.events()
    }


    // Rendering --------------------------------------------------------------
    pub fn begin(&mut self, args: RenderArgs) {
        self.t = clock_ticks::precise_time_ms();
        self.u = 1.0 / (1.0 / self.updates_per_second as f64) * (args.ext_dt * 1000000000.0);
        self.width = args.draw_width as f64;
        self.height = args.draw_height as f64;
        self.window.make_current();
    }

    #[inline]
    pub fn get_t(&self) -> u64 {
        self.t
    }

    #[inline]
    pub fn get_u(&self) -> f64 {
        self.u
    }

    #[inline]
    pub fn width(&self) -> f64 {
        self.width
    }

    #[inline]
    pub fn height(&self) -> f64 {
        self.height
    }

    pub fn end(&mut self) {

    }


    // Rendering Operations ---------------------------------------------------
    pub fn clear(&mut self, color: [f32; 4]) {

    }

    pub fn line(&mut self) {

    }

    pub fn rectangle(&mut self) {

    }

    pub fn circle(&mut self) {

    }

}


// Traits ---------------------------------------------------------------------
impl Window for Renderer {

    type Event = <GlutinWindow as Window>::Event;

    fn should_close(&self) -> bool { self.window.should_close() }
    fn set_should_close(&mut self, value: bool) {
        self.window.set_should_close(value)
    }
    fn size(&self) -> Size { self.window.size() }
    fn draw_size(&self) -> Size { self.window.draw_size() }
    fn swap_buffers(&mut self) { self.window.swap_buffers() }
    fn wait_event(&mut self) -> Self::Event {
        GlutinWindow::wait_event(&mut self.window)
    }
    fn wait_event_timeout(&mut self, timeout: Duration) -> Option<Self::Event> {
        GlutinWindow::wait_event_timeout(&mut self.window, timeout)
    }
    fn poll_event(&mut self) -> Option<Self::Event> {
        GlutinWindow::poll_event(&mut self.window)
    }

}


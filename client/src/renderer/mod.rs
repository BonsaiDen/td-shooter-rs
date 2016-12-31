// STD Dependencies -----------------------------------------------------------
use std::time::Duration;


// External Dependencies ------------------------------------------------------
use clock_ticks;


// Glutin Dependencies --------------------------------------------------------
use glutin;
use glutin_window::GlutinWindow;
use shader_version::{ OpenGL, Shaders };
use shader_version::glsl::GLSL;


// GFX Dependencies -----------------------------------------------------------
use gfx_device_gl;
use gfx;
use gfx::Device;
use gfx::Factory;
use gfx::traits::FactoryExt;
use gfx::memory::Typed;
use gfx::format::{DepthStencil, Format, Formatted, Srgba8};
use gfx::pso::PipelineState;


// Piston Dependencies --------------------------------------------------------
use piston::window::{Size, Window, WindowSettings, OpenGLWindow};
use piston::input::RenderArgs;
use piston::event_loop::{Events, WindowEvents};


// Graphics Dependencies ------------------------------------------------------
use graphics::Context;
use graphics::math::Matrix2d;
use graphics::color::gamma_srgb_to_linear;
use graphics::BACK_END_MAX_VERTEX_COUNT as BUFFER_SIZE;


// Modules --------------------------------------------------------------------
mod shapes;
pub use self::shapes::*;

mod stencil;
pub use self::stencil::*;


// Statics --------------------------------------------------------------------
const POS_COMPONENTS: usize = 2;
const CHUNKS: usize = 100;

static VERTEX_SHADER_120: &'static [u8] = br#"
    #version 120
    attribute vec4 color;
    attribute vec2 pos;

    varying vec4 v_Color;
    uniform mat4 u_View;

    void main() {
        v_Color = color;
        gl_Position = u_View * vec4(pos, 0.0, 1.0);
    }
"#;

static VERTEX_SHADER_150: &'static [u8] = br#"
    #version 150 core
    in vec4 color;
    in vec2 pos;

    out vec4 v_Color;

    uniform Locals {
        mat4 u_View;
    };

    void main() {
        v_Color = color;
        gl_Position = u_View * vec4(pos, 0.0, 1.0);
    }
"#;


// Rendering Pipeline ---------------------------------------------------------
gfx_defines! {
    vertex PositionFormat {
        pos: [f32; 2] = "pos",
    }

    constant Locals {
        view: [[f32; 4]; 4] = "u_View",
    }

    vertex ColorFormat {
        color: [f32; 4] = "color",
    }

    vertex TexCoordsFormat {
        uv: [f32; 2] = "uv",
    }
}

gfx_pipeline_base!( pipe_colored {
    pos: gfx::VertexBuffer<PositionFormat>,
    locals: gfx::ConstantBuffer<Locals>,
    color: gfx::VertexBuffer<ColorFormat>,
    blend_target: gfx::BlendTarget<gfx::format::Srgba8>,
    stencil_target: gfx::StencilTarget<gfx::format::DepthStencil>,
    blend_ref: gfx::BlendRef,
    scissor: gfx::Scissor,
});


// Renderer Implementation ----------------------------------------------------
pub struct Renderer {
    window: GlutinWindow,
    updates_per_second: u64,
    width: f32,
    height: f32,
    context: Context,
    color: [f32; 4],
    stencil_mode: StencilMode,
    t: u64,
    u: f32,

    device: gfx_device_gl::Device,
    encoder: gfx::Encoder<gfx_device_gl::Resources, gfx_device_gl::CommandBuffer>,
    output_color: gfx::handle::RenderTargetView<gfx_device_gl::Resources, gfx::format::Srgba8>,
    output_stencil: gfx::handle::DepthStencilView<gfx_device_gl::Resources, gfx::format::DepthStencil>,

    buffer_pos: gfx::handle::Buffer<gfx_device_gl::Resources, PositionFormat>,
    buffer_color: gfx::handle::Buffer<gfx_device_gl::Resources, ColorFormat>,
    buffer_locals: gfx::handle::Buffer<gfx_device_gl::Resources, Locals>,

    colored_pso: ColoredStencil<PipelineState<gfx_device_gl::Resources, pipe_colored::Meta>>,
    colored_offset: usize
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
        let samples = 4;
        let mut window: GlutinWindow = WindowSettings::new(
                title,
                [width, height]
            )
            .opengl(opengl)
            .samples(samples)
            .vsync(false)
            .exit_on_esc(true)
            .build()
            .unwrap();

        // Hide Cursor
        window.window.set_cursor_state(glutin::CursorState::Hide).ok();

        // OpenGL Context
        let (device, mut factory) = gfx_device_gl::create(|s| {
            window.get_proc_address(s) as *const _
        });

        // Buffers
        let (output_color, output_stencil) = create_main_targets((
            width as u16,
            height as u16,
            1,
            samples.into()
        ));

        // Pipeline
        let colored_pso = create_pipeline(opengl, &mut factory);

        // Buffers
        let buffer_pos = factory.create_buffer_dynamic(
            BUFFER_SIZE * CHUNKS,
            gfx::buffer::Role::Vertex,
            gfx::Bind::empty()

        ).expect("Could not create `buffer_pos`");

        let buffer_color = factory.create_buffer_dynamic(
            BUFFER_SIZE * CHUNKS,
            gfx::buffer::Role::Vertex,
            gfx::Bind::empty()

        ).expect("Could not create `buffer_color`");

        let buffer_locals = factory.create_constant_buffer(1);

        // GFX Encoder
        let encoder = factory.create_command_buffer().into();

        Renderer {

            window: window,
            updates_per_second: updates_per_second,
            width: width as f32,
            height: height as f32,
            color: [0.0; 4],
            context: Context::new(),
            stencil_mode: StencilMode::None,
            t: clock_ticks::precise_time_ms(),
            u: 0.0,

            device: device,
            encoder: encoder,
            output_color: output_color,
            output_stencil: output_stencil,

            buffer_pos: buffer_pos,
            buffer_color: buffer_color,
            buffer_locals: buffer_locals,
            colored_pso: colored_pso,
            colored_offset: 0

        }

    }

    // Events -----------------------------------------------------------------
    pub fn events(&self) -> WindowEvents {
        self.window.events()
    }


    // Rendering --------------------------------------------------------------
    pub fn begin(&mut self, args: RenderArgs) {
        self.t = clock_ticks::precise_time_ms();
        self.u = 1.0 / (1.0 / self.updates_per_second as f32) * (args.ext_dt as f32 * 1000000000.0);
        self.width = args.draw_width as f32;
        self.height = args.draw_height as f32;
        self.window.make_current();
        self.stencil_mode = StencilMode::None;
        self.context = Context::new_viewport(args.viewport());
    }

    pub fn context(&self) -> &Context {
        &self.context
    }

    #[inline]
    pub fn t(&self) -> u64 {
        self.t
    }

    #[inline]
    pub fn u(&self) -> f32 {
        self.u
    }

    #[inline]
    pub fn width(&self) -> f32 {
        self.width
    }

    #[inline]
    pub fn height(&self) -> f32 {
        self.height
    }

    pub fn end(&mut self) {

        self.encoder.flush(&mut self.device);
        self.device.cleanup();

        // Check for window resize
        let dim = self.output_color.raw().get_dimensions();
        if dim.0 != self.width as u16 || dim.1 != self.height as u16 {
            let dim = (self.width as u16, self.height as u16, dim.2, dim.3);
            let (output_color, output_stencil) = create_main_targets(dim);
            self.output_color = output_color;
            self.output_stencil = output_stencil;
        }

    }

    // Rendering Operations ---------------------------------------------------
    pub fn set_color(&mut self, color: [f32; 4]) {
        self.color = color;
    }

    pub fn clear_color(&mut self, color: [f32; 4]) {
        let color = gamma_srgb_to_linear(color);
        self.encoder.clear(&self.output_color, color);
    }

    pub fn set_stencil_mode(&mut self, mode: StencilMode) {
        self.stencil_mode = mode;
    }

    pub fn clear_stencil(&mut self, value: u8) {
        self.encoder.clear_stencil(&self.output_stencil, value);
    }


    // Direct Shape Drawing ---------------------------------------------------
    pub fn light_polygon(
        &mut self,
        context: &Context,
        x: f32, y: f32,
        endpoints: &[(usize, (f32, f32), (f32, f32))]
    ) {
        self.draw_triangle_list(
            &context.transform,
            &LightPoylgon::vertices(x, y, endpoints)
        );
    }

    pub fn rectangle(&mut self, context: &Context, rect: &[f32; 4]) {
        let (x, y, w, h) = (rect[0], rect[1], rect[2], rect[3]);
        let (x2, y2) = (x + w, y + h);
        let vertices = [
             x,  y,
            x2,  y,
             x, y2,
            x2,  y,
            x2, y2,
             x, y2
        ];
        self.draw_triangle_list(&context.transform, &vertices);
    }

    pub fn line(&mut self, context: &Context, p: &[f32; 4], width: f32) {
        self.draw_triangle_list(&context.transform, &Line::vertices(p, width));
    }


    // Internal ---------------------------------------------------------------
    fn draw_triangle_list(&mut self, m: &Matrix2d, vertices: &[f32]) {

        let color = gamma_srgb_to_linear(self.color);
        let n = vertices.len() / POS_COMPONENTS;

        {
            use std::slice::from_raw_parts;

            unsafe {
                self.encoder.update_buffer(
                    &self.buffer_pos,
                    from_raw_parts(
                        vertices.as_ptr() as *const PositionFormat,
                        n
                    ),
                    self.colored_offset

                ).unwrap();
            }

            for i in 0..n {
                self.encoder.update_buffer(
                    &self.buffer_color, &[
                        ColorFormat {
                            color: color
                        }
                    ],
                    self.colored_offset + i

                ).unwrap();
            }

            self.colored_offset += n;

        }

        self.encoder.update_constant_buffer(&self.buffer_locals, &Locals {
            view: [
                // Rotation
                [m[0][0] as f32, m[1][0] as f32, 0.0, 0.0],
                [m[0][1] as f32, m[1][1] as f32, 0.0, 0.0],

                // Identity
                [0.0, 0.0, 1.0, 0.0],

                // Translation
                [m[0][2] as f32, m[1][2] as f32, 0.0, 1.0]

            ]
        });

        self.flush();

    }

    fn flush(&mut self) {

        if self.colored_offset > 0 {

            use draw_state::target::Rect;
            use std::u16;

            let (pso_colored, stencil_val) = self.colored_pso.get(
                self.stencil_mode
            );

            let data = pipe_colored::Data {
                pos: self.buffer_pos.clone(),
                color: self.buffer_color.clone(),
                locals: self.buffer_locals.clone(),
                blend_target: self.output_color.clone(),
                stencil_target: (
                    self.output_stencil.clone(),
                    (stencil_val, stencil_val)
                ),
                // Use white color for blend reference to make invert work.
                blend_ref: [1.0; 4],
                scissor: Rect { x: 0, y: 0, w: u16::MAX, h: u16::MAX }
            };

            let slice = gfx::Slice {
                instances: None,
                start: 0,
                end: self.colored_offset as u32,
                buffer: gfx::IndexBuffer::Auto,
                base_vertex: 0,
            };

            self.encoder.draw(&slice, pso_colored, &data);
            self.colored_offset = 0;

        }

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


// Helpers --------------------------------------------------------------------
fn create_main_targets(dim: gfx::texture::Dimensions) -> (
    gfx::handle::RenderTargetView<gfx_device_gl::Resources, gfx::format::Srgba8>,
    gfx::handle::DepthStencilView<gfx_device_gl::Resources, gfx::format::DepthStencil>

) {
    let color_format: Format = <Srgba8 as Formatted>::get_format();
    let depth_format: Format = <DepthStencil as Formatted>::get_format();
    let (output_color, output_stencil) = gfx_device_gl::create_main_targets_raw(
        dim,
        color_format.0,
        depth_format.0
    );

    (Typed::new(output_color), Typed::new(output_stencil))
}

fn create_pipeline(
    opengl: OpenGL,
    factory: &mut gfx_device_gl::Factory

) -> ColoredStencil<gfx::PipelineState<gfx_device_gl::Resources, pipe_colored::Meta>> {

    use gfx::Primitive;
    use gfx::state::{Blend, Stencil, Rasterizer, MultiSample};
    use gfx::traits::*;
    use shaders_graphics2d::colored;

    let glsl = opengl.to_glsl();

    let colored_program = factory.link_program(
        Shaders::new()
            .set(GLSL::V1_20, VERTEX_SHADER_120)
            .set(GLSL::V1_50, VERTEX_SHADER_150)
            .get(glsl).unwrap(),

        Shaders::new()
            .set(GLSL::V1_20, colored::FRAGMENT_GLSL_120)
            .set(GLSL::V1_50, colored::FRAGMENT_GLSL_150_CORE)
            .get(glsl).unwrap(),

    ).unwrap();

    let polygon_pipeline = |
        factory: &mut gfx_device_gl::Factory,
        blend_preset: Blend,
        stencil: Stencil,
        color_mask: gfx::state::ColorMask

    | -> PipelineState<gfx_device_gl::Resources, pipe_colored::Meta> {

        let mut r = Rasterizer::new_fill();
        r.samples = Some(MultiSample);

        factory.create_pipeline_from_program(
            &colored_program,
            Primitive::TriangleList,
            r,
            pipe_colored::Init {
                pos: (),
                locals: "Locals",
                color: (),
                blend_target: ("o_Color", color_mask, blend_preset),
                stencil_target: stencil,
                blend_ref: (),
                scissor: (),
            }

        ).unwrap()

    };

    ColoredStencil::new(factory, polygon_pipeline)

}



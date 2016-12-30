// STD Dependencies -----------------------------------------------------------
use std::f64::consts;
use std::time::Duration;

// External Dependencies ------------------------------------------------------
use clock_ticks;


// Glutin Dependencies --------------------------------------------------------
use glutin;
use glutin_window::GlutinWindow;
use shader_version::{ OpenGL, Shaders };
use shader_version::glsl::GLSL;


// GFX Dependencies -----------------------------------------------------------
use gfx;
use gfx::Device;
use gfx::Factory;
use gfx::memory::Typed;
use gfx::format::{DepthStencil, Format, Formatted, Srgba8};
use gfx::pso::PipelineState;
use gfx_device_gl;


// Piston Dependencies --------------------------------------------------------
use piston::window::{Size, Window, WindowSettings, OpenGLWindow};
use piston::input::RenderArgs;
use piston::event_loop::{Events, WindowEvents};

use graphics::Context;
use graphics::math::{Matrix2d, Scalar};
use graphics::color::gamma_srgb_to_linear;
use graphics::BACK_END_MAX_VERTEX_COUNT as BUFFER_SIZE;


// Statics --------------------------------------------------------------------
const POS_COMPONENTS: usize = 2;
const CHUNKS: usize = 100;


// Rendering Pipeline ---------------------------------------------------------
gfx_defines! {
    vertex PositionFormat {
        pos: [f32; 2] = "pos",
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
    color: gfx::VertexBuffer<ColorFormat>,
    blend_target: gfx::BlendTarget<gfx::format::Srgba8>,
    stencil_target: gfx::StencilTarget<gfx::format::DepthStencil>,
    blend_ref: gfx::BlendRef,
    scissor: gfx::Scissor,
});


#[derive(Debug, Copy, Clone)]
pub enum StencilMode {
    None,
    Add(u8),
    Replace(u8),
    Inside(u8),
    Outside(u8)
}

struct ColoredStencil<T> {
    none: T,
    add: T,
    replace: T,
    inside: T,
    outside: T
}

impl<T> ColoredStencil<T> {

    fn new<F>(factory: &mut gfx_device_gl::Factory, f: F) -> ColoredStencil<T> where F: Fn(
        &mut gfx_device_gl::Factory,
        gfx::state::Blend,
        gfx::state::Stencil,
        gfx::state::ColorMask

    ) -> T {

        use gfx::preset::blend;
        use gfx::state::{Comparison, Stencil, StencilOp};

        ColoredStencil {

            none: f(factory, blend::ALPHA, Stencil::new(
                Comparison::Always,
                0,
                (StencilOp::Keep, StencilOp::Keep, StencilOp::Keep)

            ), gfx::state::MASK_ALL),

            add: f(factory, blend::ALPHA, Stencil::new(
                Comparison::Never,
                255,
                (StencilOp::IncrementClamp, StencilOp::IncrementClamp, StencilOp::IncrementClamp)

            ), gfx::state::MASK_NONE),

            replace: f(factory, blend::ALPHA, Stencil::new(
                Comparison::Never,
                255,
                (StencilOp::Replace, StencilOp::Keep, StencilOp::Keep)

            ), gfx::state::MASK_NONE),

            inside: f(factory, blend::ALPHA, Stencil::new(
                Comparison::Equal,
                255,
                (StencilOp::Keep, StencilOp::Keep, StencilOp::Keep)

            ), gfx::state::MASK_ALL),

            outside: f(factory, blend::ALPHA, Stencil::new(
                Comparison::NotEqual,
                255,
                (StencilOp::Keep, StencilOp::Keep, StencilOp::Keep)

            ), gfx::state::MASK_ALL)

        }

    }

    fn get(&mut self, mode: StencilMode) -> (&mut T, u8) {
        match mode {
            StencilMode::None => (&mut self.none, 0),
            StencilMode::Add(v) => (&mut self.add, v),
            StencilMode::Replace(v) => (&mut self.replace, v),
            StencilMode::Inside(v) => (&mut self.inside, v),
            StencilMode::Outside(v) => (&mut self.outside, v)
        }
    }

}


// Renderer Abstraction -------------------------------------------------------
pub struct Renderer {

    window: GlutinWindow,
    updates_per_second: u64,
    width: f64,
    height: f64,
    context: Context,
    color: [f32; 4],
    stencil_mode: StencilMode,
    t: u64,
    u: f64,

    device: gfx_device_gl::Device,
    encoder: gfx::Encoder<gfx_device_gl::Resources, gfx_device_gl::CommandBuffer>,
    output_color: gfx::handle::RenderTargetView<gfx_device_gl::Resources, gfx::format::Srgba8>,
    output_stencil: gfx::handle::DepthStencilView<gfx_device_gl::Resources, gfx::format::DepthStencil>,

    buffer_pos: gfx::handle::Buffer<gfx_device_gl::Resources, PositionFormat>,
    buffer_color: gfx::handle::Buffer<gfx_device_gl::Resources, ColorFormat>,

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

        // GFX Encoder
        let encoder = factory.create_command_buffer().into();

        Renderer {

            window: window,
            updates_per_second: updates_per_second,
            width: width as f64,
            height: height as f64,
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
        self.u = 1.0 / (1.0 / self.updates_per_second as f64) * (args.ext_dt * 1000000000.0);
        self.width = args.draw_width as f64;
        self.height = args.draw_height as f64;
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
    pub fn u(&self) -> f64 {
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

        self.flush_colored();
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

    fn flush_colored(&mut self) {

        if self.colored_offset > 0 {

            use draw_state::target::Rect;
            use std::u16;

            let (pso_colored, stencil_val) = self.colored_pso.get(
                self.stencil_mode
            );

            let data = pipe_colored::Data {
                pos: self.buffer_pos.clone(),
                color: self.buffer_color.clone(),
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

    pub fn draw_triangle_list(&mut self, color: [f32; 4], vertices: &[f32]) {

        let color = gamma_srgb_to_linear(color);
        let n = vertices.len() / POS_COMPONENTS;

        // Render if there is not enough room.
        if self.colored_offset + n > BUFFER_SIZE * CHUNKS {
            self.flush_colored();
        }

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
        self.flush_colored();
        self.stencil_mode = mode;
    }

    pub fn clear_stencil(&mut self, value: u8) {
        self.encoder.clear_stencil(&self.output_stencil, value);
    }

    pub fn line(&mut self) {
        // TODO flush with line pipeline
    }

    pub fn rectangle(&mut self, context: &Context, rect: &[f64; 4]) {
        let m = context.transform;
        let (x, y, w, h) = (rect[0], rect[1], rect[2], rect[3]);
        let (x2, y2) = (x + w, y + h);
        let vertices = [
            tx(m,  x,  y), ty(m,  x,  y),
            tx(m, x2,  y), ty(m, x2,  y),
            tx(m,  x, y2), ty(m,  x, y2),
            tx(m, x2,  y), ty(m, x2,  y),
            tx(m, x2, y2), ty(m, x2, y2),
            tx(m,  x, y2), ty(m,  x, y2)
        ];
        let color = self.color;
        self.draw_triangle_list(color, &vertices);
    }

    pub fn circle(
        &mut self,
        context: &Context,
        segments: usize,
        x: f64,
        y: f64,
        r: f64
    ) {

        // TODO support circles with openings?
        // TODO use shaders for circle drawing?

        // TODO pre-calculate base circles?
        let m = context.transform;
        let step = consts::PI * 2.0 / segments as f64;
        let mut vertices = Vec::new();
        for i in 0..segments {

            // Center
            vertices.push(tx(m, x, y));
            vertices.push(ty(m, x, y));

            // First outer point
            let ar = i as f64 * step;
            let (ax, ay) = (x + ar.cos() * r, y + ar.sin() * r);
            vertices.push(tx(m, ax, ay));
            vertices.push(ty(m, ax, ay));

            // Second outer point
            let br = ar + step;
            let (bx, by) = (x + br.cos() * r, y + br.sin() * r);
            vertices.push(tx(m, bx, by));
            vertices.push(ty(m, bx, by));

        }

        let color = self.color;
        self.draw_triangle_list(color, &vertices);

    }

}


// Helpers --------------------------------------------------------------------
#[inline(always)]
fn tx(m: Matrix2d, x: Scalar, y: Scalar) -> f32 {
    (m[0][0] * x + m[0][1] * y + m[0][2]) as f32
}

#[inline(always)]
fn ty(m: Matrix2d, x: Scalar, y: Scalar) -> f32 {
    (m[1][0] * x + m[1][1] * y + m[1][2]) as f32
}

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
    use gfx::state::{Blend, Stencil, Rasterizer, MultiSample, RasterMethod};
    use gfx::traits::*;
    use shaders_graphics2d::colored;

    let glsl = opengl.to_glsl();

    let colored_program = factory.link_program(
        Shaders::new()
            .set(GLSL::V1_20, colored::VERTEX_GLSL_120)
            .set(GLSL::V1_50, colored::VERTEX_GLSL_150_CORE)
            .get(glsl).unwrap(),

        Shaders::new()
            .set(GLSL::V1_20, colored::FRAGMENT_GLSL_120)
            .set(GLSL::V1_50, colored::FRAGMENT_GLSL_150_CORE)
            .get(glsl).unwrap(),

    ).unwrap();

    let colored_pipeline = |
        factory: &mut gfx_device_gl::Factory,
        blend_preset: Blend,
        stencil: Stencil,
        color_mask: gfx::state::ColorMask

    | -> PipelineState<gfx_device_gl::Resources, pipe_colored::Meta> {

        let mut r = Rasterizer::new_fill();
        // TODO create an additional line pipeline
        //r.method = RasterMethod::Line(3);
        // TODO have another pipeline without multisampling for UI?
        r.samples = Some(MultiSample);

        factory.create_pipeline_from_program(
            &colored_program,
            Primitive::TriangleList,
            r,
            pipe_colored::Init {
                pos: (),
                color: (),
                blend_target: ("o_Color", color_mask, blend_preset),
                stencil_target: stencil,
                blend_ref: (),
                scissor: (),
            }

        ).unwrap()

    };

    ColoredStencil::new(factory, colored_pipeline)

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


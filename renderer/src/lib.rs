// Crates ---------------------------------------------------------------------
#[macro_use]
extern crate gfx;
extern crate gfx_window_glutin;
extern crate gfx_device_gl;
extern crate glutin;
extern crate glutin_window;
extern crate shader_version;
extern crate clock_ticks;
extern crate draw_state;
extern crate graphics;
extern crate piston;


// STD Dependencies -----------------------------------------------------------
use std::iter;
use std::time::Duration;


// Glutin Dependencies --------------------------------------------------------
use glutin_window::GlutinWindow;
use shader_version::OpenGL;


// GFX Dependencies -----------------------------------------------------------
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
mod data;
mod shape;
mod texture;
mod pipeline;

use self::data::*;
use self::pipeline::RenderPipeline;
pub use self::shape::*;
pub use self::texture::*;
pub use self::pipeline::StencilMode;


// Statics --------------------------------------------------------------------
pub const MAX_PARTICLES: usize = 400;
const CHUNKS: usize = 100;


// Renderer Implementation ----------------------------------------------------
pub struct Renderer {

    updates_per_second: u64,
    width: f32,
    height: f32,
    context: Context,
    color: [f32; 4],
    texture: [f32; 4],
    stencil_mode: StencilMode,
    t: u64,
    dt: f32,
    u: f32,

    wireframe: bool,

    particle_position: Vec<f32>,
    particle_scale: Vec<f32>,
    particle_color: Vec<[f32; 4]>,

    window: GlutinWindow,
    device: gfx_device_gl::Device,
    encoder: gfx::Encoder<gfx_device_gl::Resources, gfx_device_gl::CommandBuffer>,
    primitive: gfx::Primitive,

    output_color: gfx::handle::RenderTargetView<gfx_device_gl::Resources, gfx::format::Srgba8>,
    output_stencil: gfx::handle::DepthStencilView<gfx_device_gl::Resources, gfx::format::DepthStencil>,

    buffer_matrix: [[f32; 4]; 4],
    buffer_pos: gfx::handle::Buffer<gfx_device_gl::Resources, PositionFormat>,
    buffer_scale: gfx::handle::Buffer<gfx_device_gl::Resources, ScaleFormat>,
    buffer_color: gfx::handle::Buffer<gfx_device_gl::Resources, ColorFormat>,
    buffer_locals: gfx::handle::Buffer<gfx_device_gl::Resources, Locals>,
    buffer_texture: gfx::handle::Buffer<gfx_device_gl::Resources, TextureLocals>,
    buffer_offset: usize,

    list_pipeline: RenderPipeline<PipelineState<gfx_device_gl::Resources, pipe_colored::Meta>>,
    wire_pipeline: RenderPipeline<PipelineState<gfx_device_gl::Resources, pipe_colored::Meta>>,
    point_pipeline: RenderPipeline<PipelineState<gfx_device_gl::Resources, pipe_colored::Meta>>,
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
            //.fullscreen(true)
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

        // Buffers
        let buffer_locals = factory.create_constant_buffer(1);
        let buffer_texture = factory.create_constant_buffer(1);
        let buffer_pos = factory.create_buffer_dynamic(
            BUFFER_SIZE * CHUNKS,
            gfx::buffer::Role::Vertex,
            gfx::Bind::empty()

        ).expect("Could not create `buffer_pos`");

        let buffer_scale = factory.create_buffer_dynamic(
            BUFFER_SIZE * CHUNKS,
            gfx::buffer::Role::Vertex,
            gfx::Bind::empty()

        ).expect("Could not create `buffer_scale`");

        let buffer_color = factory.create_buffer_dynamic(
            BUFFER_SIZE * CHUNKS,
            gfx::buffer::Role::Vertex,
            gfx::Bind::empty()

        ).expect("Could not create `buffer_color`");

        // Pipelines
        let list_pipeline = RenderPipeline::<()>::create(
            opengl, &mut factory,
            gfx::Primitive::TriangleList,
            gfx::state::RasterMethod::Fill
        );

        let wire_pipeline = RenderPipeline::<()>::create(
            opengl, &mut factory,
            gfx::Primitive::TriangleList,
            gfx::state::RasterMethod::Line(1)
        );

        let point_pipeline = RenderPipeline::<()>::create(
            opengl, &mut factory,
            gfx::Primitive::PointList,
            gfx::state::RasterMethod::Fill
        );

        // GFX Encoder
        let encoder = factory.create_command_buffer().into();

        Renderer {

            updates_per_second: updates_per_second,
            width: width as f32,
            height: height as f32,
            color: [0.0; 4],
            texture: [0.0; 4],
            context: Context::new(),
            stencil_mode: StencilMode::None,
            t: clock_ticks::precise_time_ms(),
            dt: 0.0,
            u: 0.0,

            wireframe: false,

            // TODO optimize further
            particle_position: iter::repeat(0.0).take(MAX_PARTICLES * 2).collect(),
            particle_scale: iter::repeat(0.0).take(MAX_PARTICLES * 2).collect(),
            particle_color: iter::repeat([0f32; 4]).take(MAX_PARTICLES).collect(),

            window: window,
            device: device,
            encoder: encoder,
            primitive: gfx::Primitive::TriangleList,

            output_color: output_color,
            output_stencil: output_stencil,

            buffer_matrix: [[0.0; 4]; 4],
            buffer_pos: buffer_pos,
            buffer_scale: buffer_scale,
            buffer_color: buffer_color,
            buffer_locals: buffer_locals,
            buffer_texture: buffer_texture,
            buffer_offset: 0,

            list_pipeline: list_pipeline,
            wire_pipeline: wire_pipeline,
            point_pipeline: point_pipeline

        }

    }

    // Events -----------------------------------------------------------------
    pub fn events(&self) -> WindowEvents {
        self.window.events()
    }


    // Configuration ----------------------------------------------------------
    pub fn wireframe(&mut self) -> bool {
        self.wireframe
    }

    pub fn set_wireframe(&mut self, enabled: bool) {
        self.wireframe = enabled;
    }


    // Rendering --------------------------------------------------------------
    pub fn begin(&mut self, args: RenderArgs) {
        self.t = clock_ticks::precise_time_ms();
        self.dt = args.ext_dt as f32 * 1000000000.0;
        self.u = 1.0 / (1.0 / self.updates_per_second as f32) * self.dt;
        self.width = args.draw_width as f32;
        self.height = args.draw_height as f32;
        self.window.make_current();
        self.stencil_mode = StencilMode::None;
        self.context = Context::new_viewport(args.viewport());
    }

    #[inline]
    pub fn context(&self) -> &Context {
        &self.context
    }

    #[inline]
    pub fn t(&self) -> u64 {
        self.t
    }

    #[inline]
    pub fn dt(&self) -> f32 {
        self.dt
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
        self.flush();
    }

    pub fn finalize(&mut self) {

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
    pub fn set_texture(&mut self, texture: Texture) {
        self.flush();
        let (typ, arg, _) = texture.into_tuple();
        self.texture = [typ as f32, arg, 0.0, 0.0];
    }

    pub fn set_color(&mut self, color: [f32; 4]) {
        self.flush();
        self.color = gamma_srgb_to_linear(color);
    }

    pub fn clear_color(&mut self, color: [f32; 4]) {
        self.encoder.clear(&self.output_color, gamma_srgb_to_linear(color));
    }

    pub fn set_stencil_mode(&mut self, mode: StencilMode) {
        // Stencil changes need to flush the render buffer
        self.flush();
        self.stencil_mode = mode;
    }

    pub fn clear_stencil(&mut self, value: u8) {
        self.encoder.clear_stencil(&self.output_stencil, value);
    }


    // Direct Shape Drawing ---------------------------------------------------
    pub fn polygon(&mut self, context: &Context, vertices: &[f32]) {
        self.draw_triangle_list(&context.transform, &vertices);
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

    pub fn add_particle(
        &mut self,
        scale: f32, index: usize,
        x: f32, y: f32, size: f32,
        color: &[f32; 4]
    ) {

        let v = index * 2;
        self.particle_position[v] = x;
        self.particle_position[v + 1] = y;
        self.particle_scale[v] = size * scale;

        self.particle_color[index] = gamma_srgb_to_linear(*color);

    }

    // Particle Drawing -------------------------------------------------------
    pub fn render_particles(&mut self, m: &Matrix2d, count: usize) {

        self.pre_draw(gfx::Primitive::PointList, m, count);

        {
            use std::slice::from_raw_parts;

            unsafe {
                self.encoder.update_buffer(
                    &self.buffer_pos,
                    from_raw_parts(
                        self.particle_position.as_ptr() as *const PositionFormat,
                        count * 2
                    ),
                    self.buffer_offset

                ).unwrap();
            }

            unsafe {
                self.encoder.update_buffer(
                    &self.buffer_scale,
                    from_raw_parts(
                        self.particle_scale.as_ptr() as *const ScaleFormat,
                        count * 2
                    ),
                    self.buffer_offset

                ).unwrap();
            }

            unsafe {
                self.encoder.update_buffer(
                    &self.buffer_color,
                    from_raw_parts(
                        self.particle_color.as_ptr() as *const ColorFormat,
                        count
                    ),
                    self.buffer_offset

                ).unwrap();
            }

            self.buffer_offset += count;

        }

    }

    // Internal ---------------------------------------------------------------
    fn draw_triangle_list(&mut self, m: &Matrix2d, vertices: &[f32]) {
        self.draw(gfx::Primitive::TriangleList, m, vertices);
    }

    fn draw(&mut self, primitive: gfx::Primitive, m: &Matrix2d, vertices: &[f32]) {

        let n = vertices.len() / POS_COMPONENTS;
        self.pre_draw(primitive, m, n);

        {
            use std::slice::from_raw_parts;

            unsafe {
                self.encoder.update_buffer(
                    &self.buffer_pos,
                    from_raw_parts(
                        vertices.as_ptr() as *const PositionFormat,
                        n
                    ),
                    self.buffer_offset

                ).unwrap();
            }

            self.buffer_offset += n;

        }

    }

    fn pre_draw(&mut self, primitive: gfx::Primitive, m: &Matrix2d, n: usize) {

        let view_matrix = [
            // Rotation
            [m[0][0] as f32, m[1][0] as f32, 0.0, 0.0],
            [m[0][1] as f32, m[1][1] as f32, 0.0, 0.0],

            // Identity
            [0.0, 0.0, 1.0, 0.0],

            // Translation
            [m[0][2] as f32, m[1][2] as f32, 0.0, 1.0]

        ];

        // Flush buffer if rendering primitive or view matrix changes or if
        // the vertices would overflow the buffer
        if self.primitive != primitive || self.buffer_matrix != view_matrix || self.buffer_offset + n > BUFFER_SIZE * CHUNKS {
            self.flush();
            self.primitive = primitive;
            self.buffer_matrix = view_matrix;
        }

    }

    fn flush(&mut self) {

        if self.buffer_offset > 0 {

            self.encoder.update_constant_buffer(&self.buffer_locals, &Locals {
                view: self.buffer_matrix,
                color: self.color,
            });

            self.encoder.update_constant_buffer(&self.buffer_texture, &TextureLocals {
                texture: self.texture
            });

            let (pso_colored, stencil_val) = if self.primitive == gfx::Primitive::TriangleList {
                if self.wireframe {
                    self.wire_pipeline.get(self.stencil_mode)

                } else {
                    self.list_pipeline.get(self.stencil_mode)
                }

            } else {
                self.point_pipeline.get(self.stencil_mode)
            };

            let data = pipe_colored::Data {
                pos: self.buffer_pos.clone(),
                scale: self.buffer_scale.clone(),
                color: self.buffer_color.clone(),
                locals: self.buffer_locals.clone(),
                texture: self.buffer_texture.clone(),
                blend_target: self.output_color.clone(),
                stencil_target: (
                    self.output_stencil.clone(),
                    (stencil_val, stencil_val)
                ),
                blend_ref: [1.0; 4]
            };

            let slice = gfx::Slice {
                instances: None,
                start: 0,
                end: self.buffer_offset as u32,
                buffer: gfx::IndexBuffer::Auto,
                base_vertex: 0,
            };

            self.encoder.draw(&slice, pso_colored, &data);
            self.buffer_offset = 0;

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


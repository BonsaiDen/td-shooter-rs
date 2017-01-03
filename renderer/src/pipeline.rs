// External Dependencies ------------------------------------------------------
use gfx;
use gfx_device_gl;
use gfx::pso::PipelineState;
use shader_version::{OpenGL, Shaders};
use shader_version::glsl::GLSL;


// Internal Dependencies ------------------------------------------------------
use ::data::*;


// Stencil Mode ---------------------------------------------------------------
#[derive(Debug, Copy, Clone)]
pub enum StencilMode {
    None,
    Add,
    Replace(u8),
    ClearLightCones,
    ReplaceNonLightCircle,
    InsideLightCircle,
    OutsideVisibleArea
}


// Rendering Pipeline with Stencil Modes --------------------------------------
pub struct RenderPipeline<T> {
    none: T,
    add: T,
    clear_light_cones: T,
    replace: T,
    replace_non_light: T,
    inside_visible: T,
    outside_visible: T,
}

impl<T> RenderPipeline<T> {

    pub fn new<F>(
        factory: &mut gfx_device_gl::Factory,
        f: F

    ) -> RenderPipeline<T> where F: Fn(
        &mut gfx_device_gl::Factory,
        gfx::state::Blend,
        gfx::state::Stencil,
        gfx::state::ColorMask

    ) -> T {

        use gfx::preset::blend;
        use gfx::state::{Comparison, Stencil, StencilOp};

        RenderPipeline {

            none: f(factory, blend::ALPHA, Stencil::new(
                Comparison::Always,
                0,
                (StencilOp::Keep, StencilOp::Keep, StencilOp::Keep)

            ), gfx::state::MASK_ALL),

            // Adds 1 to the stencil buffer clamping at 255
            add: f(factory, blend::ALPHA, Stencil::new(
                Comparison::Never,
                255,
                (StencilOp::IncrementClamp, StencilOp::Keep, StencilOp::Keep)

            ), gfx::state::MASK_NONE),

            // Always replaces the stencil buffer with a specified value
            replace: f(factory, blend::ALPHA, Stencil::new(
                Comparison::Never,
                255,
                (StencilOp::Replace, StencilOp::Keep, StencilOp::Keep)

            ), gfx::state::MASK_NONE),

            // Replaces all non-255 values in the stencil buffer with 254
            replace_non_light: f(factory, blend::ALPHA, Stencil::new(
                Comparison::Equal,
                254,
                (StencilOp::Replace, StencilOp::Keep, StencilOp::Keep)

            ), gfx::state::MASK_NONE),

            // Clears all remaining values of 254 in the stencil buffer to 0
            clear_light_cones: f(factory, blend::ALPHA, Stencil::new(
                Comparison::Equal,
                255,
                (StencilOp::Zero, StencilOp::Keep, StencilOp::Keep)

            ), gfx::state::MASK_NONE),

            // Only renders where the stencil buffer is 255
            inside_visible: f(factory, blend::ALPHA, Stencil::new(
                Comparison::Equal,
                255,
                (StencilOp::Keep, StencilOp::Keep, StencilOp::Keep)

            ), gfx::state::MASK_ALL),

            // Renders everywhere the stencil buffer ist NOT 255
            outside_visible: f(factory, blend::ALPHA, Stencil::new(
                Comparison::NotEqual,
                255,
                (StencilOp::Keep, StencilOp::Keep, StencilOp::Keep)

            ), gfx::state::MASK_ALL)

        }

    }

    pub fn get(&mut self, mode: StencilMode) -> (&mut T, u8) {
        match mode {
            StencilMode::None => (&mut self.none, 0),
            StencilMode::Add => (&mut self.add, 1),
            StencilMode::Replace(v) => (&mut self.replace, v),
            StencilMode::ClearLightCones => (&mut self.clear_light_cones, 255),
            StencilMode::ReplaceNonLightCircle => (&mut self.replace_non_light, 254),
            StencilMode::InsideLightCircle => (&mut self.inside_visible, 255),
            StencilMode::OutsideVisibleArea => (&mut self.outside_visible, 255)
        }
    }


    // Statics ----------------------------------------------------------------
    pub fn create(
        opengl: OpenGL,
        factory: &mut gfx_device_gl::Factory,
        primitive: gfx::Primitive,
        method: gfx::state::RasterMethod

    ) -> RenderPipeline<gfx::PipelineState<gfx_device_gl::Resources, pipe_colored::Meta>> {

        use gfx::state::{Blend, Stencil, Rasterizer, MultiSample, CullFace};
        use gfx::traits::*;
        use shaders_graphics2d::colored;

        let glsl = opengl.to_glsl();

        let shader_program = if primitive == gfx::Primitive::TriangleList {
            factory.link_program(
                Shaders::new()
                    .set(GLSL::V1_20, TRIANGLE_VERTEX_SHADER_120)
                    .set(GLSL::V1_50, TRIANGLE_VERTEX_SHADER_150)
                    .get(glsl).unwrap(),

                Shaders::new()
                    .set(GLSL::V1_20, colored::FRAGMENT_GLSL_120)
                    .set(GLSL::V1_50, colored::FRAGMENT_GLSL_150_CORE)
                    .get(glsl).unwrap(),

            ).unwrap()

        } else {
            factory.link_program(
                Shaders::new()
                    .set(GLSL::V1_20, POINT_VERTEX_SHADER_120)
                    .set(GLSL::V1_50, POINT_VERTEX_SHADER_150)
                    .get(glsl).unwrap(),

                Shaders::new()
                    .set(GLSL::V1_20, colored::FRAGMENT_GLSL_120)
                    .set(GLSL::V1_50, colored::FRAGMENT_GLSL_150_CORE)
                    .get(glsl).unwrap(),

            ).unwrap()
        };

        let polygon_pipeline = |
            factory: &mut gfx_device_gl::Factory,
            blend_preset: Blend,
            stencil: Stencil,
            color_mask: gfx::state::ColorMask,

        | -> PipelineState<gfx_device_gl::Resources, pipe_colored::Meta> {

            let mut r = Rasterizer::new_fill();
            r.cull_face = CullFace::Front;
            r.method = method;
            r.samples = Some(MultiSample);

            factory.create_pipeline_from_program(
                &shader_program,
                primitive,
                r,
                pipe_colored::Init {
                    pos: (),
                    scale: (),
                    locals: "Locals",
                    color: (),
                    blend_target: ("o_Color", color_mask, blend_preset),
                    stencil_target: stencil,
                    blend_ref: ()
                }

            ).unwrap()

        };

        RenderPipeline::new(factory, polygon_pipeline)

    }

}


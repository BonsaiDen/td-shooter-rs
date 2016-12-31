// GFX Dependencies -----------------------------------------------------------
use gfx;
use gfx_device_gl;


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


// Stencil Pipeline -----------------------------------------------------------
pub struct ColoredStencil<T> {
    none: T,
    add: T,
    clear_light_cones: T,
    replace: T,
    replace_non_light: T,
    inside_visible: T,
    outside_visible: T,
}

impl<T> ColoredStencil<T> {

    pub fn new<F>(factory: &mut gfx_device_gl::Factory, f: F) -> ColoredStencil<T> where F: Fn(
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

}


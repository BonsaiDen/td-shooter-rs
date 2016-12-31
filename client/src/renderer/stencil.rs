// GFX Dependencies -----------------------------------------------------------
use gfx;
use gfx_device_gl;


// Stencil Mode ---------------------------------------------------------------
#[derive(Debug, Copy, Clone)]
pub enum StencilMode {
    None,
    Add(u8),
    Replace(u8),
    Inside(u8),
    Outside(u8)
}


// Stencil Pipeline -----------------------------------------------------------
pub struct ColoredStencil<T> {
    none: T,
    add: T,
    replace: T,
    inside: T,
    outside: T
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

    pub fn get(&mut self, mode: StencilMode) -> (&mut T, u8) {
        match mode {
            StencilMode::None => (&mut self.none, 0),
            StencilMode::Add(v) => (&mut self.add, v),
            StencilMode::Replace(v) => (&mut self.replace, v),
            StencilMode::Inside(v) => (&mut self.inside, v),
            StencilMode::Outside(v) => (&mut self.outside, v)
        }
    }

}


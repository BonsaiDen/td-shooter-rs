// Modules --------------------------------------------------------------------
pub mod laser_beam;


// Re-Exports -----------------------------------------------------------------
pub use self::laser_beam::LaserBeam;


// Internal Dependencies ------------------------------------------------------
use ::renderer::Renderer;
use ::camera::Camera;


// Effect Trait ---------------------------------------------------------------
pub trait Effect {
    fn alive(&self, u64) -> bool;
    fn render(&self, &mut Renderer, &Camera);
}


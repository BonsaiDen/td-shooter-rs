// Modules --------------------------------------------------------------------
mod laser_beam;
mod laser_beam_hit;


// Re-Exports -----------------------------------------------------------------
pub use self::laser_beam::LaserBeam;
pub use self::laser_beam_hit::LaserBeamHit;


// Internal Dependencies ------------------------------------------------------
use ::renderer::Renderer;
use ::camera::Camera;


// Effect Trait ---------------------------------------------------------------
pub trait Effect {
    fn alive(&self, u64) -> bool;
    fn render(&self, &mut Renderer, &Camera);
}


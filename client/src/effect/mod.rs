// Modules --------------------------------------------------------------------
mod particle;
mod laser_beam;
mod laser_beam_hit;
mod particle_system;
mod screen_flash;


// Re-Exports -----------------------------------------------------------------
pub use self::laser_beam::LaserBeam;
pub use self::laser_beam_hit::LaserBeamHit;
pub use self::screen_flash::ScreenFlash;
pub use self::particle_system::*;
pub use self::particle::*;


// Internal Dependencies ------------------------------------------------------
use ::renderer::Renderer;
use ::camera::Camera;


// Effect Trait ---------------------------------------------------------------
pub trait Effect {
    fn alive(&self, u64) -> bool;
    fn render(&self, &mut Renderer, &Camera);
}


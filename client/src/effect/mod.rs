// External Dependencies ------------------------------------------------------
use piston_window::{Context, G2d};


// Modules --------------------------------------------------------------------
pub mod laser;


// Re-Exports -----------------------------------------------------------------
pub use self::laser::LaserBeam;


// Effect Trait ---------------------------------------------------------------
pub trait Effect {
    fn alive(&self, t: u64) -> bool;
    fn draw_2d(&self, Context, &mut G2d, t: u64);
}


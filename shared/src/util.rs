// STD Dependencies -----------------------------------------------------------
use std::f32::consts;



// Conversion Utilities -------------------------------------------------------
pub fn rad_to_u16(r: f32) -> u16 {
    ((r + consts::PI) * 2000.0).round() as u16
}

pub fn u16_to_rad(r: u16) -> f32 {
    (r as f32) / 2000.0 - consts::PI
}


// STD Dependencies -----------------------------------------------------------
use std::f32::consts;


// Conversion Utilities -------------------------------------------------------
pub fn rad_to_u16(r: f32) -> u16 {
    ((r + consts::PI) * 2000.0).round() as u16
}

pub fn u16_to_rad(r: u16) -> f32 {
    (r as f32) / 2000.0 - consts::PI
}

pub fn distance(x: f32, y: f32, ox: f32, oy: f32) -> f32 {
    let (dx, dy) = (x - ox, y - oy);
    (dx * dx + dy * dy).sqrt()
}

pub fn angle_within_cone(
    x: f32, y: f32, r: f32,
    ox: f32, oy: f32,
    offset: f32,
    cone: f32

) -> bool {

    let (cx, cy) = (x - r.cos() * offset, y - r.sin() * offset);
    let (dx, dy) = (ox - cx, oy - cy);
    let or = dy.atan2(dx);

    let dr = r - or;
    dr.sin().atan2(dr.cos()).abs() < cone

}


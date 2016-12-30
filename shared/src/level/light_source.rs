// STD Dependencies -----------------------------------------------------------
use std::f64::consts;
use std::cmp::Ordering;


// Internal Dependencies ------------------------------------------------------
use ::level::{Level, LevelWall, LevelVisibility, LevelCollision};


// Light Source ---------------------------------------------------------------
#[derive(Debug)]
pub struct LightSource {
    pub x: f64,
    pub y: f64,
    pub radius: f64,
    pub aabb: [f64; 4]
}

impl LightSource {

    pub fn new(x: f64, y: f64, radius: f64) -> LightSource {
        LightSource {
            x: x,
            y: y,
            radius: radius,
            aabb: [x - radius, y - radius, x + radius, y + radius]
        }
    }

    pub fn circle_intersect(&self, x: f64, y: f64, radius: f64) -> bool {
        // TODO also perform a collide_line to avoid issues with walls
        let (dx, dy) = (self.x - x, self.y - y);
        let d = (dx * dx + dy * dy).sqrt();
        d < self.radius + radius
    }

}

// Helpers --------------------------------------------------------------------
fn to_point(px: f64, py: f64, ox: f64, oy: f64, wall_index: isize) -> (f64, f64, [f64; 2], isize) {
    let (dx, dy) = (px - ox, py - oy);
    let l = (dx * dx + dy * dy).sqrt();
    let r = dy.atan2(dx);
    (r, l, [px, py], wall_index)
}

fn to_limited_point(px: f64, py: f64, ox: f64, oy: f64, radius: f64) -> [f64; 2] {
    let (dx, dy) = (px - ox, py - oy);
    let l = (dx * dx + dy * dy).sqrt();
    let r = dy.atan2(dx);
    [ox + r.cos() * l.min(radius), oy + r.sin() * l.min(radius)]
}

fn attach_to_wall(p: [f64; 2], wall: &LevelWall) -> [f64; 2] {
    let d = wall.distance_from_point(p[0], p[1]);
    if d > 2.0 && d < 10.0 {
        if wall.is_vertical {
            [wall.points[0], p[1]]

        } else if wall.is_horizontal {
            [p[0], wall.points[1]]

        } else {
            p
        }

    } else {
        p
    }
}


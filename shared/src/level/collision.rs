// STD Dependencies -----------------------------------------------------------
use std::f32::consts;
use std::collections::HashSet;


// Internal Dependencies ------------------------------------------------------
use ::entity::PLAYER_RADIUS;
use super::{Level, MAX_LEVEL_SIZE};


// Statics --------------------------------------------------------------------
pub const COLLISION_GRID_SPACING: f32 = 50.0;


// Traits ---------------------------------------------------------------------
pub trait LevelCollision {
    fn collision_bounds(&self, x: f32, y: f32) -> [f32; 4];
    fn collide(&self, x: &mut f32, y: &mut f32, radius: f32, active: bool);
    fn collide_beam(&self, x: f32, y: f32, r: f32, l: f32) -> Option<(usize, [f32; 3])>;
    fn collide_beam_wall(&self, x: f32, y: f32, r: f32, l: f32) -> Option<f32>;
    fn collide_line(&self, line: &[f32; 4]) -> Option<(usize, [f32; 3])>;
}

impl LevelCollision for Level {

    fn collision_bounds(&self, x: f32, y: f32) -> [f32; 4] {
        let (gx, gy) = self.w2g(x, y);
        [
            (gx as f32) * COLLISION_GRID_SPACING,
            (gy as f32) * COLLISION_GRID_SPACING,
            COLLISION_GRID_SPACING,
            COLLISION_GRID_SPACING
        ]
    }

    fn collide(&self, x: &mut f32, y: &mut f32, radius: f32, active: bool) {

        let walls = self.get_walls_in_bounds(&[
            *x - radius,
            *y - radius,
            *x + radius,
            *y + radius
        ]);

        let mut iterations = 0;
        let mut collisions = 1;
        while collisions > 0 && iterations < 10 {

            collisions = 0;

            let mut overlap = (0.0, 0.0);
            for i in &walls {

                let wall = &self.walls[*i];

                if aabb_intersect_circle(
                    &wall.aabb,
                    *x,
                    *y,
                    radius + 1.0
                ) {
                    if let Some(collision) = line_intersect_circle(
                        &wall.collision,
                        *x,
                        *y,
                        radius + 1.0
                    ) {
                        overlap.0 += collision[7].cos() * collision[6];
                        overlap.1 += collision[7].sin() * collision[6];
                        collisions += 1;
                    }
                }

            }

            // Avoid edge sliding without player input
            if active == false && overlap.0.abs() < 0.1 && overlap.1.abs() < 0.1 {
                break;
            }

            *x -= overlap.0;
            *y -= overlap.1;

            iterations += 1;

            // No need to iterate idle entities multiple times per frame
            if active == false {
                break;
            }

        }

        *x = x.min(MAX_LEVEL_SIZE).max(-MAX_LEVEL_SIZE);
        *y = y.min(MAX_LEVEL_SIZE).max(-MAX_LEVEL_SIZE);

    }

    fn collide_beam(&self, x: f32, y: f32, r: f32, l: f32) -> Option<(usize, [f32; 3])> {

        let line = [
            x,
            y,
            x + r.cos() * l,
            y + r.sin() * l
        ];

        self.collide_line(&line)

    }

    fn collide_beam_wall(&self, x: f32, y: f32, r: f32, l: f32) -> Option<f32> {

        let line = [
            x,
            y,
            x + r.cos() * l,
            y + r.sin() * l
        ];

        // Return wall angle
        if let Some(intersect) = self.collide_line(&line) {

            let wall = &self.walls[intersect.0];

            // Vertical |
            if wall.is_vertical {
                // Left or right of the wall
                if x > wall.points[0] {
                    Some(consts::PI)

                } else {
                    Some(0.0)
                }

            // Horizontal --
            } else if wall.is_horizontal {
                // Above or below the wall
                if y > wall.points[1] {
                    Some(-consts::PI * 0.5)

                } else {
                    Some(consts::PI * 0.5)
                }

            // Diagonal \
            } else if wall.points[0] < wall.points[2] && wall.points[1] < wall.points[3] {
                if r > consts::PI * 0.35 && r < consts::PI * 1.25 {
                    Some(consts::PI * 0.75)

                } else {
                    Some(consts::PI * 1.75)
                }

            // Diagonal /
            } else if r > consts::PI * 0.75 && r < consts::PI * 1.75 {
                Some(consts::PI * 1.25)

            } else {
                Some(consts::PI * 0.25)
            }

        } else {
            None
        }

    }

    fn collide_line(&self, line: &[f32; 4]) -> Option<(usize, [f32; 3])> {
        self.collide_beam_with_walls(&line, &self.get_walls_in_bounds(&line))
    }

}

// Internal Helpers -----------------------------------------------------------
impl Level {

    pub fn w2g(&self, x: f32, y: f32) -> (isize, isize) {
        let gx = ((x - COLLISION_GRID_SPACING * 0.5) / COLLISION_GRID_SPACING).round();
        let gy = ((y - COLLISION_GRID_SPACING * 0.5) / COLLISION_GRID_SPACING).round();
        (gx as isize, gy as isize)
    }

    fn collide_beam_with_walls(&self, line: &[f32; 4], walls: &HashSet<usize>) -> Option<(usize, [f32; 3])> {

        let mut intersection: Option<(usize, [f32; 3])> = None;
        for i in walls {

            let wall = &self.walls[*i];
            if let Some(new) = line_intersect_line(&line, &wall.points) {

                let is_closer = if let Some(existing) = intersection {
                    new[2] < existing.1[2]

                } else {
                    true
                };

                if is_closer {
                    intersection = Some((*i, new));
                }

            }
        }

        intersection

    }

}


// Collision Helpers ----------------------------------------------------------
pub fn aabb_intersect_circle(aabb: &[f32; 4], x: f32, y: f32, r: f32) -> bool {

    let px = if x > aabb[2] {
        aabb[2]

    } else if x < aabb[0] {
        aabb[0]

    } else {
        x
    };

    let py = if y > aabb[3] {
        aabb[3]

    } else if y < aabb[1] {
        aabb[1]

    } else {
        y
    };

    let dx = px - x;
    let dy = py - y;
    (dx * dx + dy * dy).sqrt() < r

}

pub fn line_intersect_circle(line: &[f32; 4], cx: f32, cy: f32, r: f32) -> Option<[f32; 8]> {

    let (ax, ay) = (line[0], line[1]);
    let (bx, by) = (line[2], line[3]);

    // compute the euclidean distance between A and B
    let lab = ((bx - ax).powf(2.0) + (by - ay).powf(2.0)).sqrt();

    // compute the direction vector D from A to B
    let (dx, dy) = ((bx - ax) / lab, (by - ay) / lab);

    // compute the value t of the closest point to the circle center (Cx, Cy)
    let t = dx * (cx - ax) + dy * (cy - ay);

    // compute the coordinates of the point E on line and closest to C
    let (ex, ey) = (t * dx + ax, t * dy + ay);

    // compute the euclidean distance from E to C
    let lec = ((ex - cx).powf(2.0) + (ey - cy).powf(2.0)).sqrt();

    // test if the line intersects the circle
    if lec < r {

        // compute distance from t to circle intersection point
        let dt = (r * r - lec * lec).sqrt();

        // compute first intersection point
        let (fx, fy) = ((t - dt).max(0.0) * dx + ax, (t - dt).max(0.0) * dy + ay);

        // compute second intersection point
        let (gx, gy) = ((t + dt).min(lab) * dx + ax, (t + dt).min(lab) * dy + ay);

        // projected end of intersection line
        let (hx, hy) = (fx + (gx - fx) * 0.5, fy + (gy - fy) * 0.5);

        // Overlap
        let (ox, oy) = (hx - cx, hy - cy);
        let o = r - (ox * ox + oy * oy).sqrt();

        Some([fx, fy, gx, gy, hx, hy, o, oy.atan2(ox)])

    } else {
        None
    }

}

pub fn line_segment_intersect_circle(line: &[f32; 4], cx: f32, cy: f32, r: f32) -> Option<[f32; 8]> {

    let (ax, ay) = (line[0], line[1]);
    let (bx, by) = (line[2], line[3]);
    let (dx, dy) = (bx - ax, by - ay);

    let a = dx * dx + dy * dy;
    let b = 2.0 * (dx * (ax - cx) + dy * (ay - cy));

    let c = (ax - cx) * (ax - cx) + (ay - cy) * (ay - cy) - r * r;
    let det = b * b - 4.0 * a * c;

    if det >= 0.0 {

        // compute first intersection point
        let t = (-b + det.sqrt()) / (2.0 * a);

        let (fx, fy) = (ax + t * dx, ay + t * dy);

        // compute second intersection point
        let t = (-b - det.sqrt()) / (2.0 * a);
        let (gx, gy) = (ax + t * dx, ay + t * dy);

        // projected end of intersection line
        let (hx, hy) = (fx + (gx - fx) * 0.5, fy + (gy - fy) * 0.5);

        // Overlap
        let (ox, oy) = (hx - cx, hy - cy);
        let o = r - (ox * ox + oy * oy).sqrt();

        Some([fx, fy, gx, gy, hx, hy, o, oy.atan2(ox)])

    } else {
        None
    }

}

pub fn line_segment_intersect_circle_test(line: &[f32; 4], cx: f32, cy: f32, r: f32) -> bool {

    let (ax, ay) = (line[0], line[1]);
    let (bx, by) = (line[2], line[3]);
    let (dx, dy) = (bx - ax, by - ay);

    let a = dx * dx + dy * dy;
    let b = 2.0 * (dx * (ax - cx) + dy * (ay - cy));

    let c = (ax - cx) * (ax - cx) + (ay - cy) * (ay - cy) - r * r;
    let det = b * b - 4.0 * a * c;

    if det > 0.0 {
        let t = -b / (2.0 * a);
        t >= 0.0 && t <= 1.0

    } else {
        false
    }

}

pub fn line_intersect_line(line: &[f32; 4], other: &[f32; 4]) -> Option<[f32; 3]> {

    let (ax, ay) = ( line[2] -  line[0],  line[3] -  line[1]);
    let (bx, by) = (other[2] - other[0], other[3] - other[1]);
    let (cx, cy) = ( line[0] - other[0],  line[1] - other[1]);

    let d = ax * by - bx * ay;
    if d != 0.0 {

        let s = ax * cy - cx * ay;
        if s <= 0.0 && d < 0.0 && s >= d || s >= 0.0 && d > 0.0 && s <= d {

            let t = bx * cy - cx * by;
            if t <= 0.0 && d < 0.0 && t > d || t >= 0.0 && d > 0.0 && t < d {

                let t = t / d;
                let dx = line[0] + t * ax;
                let dy = line[1] + t * ay;
                let (ex, ey) = (line[0] - dx, line[1] - dy);

                return Some([dx, dy, (ex * ex + ey * ey).sqrt()]);

            }

        }

    }

    None

}


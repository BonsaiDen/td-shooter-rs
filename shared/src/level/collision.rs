// STD Dependencies -----------------------------------------------------------
use std::collections::HashSet;


// Statics --------------------------------------------------------------------
pub const COLLISION_GRID_SPACING: f32 = 100.0;


// Internal Dependencies ------------------------------------------------------
use super::{Level, MAX_LEVEL_SIZE};


// Traits ---------------------------------------------------------------------
pub trait LevelCollision {
    fn collision_bounds(&self, x: f32, y: f32) -> [f32; 4];
    fn collide(&self, x: &mut f32, y: &mut f32, radius: f32);
    fn collide_beam(&self, x: f32, y: f32, r: f32, l: f32) -> Option<[f32; 5]>;
    fn collide_line(&self, line: &[f32; 4]) -> Option<[f32; 5]>;
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

    fn collide(&self, x: &mut f32, y: &mut f32, radius: f32) {

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

            *x -= overlap.0;
            *y -= overlap.1;

            iterations += 1;

        }

        *x = x.min(MAX_LEVEL_SIZE).max(-MAX_LEVEL_SIZE);
        *y = y.min(MAX_LEVEL_SIZE).max(-MAX_LEVEL_SIZE);

    }

    fn collide_beam(&self, x: f32, y: f32, r: f32, l: f32) -> Option<[f32; 5]> {

        let line = [
            x,
            y,
            x + r.cos() * l,
            y + r.sin() * l
        ];

        self.collide_line(&line)

    }

    fn collide_line(&self, line: &[f32; 4]) -> Option<[f32; 5]> {
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

    fn collide_beam_with_walls(&self, line: &[f32; 4], walls: &HashSet<usize>) -> Option<[f32; 5]> {

        let mut intersection: Option<[f32; 5]> = None;
        for i in walls {

            let wall = &self.walls[*i];
            if let Some(new) = line_intersect_line(&line, &wall.points) {

                let is_closer = if let Some(existing) = intersection {
                    new[4] < existing[4]

                } else {
                    true
                };

                if is_closer {
                    intersection = Some(new);
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

    // Now the line equation is x = Dx*t + Ax, y = Dy*t + Ay with 0 <= t <= 1.

    // compute the value t of the closest point to the circle center (Cx, Cy)
    let t = dx * (cx - ax) + dy * (cy - ay);

    // This is the projection of C on the line from A to B.

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

pub fn line_intersect_line(line: &[f32; 4], other: &[f32; 4]) -> Option<[f32; 5]> {

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

                return Some([line[0], line[1], dx, dy, (ex * ex + ey * ey).sqrt()]);

            }

        }

    }

    None

}


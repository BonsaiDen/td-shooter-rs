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

    pub fn generate_vertices(&self, level: &Level) -> Vec<[f64; 2]> {

        // TODO clean up
        let mut points = Vec::new();
        for (wall_index, a, b) in level.calculate_visibility(self.x, self.y) {

            let a = to_point(a.0, a.1, self.x, self.y, wall_index as isize);
            let b = to_point(b.0, b.1, self.x, self.y, wall_index as isize);
            if a.1 > 0.0 {
                points.push(a);
            }

            if b.1 > 0.0 {
                points.push(b);
            }

        }

        // Sub-divide existing visibility polygon
        let mut divided = true;
        while divided {
            divided = false;

            let mut i = 0;
            let step = (consts::PI * 2.0) / 256.0;
            let mut extra_points = vec![];
            while i < points.len() {

                let a = points[i];
                let b = points[(i + 1) % points.len()];

                let mut dr = b.0 - a.0;
                if dr <= -consts::PI {
                    dr += 2.0 * consts::PI;
                }

                if dr > consts::PI {
                    dr -= 2.0 * consts::PI;
                }

                if dr > step  {

                    let ir = a.0 + dr / 2.0;
                    let (mut ix, mut iy) = (
                        self.x + ir.cos() * self.radius,
                        self.y + ir.sin() * self.radius
                    );

                    let mut l = self.radius;
                    if let Some(intersection) = level.collide_line(&[self.x, self.y, ix, iy]) {
                        ix = intersection[2];
                        iy = intersection[3];
                        l = intersection[4];
                    }

                    extra_points.push((ir, l, [ix, iy], -1));
                    divided = true;

                }

                i += 1;

            }

            points.append(&mut extra_points);

            points.sort_by(|a, b| {
                if a.0 > b.0 {
                    Ordering::Greater

                } else if a.0 < b.0 {
                    Ordering::Less

                } else {
                    Ordering::Equal
                }
            });

        }

        // Simplify divided polygon
        let mut filtered_points = vec![];
        let mut count = 0;
        let mut i = 0;
        while i < points.len() {

            //let a = points[i];
            let b = points[(i + 1) % points.len()];

            if count == 0 {
                filtered_points.push(b);
                count = 8;
            }

            count -= 1;
            i += 1;

        }

        println!("Light with {} vertices", filtered_points.len());

        // Triangulate the polygon
        let mut i = 0;
        let mut vertices = Vec::new();
        while i < filtered_points.len() {
            let a = filtered_points[i];
            let b = filtered_points[(i + 1) % filtered_points.len()];
            vertices.push([self.x, self.y]);

            let mut pa = to_limited_point(a.2[0], a.2[1], self.x, self.y, self.radius);
            if a.3 != -1 {
                pa = attach_to_wall(pa, &level.walls[a.3 as usize]);
            }
            vertices.push(pa);

            let mut pb = to_limited_point(b.2[0], b.2[1], self.x, self.y, self.radius);
            if b.3 != -1 {
                pb = attach_to_wall(pb, &level.walls[b.3 as usize]);
            }
            vertices.push(pb);


            i += 1;
        }

        vertices

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


// STD Dependencies -----------------------------------------------------------
use std::collections::{HashMap, HashSet};


// Statics --------------------------------------------------------------------
const GRID_SPACING: f64 = 100.0;


// Level Abstraction ----------------------------------------------------------
pub trait LevelCollision {
    fn collide(&self, x: &mut f32, y: &mut f32, radius: f64);
}

#[derive(Debug, Default)]
pub struct Level {
    pub walls: Vec<LevelWall>,
    grid: HashMap<(isize, isize), Vec<usize>>
}

impl Level {

    pub fn new() -> Level {
        Level {
            // TODO add a grid around the origin (-n to +n) for spatial lookup
            // of walls for both rendering and collision detection
            walls: Vec::new(),
            grid: HashMap::new()
        }
    }

    pub fn add_wall(&mut self, wall: LevelWall) {
        self.store_wall_grid(&wall.aabb);
        self.walls.push(wall);
    }

    pub fn load() -> Level {
        let mut level = Level::new();
        level.add_wall(LevelWall::new(100.0, 100.0, -100.0, 100.0));
        level.add_wall(LevelWall::new(-100.0, -100.0, -100.0, 100.0));
        level.add_wall(LevelWall::new(0.0, 0.0, 100.0, -100.0));
        level.add_wall(LevelWall::new(0.0, 0.0, 100.0, 100.0));
        level
    }

    fn store_wall_grid(&mut self, aabb: &[f64; 4]) {

        let (top_left, bottom_right) = (
            self.grid_cell(aabb[0], aabb[1]),
            self.grid_cell(aabb[2], aabb[3])
        );

        for y in top_left.1..bottom_right.1 + 1{
            for x in top_left.0..bottom_right.0 + 1{
                self.grid.entry((x, y)).or_insert_with(Vec::new).push(self.walls.len());
            }
        }

    }

    fn grid_cell(&self, x: f64, y: f64) -> (isize, isize) {
        let gx = (x / GRID_SPACING).round();
        let gy = (y / GRID_SPACING).round();
        (gx as isize, gy as isize)
    }

}

impl LevelCollision for Level {

    fn collide(&self, x: &mut f32, y: &mut f32, radius: f64) {

        let (top_left, bottom_right) = (
            self.grid_cell(*x as f64 + radius, *y as f64 + radius),
            self.grid_cell(*x as f64 - radius, *y as f64 - radius)
        );

        let mut walls = HashSet::new();
        for y in (top_left.1 - 1)..bottom_right.1 + 1{
            for x in (top_left.0 - 1)..bottom_right.0 + 1{
                if let Some(indicies) = self.grid.get(&(x, y)) {
                    for i in indicies {
                        walls.insert(*i);
                    }
                }
            }
        }

        let mut iterations = 0;
        let mut collisions = 1;
        while collisions > 0 && iterations < 10 {

            collisions = 0;

            let mut overlap = (0.0, 0.0);
            for i in &walls {

                let wall = &self.walls[*i];

                if aabb_intersect_circle(
                    &wall.aabb,
                    *x as f64,
                    *y as f64,
                    radius + 1.0
                ) {
                    if let Some(collision) = line_intersect_circle(
                        &wall.collision,
                        *x as f64,
                        *y as f64,
                        radius + 1.0
                    ) {

                        overlap.0 += (collision[7].cos() * collision[6]) as f32;
                        overlap.1 += (collision[7].sin() * collision[6]) as f32;

                        collisions += 1;

                    }
                }

            }

            *x -= overlap.0;
            *y -= overlap.1;

            iterations += 1;

        }

    }

}

#[derive(Debug)]
pub struct LevelWall {
    pub points: [f64; 4],
    pub collision: [f64; 4],
    pub aabb: [f64; 4]
}

impl LevelWall {

    pub fn new(a: f64, b: f64, c: f64, d: f64) -> LevelWall {

        // Shorten edges for less collision glitches
        let (dx, dy) = (a - c, b - d);
        let l = (dx * dx + dy * dy).sqrt();
        let r = dy.atan2(dx);

        let (cx, cy) = (a - r.cos() * l * 0.5, b - r.sin() * l * 0.5);
        let (ax, ay) = (cx + r.cos() * (l * 0.5 - 0.5), cy + r.sin() * (l * 0.5 - 0.5));
        let (bx, by) = (cx - r.cos() * (l * 0.5 - 0.5), cy - r.sin() * (l * 0.5 - 0.5));

        LevelWall {
            points: [a, b, c, d],
            collision: [ax, ay, bx, by],
            aabb: [a.min(c), b.min(d), a.max(c), b.max(d)]
        }

    }

}


// Helpers --------------------------------------------------------------------
pub fn aabb_intersect_circle(aabb: &[f64; 4], x: f64, y: f64, r: f64) -> bool {

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

pub fn line_intersect_circle(line: &[f64; 4], cx: f64, cy: f64, r: f64) -> Option<[f64; 8]> {

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


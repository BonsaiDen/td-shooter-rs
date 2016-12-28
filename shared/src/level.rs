// STD Dependencies -----------------------------------------------------------
use std::f64::consts;
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};


// External Dependencies ------------------------------------------------------
use clock_ticks;


// Internal Dependencies ------------------------------------------------------
use ::entity::PLAYER_RADIUS;


// Statics --------------------------------------------------------------------
const MAX_LEVEL_SIZE: f32 = 512.0;
const COLLISION_GRID_SPACING: f64 = 100.0;
const VISIBILITY_GRID_SPACING: f64 = PLAYER_RADIUS * 2.0;
const VISIBILITY_MAX_DISTANCE: f64 = 150.0;


// Level Abstraction ----------------------------------------------------------
pub trait LevelCollision {
    fn collision_bounds(&self, x: f64, y: f64) -> [f64; 4];
    fn collide(&self, x: &mut f32, y: &mut f32, radius: f64);
    fn collide_beam(&self, x: f64, y: f64, r: f64, l: f64) -> Option<[f64; 5]>;
    fn collide_line(&self, line: &[f64; 4]) -> Option<[f64; 5]>;
}

pub trait LevelVisibility {
    fn visible_points(&self, x: f64, y: f64) -> Vec<((f64, f64), (f64, f64))>;
    fn visibility_bounds(&self, x: f64, y: f64) -> [f64; 4];
    fn circle_visible_from(&self, cx: f64, cy: f64, radius: f64, x: f64, y: f64) -> bool;
}

#[derive(Debug, Default)]
pub struct Level {
    pub walls: Vec<LevelWall>,
    bounds: [f64; 4],
    // TODO later on we can optimize the memory access here
    collision_grid: HashMap<(isize, isize), Vec<usize>>,
    visibility_grid: HashMap<(isize, isize), Vec<(usize, usize, f64, f64)>>
}

impl Level {

    pub fn new() -> Level {
        Level {
            walls: Vec::new(),
            bounds: [1000000.0, 1000000.0, -100000.0, -1000000.0],
            collision_grid: HashMap::new(),
            visibility_grid: HashMap::new()
        }
    }

    /*
    fn pre_calculate_visibility(&mut self) {

        println!("[Level] Bounds {:?}", self.bounds);

        let start = clock_ticks::precise_time_ms();

        let (top_left, bottom_right) = (
            self.w2v(self.bounds[0], self.bounds[1]),
            self.w2v(self.bounds[2], self.bounds[3])
        );

        // Go through all possible visibility cells
        let mut visibility_grid = HashMap::new();
        for y in top_left.1..bottom_right.1 + 1 {
            for x in top_left.0..bottom_right.0 + 1 {

                // Calculate cell center
                let (cx, cy) = (
                    (x as f64) * VISIBILITY_GRID_SPACING + VISIBILITY_GRID_SPACING * 0.5,
                    (y as f64) * VISIBILITY_GRID_SPACING + VISIBILITY_GRID_SPACING * 0.5
                );

                // Get list of walls within the bounds for this cell
                let walls = self.get_walls_in_bounds(&[
                    cx - VISIBILITY_MAX_DISTANCE,
                    cy - VISIBILITY_MAX_DISTANCE,
                    cx + VISIBILITY_MAX_DISTANCE,
                    cy + VISIBILITY_MAX_DISTANCE
                ]);

                // Calculate visible points for all walls
                if !walls.is_empty() {

                    let mut visible_points = Vec::new();

                    // Add points in range for each of the walls
                    for i in &walls {
                        let wall = &self.walls[*i];
                        visible_points.push((*i, 0, wall.points[0], wall.points[1]));
                        visible_points.push((*i, 1, wall.points[2], wall.points[3]));
                    }

                    if !visible_points.is_empty() {
                        visibility_grid.insert((x, y), visible_points);
                    }

                }

            }
        }

        // Merge adjacents visibility cells and filter out duplicate entries
        let mut merged_visibility_grid = HashMap::new();
        for &(gx, gy) in visibility_grid.keys() {

            // List of all merged points
            let mut merged_visible_points = Vec::new();

            // HashSet for point de-duplication
            let mut merged_indicies = HashSet::new();

            for y in (gy - 1)..(gy + 2) {
                for x in (gx - 1)..(gx + 2) {

                    // Get points in current grid cell
                    if let Some(points) = visibility_grid.get(&(x, y)) {

                        for point in points {
                            let index = (point.0, point.1);
                            if !merged_indicies.contains(&index) {
                                merged_indicies.insert(index);
                                merged_visible_points.push(*point);
                            }
                        }

                    }

                }
            }

            if !merged_visible_points.is_empty() {

                // Get center of current grid cell
                let (cx, cy) = (
                    (gx as f64) * VISIBILITY_GRID_SPACING + VISIBILITY_GRID_SPACING * 0.5,
                    (gy as f64) * VISIBILITY_GRID_SPACING + VISIBILITY_GRID_SPACING * 0.5
                );

                // Calculate point angles
                let mut point_angles: Vec<(f64, _)> = {
                    merged_visible_points.into_iter().map(|p| {
                        let (dx, dy) = (cx - p.2, cy - p.3);
                        (dy.atan2(dx) + consts::PI, p)

                    }).collect()
                };

                // Sort points in clockwise order
                point_angles.sort_by(|a, b| {
                    if a.0 < b.0 {
                        Ordering::Greater

                    } else if a.0 > b.0 {
                        Ordering::Less

                    } else {
                        Ordering::Equal
                    }
                });

                let sorted_points: Vec<_> = point_angles.into_iter().map(|(_, p)| {
                    p

                }).collect();

                merged_visibility_grid.insert((gx, gy), sorted_points);
            }

        }

        self.visibility_grid = merged_visibility_grid;

        println!("[Level] Visibility pre-calculated in {}ms", clock_ticks::precise_time_ms() - start);

    }
    */

    fn add_wall(&mut self, wall: LevelWall) {

        {
            let aabb = &wall.aabb;
            let (top_left, bottom_right) = (
                self.w2g(aabb[0], aabb[1]),
                self.w2g(aabb[2], aabb[3])
            );

            self.bounds[0] = self.bounds[0].min(aabb[0]);
            self.bounds[1] = self.bounds[1].min(aabb[1]);

            self.bounds[2] = self.bounds[2].max(aabb[2]);
            self.bounds[3] = self.bounds[3].max(aabb[3]);

            for y in (top_left.1 - 1)..bottom_right.1 + 1 {
                for x in (top_left.0 - 1)..bottom_right.0 + 1 {
                    self.collision_grid.entry((x, y)).or_insert_with(Vec::new).push(self.walls.len());
                }
            }
        }

        self.walls.push(wall);

    }

    pub fn get_walls_in_bounds(
        &self,
        bounds: &[f64; 4]

    ) -> HashSet<usize> {

        let (top_left, bottom_right) = (
            self.w2g(bounds[0].min(bounds[2]), bounds[1].min(bounds[3])),
            self.w2g(bounds[2].max(bounds[0]), bounds[3].max(bounds[1]))
        );

        let mut walls = HashSet::new();
        for y in (top_left.1 - 1)..bottom_right.1 + 1 {
            for x in (top_left.0 - 1)..bottom_right.0 + 1 {
                if let Some(indicies) = self.collision_grid.get(&(x, y)) {
                    for i in indicies {
                        walls.insert(*i);
                    }
                }
            }
        }

        walls

    }

    pub fn load() -> Level {
        let mut level = Level::new();
        level.add_wall(LevelWall::new(100.0, 100.0, -100.0, 100.0));
        level.add_wall(LevelWall::new(-100.0, -100.0, -100.0, 100.0));
        level.add_wall(LevelWall::new(-50.0, -100.0, -50.0, 0.0));
        level.add_wall(LevelWall::new(0.0, 0.0, 100.0, -100.0));
        level.add_wall(LevelWall::new(0.0, 0.0, 100.0, 100.0));
        level.add_walls_from_bounds(&[
            -VISIBILITY_MAX_DISTANCE, -VISIBILITY_MAX_DISTANCE,
            VISIBILITY_MAX_DISTANCE, VISIBILITY_MAX_DISTANCE
        ]);
        //level.pre_calculate_visibility();
        level
    }

    fn add_walls_from_bounds(&mut self, bounds: &[f64; 4]) {

        // Top
        self.add_wall(LevelWall::new(bounds[0], bounds[1], bounds[2], bounds[1]));

        // Right
        self.add_wall(LevelWall::new(bounds[0], bounds[1], bounds[0], bounds[3]));

        // Bottom
        self.add_wall(LevelWall::new(bounds[0], bounds[3], bounds[2], bounds[3]));

        // Left
        self.add_wall(LevelWall::new(bounds[2], bounds[1], bounds[2], bounds[3]));

    }

    fn w2g(&self, x: f64, y: f64) -> (isize, isize) {
        let gx = ((x - COLLISION_GRID_SPACING * 0.5) / COLLISION_GRID_SPACING).round();
        let gy = ((y - COLLISION_GRID_SPACING * 0.5) / COLLISION_GRID_SPACING).round();
        (gx as isize, gy as isize)
    }

    fn w2v(&self, x: f64, y: f64) -> (isize, isize) {
        let gx = ((x - VISIBILITY_GRID_SPACING * 0.5) / VISIBILITY_GRID_SPACING).round();
        let gy = ((y - VISIBILITY_GRID_SPACING * 0.5) / VISIBILITY_GRID_SPACING).round();
        (gx as isize, gy as isize)
    }

    fn collide_beam_with_walls(&self, line: &[f64; 4], walls: &HashSet<usize>) -> Option<[f64; 5]> {

        let mut intersection: Option<[f64; 5]> = None;
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

#[derive(Clone)]
struct Endpoint {
    wall_index: usize,
    segment_index: usize,
    begins_segment: bool,
    r: f64,
    x: f64,
    y: f64
}

struct Segment {
    wall_index: usize,
    p1: Endpoint,
    p2: Endpoint
}

impl LevelVisibility for Level {

    // TODO version which returns wall indicies for visibility pre-calcualtion of walls from
    // certain grid spaces
    fn visible_points(&self, x: f64, y: f64) -> Vec<((f64, f64), (f64, f64))> {

        // TODO do a lookup into the visibility grid -> wall index cache
        let walls = self.get_walls_in_bounds(&[
            x - VISIBILITY_MAX_DISTANCE,
            y - VISIBILITY_MAX_DISTANCE,
            x + VISIBILITY_MAX_DISTANCE,
            y + VISIBILITY_MAX_DISTANCE
        ]);

        // Go through all walls in range
        // TODO make this more efficient later on
        // TODO figure out what information can be pre-calculated?
        // TODO do a rough estimation of all visible walls from the center
        // of the visibility grid cells so we have a better pre-filtering of
        // the walls when performing real-time player queries
        let mut endpoints = Vec::new();
        let mut segments = Vec::new();
        for wi in walls {

            let wall = &self.walls[wi];

            // Calculate endpoints
            let r1 = endpoint_angle(wall.points[0], wall.points[1], x, y);
            let r2 = endpoint_angle(wall.points[2], wall.points[3], x, y);

            let mut dr = r2 - r1;
            if dr <= -consts::PI {
                dr += 2.0 * consts::PI;
            }

            if dr > consts::PI {
                dr -= 2.0 * consts::PI;
            }

            let p1_begins_segment = dr > 0.0;
            let segment = Segment {
                wall_index: wi,
                p1: Endpoint {
                    wall_index: wi,
                    segment_index: segments.len(),
                    begins_segment: p1_begins_segment,
                    r: r1,
                    x: wall.points[0],
                    y: wall.points[1]
                },
                p2: Endpoint {
                    wall_index: wi,
                    segment_index: segments.len(),
                    begins_segment: !p1_begins_segment,
                    r: r2,
                    x: wall.points[2],
                    y: wall.points[3]
                }
            };

            endpoints.push(segment.p1.clone());
            endpoints.push(segment.p2.clone());
            segments.push(segment);

        }

        // Sort endpoints
        endpoints.sort_by(|a, b| {

            if a.r > b.r {
                Ordering::Greater

            } else if a.r < b.r {
                Ordering::Less

            } else if !a.begins_segment && b.begins_segment {
                Ordering::Greater

            } else if a.begins_segment && !b.begins_segment {
                Ordering::Less

            } else {
                Ordering::Equal
            }

        });

        // Calculate visibility
        let mut start_r = 0.0;
        let mut open_segments: Vec<isize> = Vec::new();
        let mut triangle_points = Vec::new();

        for pass in 0..2 {

            for endpoint in &endpoints {

                // Get current open segment to check if it changed later on
                // TODO optimize all of these
                let open_segment_index = open_segments.get(0).map(|i| *i).unwrap_or(-1);

                if endpoint.begins_segment {
                    let mut index = 0;
                    // TODO Clean up access
                    let mut segment_index = open_segments.get(index).map(|i| *i).unwrap_or(-1);
                    while segment_index != -1 && segment_in_front_of(x, y, &segments[endpoint.segment_index], &segments[segment_index as usize])  {
                        index += 1;
                        segment_index = open_segments.get(index).map(|i| *i).unwrap_or(-1);
                    }

                    if segment_index == -1 {
                        open_segments.push(endpoint.segment_index as isize)

                    } else {
                        open_segments.insert(index, endpoint.segment_index as isize);
                    }

                } else {
                    open_segments.retain(|index| {
                        *index != endpoint.segment_index as isize
                    })
                }

                // Check if open segment has changed
                // TODO Clean up access
                if open_segment_index != open_segments.get(0).map(|i| *i).unwrap_or(-1) {
                    if pass == 1 {
                        // segments[open_segment_index as usize].wall_index
                        triangle_points.push(get_triangle_points(x, y, start_r, endpoint.r, segments.get(open_segment_index as usize)));
                        //output.push((start_r, endpoint.r));
                        //println!("add triangle: {}", segments[open_segment_index as usize].wall_index);
                    }
                    start_r = endpoint.r;
                }

            }

        }

        // TODO return wall indicies for pre-calculation of visible walls?
        triangle_points

        // TODO perform player visibility lookup with a simpler calculation?
        // get edgepoints of enemy player circle (extrude them to compensate for lage)
        // and perform a simple 4-times line intersection, if any of the lines connects
        // we have visibility

    }

    fn visibility_bounds(&self, x: f64, y: f64) -> [f64; 4] {
        let (gx, gy) = self.w2v(x, y);
        [
            (gx as f64) * VISIBILITY_GRID_SPACING,
            (gy as f64) * VISIBILITY_GRID_SPACING,
            VISIBILITY_GRID_SPACING,
            VISIBILITY_GRID_SPACING
        ]
    }

    fn circle_visible_from(&self, ox: f64, oy: f64, radius: f64, x: f64, y: f64) -> bool {

        let (dx, dy) = (x - ox, y - oy);
        let l = (dx * dx + dy * dy).sqrt();
        if l > VISIBILITY_MAX_DISTANCE * 1.4 {
            false

        } else {
            self.collide_line(&[x, y, ox + radius, oy]).is_none()
                || self.collide_line(&[x, y, ox - radius, oy]).is_none()
                || self.collide_line(&[x, y, ox, oy + radius]).is_none()
                || self.collide_line(&[x, y, ox, oy - radius]).is_none()
        }

    }

}

impl LevelCollision for Level {

    fn collision_bounds(&self, x: f64, y: f64) -> [f64; 4] {
        let (gx, gy) = self.w2g(x, y);
        [
            (gx as f64) * COLLISION_GRID_SPACING,
            (gy as f64) * COLLISION_GRID_SPACING,
            COLLISION_GRID_SPACING,
            COLLISION_GRID_SPACING
        ]
    }

    fn collide(&self, x: &mut f32, y: &mut f32, radius: f64) {

        let walls = self.get_walls_in_bounds(&[
            *x as f64 - radius,
            *y as f64 - radius,
            *x as f64 + radius,
            *y as f64 + radius
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

        *x = x.min(MAX_LEVEL_SIZE).max(-MAX_LEVEL_SIZE);
        *y = y.min(MAX_LEVEL_SIZE).max(-MAX_LEVEL_SIZE);

    }

    fn collide_beam(&self, x: f64, y: f64, r: f64, l: f64) -> Option<[f64; 5]> {

        let line = [
            x,
            y,
            x + r.cos() * l,
            y + r.sin() * l
        ];

        self.collide_line(&line)

    }

    fn collide_line(&self, line: &[f64; 4]) -> Option<[f64; 5]> {
        self.collide_beam_with_walls(&line, &self.get_walls_in_bounds(&line))
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


// Visibility Helpers ---------------------------------------------------------
fn endpoint_angle(ax: f64, ay: f64, bx: f64, by: f64) -> f64 {
    let (dx, dy) = (ax - bx, ay - by);
    dy.atan2(dx)
}

fn point_left_of(segment: &Segment, point: (f64, f64)) -> bool {
    let cross = (segment.p2.x - segment.p1.x) * (point.1 - segment.p1.y)
              - (segment.p2.y - segment.p1.y) * (point.0 - segment.p1.x);

    cross < 0.0
}

fn interpolate_point(ax: f64, ay: f64, bx: f64, by: f64, f: f64) -> (f64, f64) {
    (
        ax * (1.0 - f) + bx * f,
        ay * (1.0 - f) + by * f
    )
}

fn segment_in_front_of(x: f64, y: f64, a: &Segment, b: &Segment) -> bool {

    let a1 = point_left_of(a, interpolate_point(b.p1.x, b.p1.y, b.p2.x, b.p2.y, 0.01));
    let a2 = point_left_of(a, interpolate_point(b.p2.x, b.p2.y, b.p1.x, b.p1.y, 0.01));
    let a3 = point_left_of(a, (x, y));
    let b1 = point_left_of(b, interpolate_point(a.p1.x, a.p1.y, a.p2.x, a.p2.y, 0.01));
    let b2 = point_left_of(b, interpolate_point(a.p2.x, a.p2.y, a.p1.x, a.p1.y, 0.01));
    let b3 = point_left_of(b, (x, y));

    if b1 == b2 && b2 != b3 {
        true

    } else if a1 == a2 && a2 == a3 {
        true

    // TODO these are superflous since we alway return false anyways
    //} else if A1 == A2 && A2 != A3 {
    //    false

    //} else if B1 == B2 && B2 == B3 {
    //    false

    } else {
        false
    }
}

fn get_triangle_points(x: f64, y: f64, r1: f64, r2: f64, segment: Option<&Segment>) -> ((f64, f64), (f64, f64)) {

    let p1 = (x, y);
    let mut p2 = (x + r1.cos(), y + r1.sin());
    let mut p3 = (0.0, 0.0);
    let mut p4 = (0.0, 0.0);

    if let Some(segment) = segment {
        p3.0 = segment.p1.x;
        p3.1 = segment.p1.y;
        p4.0 = segment.p2.x;
        p4.1 = segment.p2.y;

    // Fallback for open level bounds
    } else {
        p3.0 = x + r1.cos() * VISIBILITY_MAX_DISTANCE;
        p3.1 = y + r1.sin() * VISIBILITY_MAX_DISTANCE;
        p4.0 = x + r2.cos() * VISIBILITY_MAX_DISTANCE;
        p4.1 = y + r2.sin() * VISIBILITY_MAX_DISTANCE;
    }

    let p_begin = line_intersection(p3, p4, p1, p2);

    p2.0 = x + r2.cos();
    p2.1 = y + r2.sin();

    let p_end = line_intersection(p3, p4, p1, p2);

    (p_begin, p_end)
}


fn line_intersection(a: (f64, f64), b: (f64, f64), c: (f64, f64), d: (f64, f64)) -> (f64, f64) {

    let s = (
        (d.0 - c.0) * (a.1 - c.1) - (d.1 - c.1) * (a.0 - c.0)

    ) / (
        (d.1 - c.1) * (b.0 - a.0) - (d.0 - c.0) * (b.1 - a.1)
    );

    (
        a.0 + s * (b.0 - a.0),
        a.1 + s * (b.1 - a.1)
    )

}


// Collision Helpers ----------------------------------------------------------
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

pub fn line_intersect_line(line: &[f64; 4], other: &[f64; 4]) -> Option<[f64; 5]> {

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


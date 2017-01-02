// STD Dependencies -----------------------------------------------------------
use std::f32::consts;
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};


// External Dependencies ------------------------------------------------------
use clock_ticks;


// Internal Dependencies ------------------------------------------------------
use ::util;
use ::entity::{
    PlayerData,
    PLAYER_RADIUS,
    PLAYER_VISBILITY_CONE,
    PLAYER_VISBILITY_CONE_OFFSET
};
use super::{Level, LevelCollision, line_intersect_circle_test};


// Statics --------------------------------------------------------------------
pub const VISIBILITY_GRID_SPACING: f32 = PLAYER_RADIUS * 4.0;
pub const LEVEL_MAX_VISIBILITY_DISTANCE: f32 = 130.0;


// Traits ---------------------------------------------------------------------
pub trait LevelVisibility {
    fn calculate_visibility(&self, x: f32, y: f32, radius: f32) -> Vec<(usize, (f32, f32), (f32, f32))>;
    fn visibility_bounds(&self, x: f32, y: f32) -> [f32; 4];
    fn circle_visible_from(&self, cx: f32, cy: f32, radius: f32, x: f32, y: f32) -> bool;
    fn circle_in_light(&self, x: f32, y: f32, radius: f32) -> bool;
    fn player_within_visibility(&self, a: &PlayerData, b: &PlayerData) -> bool;
}

impl LevelVisibility for Level {

    fn calculate_visibility(&self, x: f32, y: f32, radius: f32) -> Vec<(usize, (f32, f32), (f32, f32))> {
        if let Some(walls) = self.visibility_grid.get(&self.w2v(x, y)) {
            self.get_visibility_for_walls(x, y, radius, &walls)

        } else {
            Vec::new()
        }
    }

    fn visibility_bounds(&self, x: f32, y: f32) -> [f32; 4] {
        let (gx, gy) = self.w2v(x, y);
        [
            (gx as f32) * VISIBILITY_GRID_SPACING,
            (gy as f32) * VISIBILITY_GRID_SPACING,
            VISIBILITY_GRID_SPACING,
            VISIBILITY_GRID_SPACING
        ]
    }

    fn circle_visible_from(&self, ox: f32, oy: f32, radius: f32, x: f32, y: f32) -> bool {

        let (dx, dy) = (x - ox, y - oy);
        let l = (dx * dx + dy * dy).sqrt();
        if l > LEVEL_MAX_VISIBILITY_DISTANCE * 1.4 {
            false

        } else {
            self.collide_line(&[x, y, ox + radius, oy]).is_none()
                || self.collide_line(&[x, y, ox - radius, oy]).is_none()
                || self.collide_line(&[x, y, ox, oy + radius]).is_none()
                || self.collide_line(&[x, y, ox, oy - radius]).is_none()
        }

    }

    fn circle_in_light(&self, x: f32, y: f32, radius: f32) -> bool {
        for light in &self.lights {
            if light.circle_intersect(x, y, radius) {
                if self.circle_visible_from(
                    x,
                    y,
                    radius,
                    light.x,
                    light.y
                ) {
                    return true;
                }
            }
        }
        false
    }

    fn player_within_visibility(&self, a: &PlayerData, b: &PlayerData) -> bool {

        // Players standing in a light circle are always visible
        if self.circle_in_light(b.x, b.y, PLAYER_RADIUS) {
            true

        // Dead players cannot see any other players
        } else if a.hp == 0 {
            false

        // Players outside the maximum visibility distance are never visible
        } else if util::distance(b.x, b.y, a.x, a.y) > LEVEL_MAX_VISIBILITY_DISTANCE - PLAYER_VISBILITY_CONE_OFFSET + PLAYER_RADIUS * 0.5 {
            false

        // Players outside the visibility cone are never visible
        } else if !within_visibility_cone(a.x, a.y, a.r, b.x, b.y, PLAYER_VISBILITY_CONE_OFFSET, PLAYER_VISBILITY_CONE, PLAYER_RADIUS) {
            false

        // Players within the visibility cone are only visible if sight is not blocked by a wall
        } else {
            self.circle_visible_from(b.x, b.y, PLAYER_RADIUS, a.x, a.y)
        }

    }

}

// Internal -------------------------------------------------------------------
impl Level {

    pub fn w2v(&self, x: f32, y: f32) -> (isize, isize) {
        let gx = ((x - VISIBILITY_GRID_SPACING * 0.5) / VISIBILITY_GRID_SPACING).round();
        let gy = ((y - VISIBILITY_GRID_SPACING * 0.5) / VISIBILITY_GRID_SPACING).round();
        (gx as isize, gy as isize)
    }

    pub fn pre_calculate_visibility(&mut self) {

        println!("[Level] Bounds {:?}", self.bounds);

        let start = clock_ticks::precise_time_ms();

        let (top_left, bottom_right) = (
            self.w2v(self.bounds[0], self.bounds[1]),
            self.w2v(self.bounds[2], self.bounds[3])
        );

        // Go through all possible visibility cells
        let mut visibility_grid: HashMap<(isize, isize), Vec<usize>> = HashMap::new();
        for y in top_left.1..bottom_right.1 + 1 {
            for x in top_left.0..bottom_right.0 + 1 {

                // Calculate cell center
                let (cx, cy) = (
                    (x as f32) * VISIBILITY_GRID_SPACING + VISIBILITY_GRID_SPACING * 0.5,
                    (y as f32) * VISIBILITY_GRID_SPACING + VISIBILITY_GRID_SPACING * 0.5
                );

                let walls = self.get_walls_in_bounds(&[
                    cx - LEVEL_MAX_VISIBILITY_DISTANCE,
                    cy - LEVEL_MAX_VISIBILITY_DISTANCE,
                    cx + LEVEL_MAX_VISIBILITY_DISTANCE,
                    cy + LEVEL_MAX_VISIBILITY_DISTANCE
                ]);

                visibility_grid.insert(
                    (x, y),
                    self.get_visibility_for_walls(
                        cx,
                        cy,
                        LEVEL_MAX_VISIBILITY_DISTANCE,
                        &walls

                    ).into_iter().map(|v| v.0).collect()
                );

            }
        }

        // Merge adjacents visibility cells and filter out duplicate entries
        let mut merged_grid = HashMap::new();
        for &(gx, gy) in visibility_grid.keys() {

            let mut visible_walls: HashSet<usize> = HashSet::new();

            // Get current cell and its 8 neighbors
            for y in (gy - 1)..(gy + 2) {
                for x in (gx - 1)..(gx + 2) {

                    // Merge all visibile wall indicies
                    if let Some(wall_indicies) = visibility_grid.get(&(x, y)) {
                        for index in wall_indicies {
                            visible_walls.insert(*index);
                        }
                    }

                }
            }

            if !visible_walls.is_empty() {
                merged_grid.insert((gx, gy), visible_walls);
            }

        }

        self.visibility_grid = merged_grid;

        println!("[Level] Visibility pre-calculated in {}ms", clock_ticks::precise_time_ms() - start);

    }

    fn get_visibility_segments(&self, x: f32, y: f32, walls: &HashSet<usize>) -> (Vec<Segment>, Vec<Endpoint>) {

        // Go through all walls in range
        let mut endpoints = Vec::new();
        let mut segments = Vec::new();
        for i in walls {

            let wall = &self.walls[*i];

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
                wall_index: *i,
                p1: Endpoint {
                    wall_index: *i,
                    segment_index: segments.len(),
                    begins_segment: p1_begins_segment,
                    r: r1,
                    x: wall.points[0],
                    y: wall.points[1]
                },
                p2: Endpoint {
                    wall_index: *i,
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

        (segments, endpoints)

    }

    fn get_visibility_for_walls(
        &self,
        x: f32,
        y: f32,
        max_distance: f32,
        walls: &HashSet<usize>

    ) -> Vec<(usize, (f32, f32), (f32, f32))> {

        let (segments, endpoints) = self.get_visibility_segments(x, y, &walls);

        let mut open_segments: Vec<isize> = Vec::new();
        let mut visibility = Vec::new();
        let mut r = 0.0;

        for pass in 0..2 {

            for endpoint in &endpoints {

                // Get current open segment to check if it changed later on
                // TODO optimize all of these
                let open_segment_index = open_segments.get(0).map_or(-1, |i| *i);

                if endpoint.begins_segment {

                    let mut index = 0;
                    // TODO Clean up access
                    let mut segment_index = open_segments.get(index).map_or(-1, |i| *i);
                    while segment_index != -1 && segment_in_front_of(
                        x, y,
                        &segments[endpoint.segment_index],
                        &segments[segment_index as usize]
                    )  {
                        index += 1;
                        segment_index = open_segments.get(index).map_or(-1, |i| *i);
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
                if open_segment_index != open_segments.get(0).map_or(-1, |i| *i) {

                    if pass == 1 {

                        let segment = segments.get(open_segment_index as usize);
                        let points = get_triangle_points(x, y, r, endpoint.r, segment, max_distance);
                        visibility.push((
                            segment.map_or(0, |s| s.wall_index),
                            points.0,
                            points.1
                        ));

                    }

                    r = endpoint.r;

                }

            }

        }

        visibility

    }

}


// Visibility Helpers ---------------------------------------------------------
#[derive(Clone)]
struct Endpoint {
    wall_index: usize,
    segment_index: usize,
    begins_segment: bool,
    r: f32,
    x: f32,
    y: f32
}

struct Segment {
    wall_index: usize,
    p1: Endpoint,
    p2: Endpoint
}

fn endpoint_angle(ax: f32, ay: f32, bx: f32, by: f32) -> f32 {
    let (dx, dy) = (ax - bx, ay - by);
    dy.atan2(dx)
}

fn point_left_of(segment: &Segment, point: (f32, f32)) -> bool {
    let cross = (segment.p2.x - segment.p1.x) * (point.1 - segment.p1.y)
              - (segment.p2.y - segment.p1.y) * (point.0 - segment.p1.x);

    cross < 0.0
}

fn interpolate_point(ax: f32, ay: f32, bx: f32, by: f32, f: f32) -> (f32, f32) {
    (
        ax * (1.0 - f) + bx * f,
        ay * (1.0 - f) + by * f
    )
}

fn segment_in_front_of(x: f32, y: f32, a: &Segment, b: &Segment) -> bool {

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

    } else {
        false
    }
}

fn get_triangle_points(
    x: f32, y: f32,
    r1: f32, r2: f32,
    segment: Option<&Segment>,
    max_distance: f32

) -> ((f32, f32), (f32, f32)) {

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
        p3.0 = x + r1.cos() * max_distance * 1.4;
        p3.1 = y + r1.sin() * max_distance * 1.4;
        p4.0 = x + r2.cos() * max_distance * 1.4;
        p4.1 = y + r2.sin() * max_distance * 1.4;
    }

    let p_begin = line_intersection(p3, p4, p1, p2);

    p2.0 = x + r2.cos();
    p2.1 = y + r2.sin();

    let p_end = line_intersection(p3, p4, p1, p2);

    (p_begin, p_end)
}


fn line_intersection(a: (f32, f32), b: (f32, f32), c: (f32, f32), d: (f32, f32)) -> (f32, f32) {

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

fn within_visibility_cone(
    x: f32, y: f32,
    r: f32,
    ox: f32, oy: f32,
    offset: f32,
    cone: f32,
    radius: f32

) -> bool {

    // Move to offset location behind player
    let (cx, cy) = (x - r.cos() * offset, y - r.sin() * offset);

    // Get intial angle between players
    let (dx, dy) = (ox - cx, oy - cy);
    let or = dy.atan2(dx);

    // Check on which side of the cone the other player is
    let dr = r - or;
    let cr = dr.sin().atan2(dr.cos());

    // Directly within the cone
    if cr.abs() < cone {
        true

    // Definitely not on the edge
    } else if cr.abs() > cone * 1.5 {
        false

    // Pontentially on one of the edges
    } else if cr > 0.0 {

        let edge = [
            cx,
            cy,
            cx + (r - cone).cos() * LEVEL_MAX_VISIBILITY_DISTANCE,
            cy + (r - cone).sin() * LEVEL_MAX_VISIBILITY_DISTANCE
        ];

        line_intersect_circle_test(&edge, ox, oy, radius)

    } else if cr < 0.0 {

        let edge = [
            cx,
            cy,
            cx + (r + cone).cos() * LEVEL_MAX_VISIBILITY_DISTANCE,
            cy + (r + cone).sin() * LEVEL_MAX_VISIBILITY_DISTANCE
        ];

        line_intersect_circle_test(&edge, ox, oy, radius)

    } else {
        false
    }

}


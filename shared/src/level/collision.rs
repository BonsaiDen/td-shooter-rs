// STD Dependencies -----------------------------------------------------------
use std::f32::consts;


// Internal Dependencies ------------------------------------------------------
use ::level::{Level, MAX_LEVEL_SIZE};
use ::collision::{
    aabb_intersect_circle,
    line_intersect_line,
    line_intersect_circle
};


// Statics --------------------------------------------------------------------
pub const COLLISION_GRID_SPACING: f32 = 200.0;


// Traits ---------------------------------------------------------------------
pub trait LevelCollision {
    fn collide(&self, x: &mut f32, y: &mut f32, radius: f32, active: bool);
    fn collide_beam(&self, x: f32, y: f32, r: f32, l: f32) -> Option<(usize, [f32; 3])>;
    fn collide_beam_wall(&self, x: f32, y: f32, r: f32, l: f32) -> Option<f32>;
    fn collide_line(&self, line: &[f32; 4]) -> Option<(usize, [f32; 3])>;
}

impl LevelCollision for Level {

    fn collide(&self, x: &mut f32, y: &mut f32, radius: f32, active: bool) {

        let mut iterations = 0;
        let mut collisions = 1;
        while collisions > 0 && iterations < 10 {

            collisions = 0;

            let mut overlap = (0.0, 0.0);
            for wall in &self.walls {

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
            if !active && overlap.0.abs() < 0.1 && overlap.1.abs() < 0.1 {
                break;
            }

            *x -= overlap.0;
            *y -= overlap.1;

            iterations += 1;

            // No need to iterate idle entities multiple times per frame
            if !active {
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
        self.collide_beam_with_walls(line)
    }

}

// Internal Helpers -----------------------------------------------------------
impl Level {

    pub fn w2g(&self, x: f32, y: f32) -> (isize, isize) {
        let gx = ((x - COLLISION_GRID_SPACING * 0.5) / COLLISION_GRID_SPACING).round();
        let gy = ((y - COLLISION_GRID_SPACING * 0.5) / COLLISION_GRID_SPACING).round();
        (gx as isize, gy as isize)
    }

    fn collide_beam_with_walls(&self, line: &[f32; 4]) -> Option<(usize, [f32; 3])> {

        let mut intersection: Option<(usize, [f32; 3])> = None;
        for (i, wall) in self.walls.iter().enumerate() {

            if let Some(new) = line_intersect_line(line, &wall.points) {

                let is_closer = if let Some(existing) = intersection {
                    new[2] < existing.1[2]

                } else {
                    true
                };

                if is_closer {
                    intersection = Some((i, new));
                }

            }
        }

        intersection

    }

}


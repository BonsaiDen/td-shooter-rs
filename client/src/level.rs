// External Dependencies ------------------------------------------------------
use piston_window::{
    Context, G2d, DrawState, Graphics, Rectangle,
    line,
    rectangle,
    triangulation,
};


// Internal Dependencies ------------------------------------------------------
use shared::level::{
    Level as SharedLevel,
    LevelCollision,
    LevelVisibility,
    aabb_intersect_circle,
    line_intersect_circle
};


// Client Level ---------------------------------------------------------------
#[derive(Debug, Default)]
pub struct Level {
    level: SharedLevel
}

impl Level {

    pub fn new(level: SharedLevel) -> Level {
        Level {
            level: level
        }
    }

    pub fn draw_2d(
        &self,
        c: Context,
        g: &mut G2d,
        bounds: &[f64; 4],
        x: f64, y: f64,
        radius: f64,
        debug: bool
    ) {

        // Background Box
        rectangle(
            [0.3, 0.3, 0.3, 1.0],
            [bounds[0], bounds[1], bounds[2] - bounds[0], bounds[3] - bounds[1]],
            c.transform, g
        );

        if debug {

            let collision_grid_color = [0.1, 0.4, 0.1, 0.5];
            let visibility_grid_color = [0.6, 0.6, 0.1, 0.5];

            // Collision grid debug
            rectangle(
                collision_grid_color,
                self.level.collision_bounds(x, y),
                c.transform, g
            );

            // Visibility grid debug
            rectangle(
                visibility_grid_color,
                self.level.visibility_bounds(x, y),
                c.transform, g
            );

        }

        // Get all walls within the screen bounds
        let walls = self.level.get_walls_in_bounds(&bounds);
        for i in &walls {

            let wall = &self.level.walls[*i];
            let wall_color = [0.8, 0.8, 0.8, 1.0];

            // Wall drawing
            line(wall_color,
                1.0,
                wall.points,
                c.transform, g);

            if debug {

                // Collision / intersection debug
                let mut markers = None;
                if aabb_intersect_circle(&wall.aabb, x, y, radius + 2.0) {

                    if let Some(collision) = line_intersect_circle(
                        &wall.collision,
                        x,
                        y,
                        radius + 2.0
                    ) {
                        markers = Some(collision);
                    }

                }

                // Player collision detection debug
                if let Some(markers) = markers {
                    rectangle([0.3, 1.0, 0.3, 1.0],
                                [markers[0] - 1.0, markers[1] - 1.0, 2.0, 2.0],
                                c.transform, g);

                    rectangle([0.3, 1.0, 0.3, 1.0],
                                [markers[2] - 1.0, markers[3] - 1.0, 2.0, 2.0],
                                c.transform, g);

                    line([0.3, 0.3, 1.0, 1.0],
                        0.5,
                        [markers[4], markers[5], x, y],
                        c.transform, g);

                }

            }

        }

    }

    pub fn draw_2d_overlay(
        &self,
        c: Context,
        g: &mut G2d,
        bounds: &[f64; 4],
        x: f64, y: f64,
        _: bool
    ) {

        // Visibility overlay
        let mut polygon = Vec::new();
        for (a, b) in self.visible_points(x, y) {
            polygon.push([x, y]);
            polygon.push([a.0, a.1]);
            polygon.push([b.0, b.1]);
        }

        // Stencil buffer
        g.tri_list(
            &DrawState::new_clip(),
            &[1.0, 1.0, 1.0, 1.0],
            |f| {
                let n = polygon.len();
                let mut i = 0;
                triangulation::stream_polygon_tri_list(c.transform, || {
                    if i >= n { return None; }
                    let j = i;
                    i += 1;
                    Some(polygon[j])

                }, f);

            }
        );

        // Actual overlay
        Rectangle::new([0.0, 0.0, 0.0, 0.75]).draw(
            [bounds[0], bounds[1], bounds[2] - bounds[0], bounds[3] - bounds[1]],
            &DrawState::new_outside(),
            c.transform, g
        );

    }

}

impl LevelVisibility for Level {

    fn visible_points(&self, x: f64, y: f64) -> Vec<((f64, f64), (f64, f64))> {
        self.level.visible_points(x, y)
    }

    fn visibility_bounds(&self, x: f64, y: f64) -> [f64; 4] {
        self.level.visibility_bounds(x, y)
    }

    fn circle_visible_from(&self, cx: f64, cy: f64, radius: f64, x: f64, y: f64) -> bool {
        self.level.circle_visible_from(cx, cy, radius, x, y)
    }

}

impl LevelCollision for Level {

    fn collision_bounds(&self, x: f64, y: f64) -> [f64; 4] {
        self.level.collision_bounds(x, y)
    }

    fn collide(&self, x: &mut f32, y: &mut f32, radius: f64) {
        self.level.collide(x, y, radius);
    }

    fn collide_beam(&self, x: f64, y: f64, r: f64, l: f64) -> Option<[f64; 5]> {
        self.level.collide_beam(x, y, r, l)
    }

    fn collide_line(&self, line: &[f64; 4]) -> Option<[f64; 5]> {
        self.level.collide_line(line)
    }

}


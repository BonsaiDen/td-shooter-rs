// External Dependencies ------------------------------------------------------
use piston_window::{
    Context, G2d,
    line,
    rectangle
};


// Internal Dependencies ------------------------------------------------------
use shared::level::{
    Level as SharedLevel,
    LevelCollision,
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
        radius: f64
    ) {

        let walls = self.level.get_walls_in_bounds(&bounds);
        for i in &walls {

            let wall = &self.level.walls[*i];

            let mut aabb_color = [0.5, 0.4, 0.4, 1.0];
            let mut line_color = [0.5, 0.5, 0.5, 1.0];

            let mut markers = None;
            if aabb_intersect_circle(&wall.aabb, x, y, radius + 2.0) {

                aabb_color = [0.8, 0.4, 0.4, 1.0];

                if let Some(collision) = line_intersect_circle(
                    &wall.collision,
                    x,
                    y,
                    radius + 2.0
                ) {
                    markers = Some(collision);
                    line_color = [0.8, 0.6, 0.6, 1.0];
                }

            }

            rectangle(aabb_color,
                      [
                        wall.aabb[0],
                        wall.aabb[1],
                        wall.aabb[2] - wall.aabb[0],
                        wall.aabb[3] - wall.aabb[1]
                      ],
                      c.transform, g);

            line(line_color,
                1.0,
                wall.points,
                c.transform, g);

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

        rectangle(
            [0.0, 1.0, 0.0, 1.0],
            [bounds[0] - 5.0, bounds[1] - 5.0, 10.0, 10.0],
            c.transform, g
        );

        rectangle(
            [0.0, 1.0, 0.0, 1.0],
            [bounds[2] - 5.0, bounds[3] - 5.0, 10.0, 10.0],
            c.transform, g
        );


    }

}

impl LevelCollision for Level {

    fn collide(&self, x: &mut f32, y: &mut f32, radius: f64) {
        self.level.collide(x, y, radius);
    }

    fn collide_beam(&self, x: f64, y: f64, r: f64, l: f64) -> Option<[f64; 5]> {
        self.level.collide_beam(x, y, r, l)
    }

}


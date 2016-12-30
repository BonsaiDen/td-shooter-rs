// Internal Dependencies ------------------------------------------------------
use ::renderer::{Renderer, StencilMode};
use ::camera::Camera;
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

    pub fn bounds(&self) -> &[f64; 4] {
        &self.level.bounds
    }

    pub fn render_background(
        &self,
        renderer: &mut Renderer,
        camera: &Camera,
        x: f64,
        y: f64,
        debug: bool
    ) {
        let bounds = self.level.bounds;
        renderer.set_color([0.3, 0.3, 0.3, 1.0]);
        renderer.rectangle(
            camera.context(),
            &[bounds[0], bounds[1], bounds[2] - bounds[0], bounds[3] - bounds[1]],
        );
    }

    pub fn render_lights(
        &self,
        renderer: &mut Renderer,
        camera: &Camera,
        debug: bool
    ) {

        // TODO pre-render stencil value into a buffer in order to speed up
        // rendering
        let bounds = camera.b2w();
        let context = camera.context();

        // Render light visibility cones into stencil
        renderer.set_stencil_mode(StencilMode::Replace(254));
        for light in &self.level.lights {
            if aabb_intersect(&light.aabb, &bounds) {
                let endpoints = self.calculate_visibility(light.x, light.y);
                renderer.light_polygon(&context, light.x, light.y, &endpoints);
            }
        }

        // Render light cirlces into stencil combining with the cones
        renderer.set_stencil_mode(StencilMode::Add(1));
        for light in &self.level.lights {
            // Only draw visible lights
            if aabb_intersect(&light.aabb, &bounds) {
                renderer.circle(&context, 12, light.x, light.y, light.radius);
            }
        }

        // Render light color circles based on stencil
        renderer.set_stencil_mode(StencilMode::Inside(255));
        renderer.set_color([0.7, 0.6, 0.0, 0.2]);
        renderer.rectangle(
            camera.context(),
            &[bounds[0], bounds[1], bounds[2] - bounds[0], bounds[3] - bounds[1]],
        );

    }

    pub fn render_shadow(
        &self,
        renderer: &mut Renderer,
        camera: &Camera,
        x: f64,
        y: f64,
        debug: bool
    ) {

    }

    pub fn render_walls(
        &self,
        renderer: &mut Renderer,
        camera: &Camera,
        x: f64,
        y: f64,
        debug: bool
    ) {

        let bounds = camera.b2w();
        let context = camera.context();
        let endpoints = self.calculate_visibility(x, y);

        // Render player visibility cone
        renderer.set_stencil_mode(StencilMode::Replace(255));
        renderer.light_polygon(&context, x, y, &endpoints);

        // Render shadows
        renderer.set_stencil_mode(StencilMode::Outside(255));
        renderer.set_color([0.0, 0.0, 0.0, 0.75]);
        renderer.rectangle(
            camera.context(),
            &[bounds[0], bounds[1], bounds[2] - bounds[0], bounds[3] - bounds[1]],
        );

    }

    /*
    pub fn draw_2d_background(
        &self,
        c: Context,
        g: &mut G2d,
        bounds: &[f64; 4],
        x: f64, y: f64,
        _: f64,
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

    }

    pub fn draw_2d_walls(
        &self,
        c: Context,
        g: &mut G2d,
        bounds: &[f64; 4],
        x: f64, y: f64,
        radius: f64,
        debug: bool
    ) {

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
        for (_, a, b) in self.calculate_visibility(x, y) {
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

        for (i, light) in self.level.lights.iter().enumerate() {
            if aabb_intersect(&light.aabb, &bounds) {

                let vertices = &self.light_vertices[i];
                g.tri_list(
                    &DrawState::new_clip(),
                    &[1.0, 1.0, 1.0, 1.00],
                    |f| {
                        let n = vertices.len();
                        let mut i = 0;
                        triangulation::stream_polygon_tri_list(c.transform, || {
                            if i >= n { return None; }
                            let j = i;
                            i += 1;
                            Some(vertices[j])

                        }, f);

                    }
                );

            }
        }

        // Actual overlay
        Rectangle::new([0.0, 0.0, 0.0, 0.75]).draw(
            [bounds[0], bounds[1], bounds[2] - bounds[0], bounds[3] - bounds[1]],
            &DrawState::new_outside(),
            c.transform, g
        );

    }
    */

}

fn aabb_intersect(a: &[f64; 4], b: &[f64; 4]) -> bool {
    !(b[0] > a[2] || b[2] < a[0] || b[1] > a[3] || b[3] < a[1])
}

impl LevelVisibility for Level {

    fn calculate_visibility(&self, x: f64, y: f64) -> Vec<(usize, (f64, f64), (f64, f64))> {
        self.level.calculate_visibility(x, y)
    }

    fn visibility_bounds(&self, x: f64, y: f64) -> [f64; 4] {
        self.level.visibility_bounds(x, y)
    }

    fn circle_visible_from(&self, cx: f64, cy: f64, radius: f64, x: f64, y: f64) -> bool {
        self.level.circle_visible_from(cx, cy, radius, x, y)
    }

    fn circle_in_light(&self, x: f64, y: f64, radius: f64) -> bool {
        self.level.circle_in_light(x, y, radius)
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


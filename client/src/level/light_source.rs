// External Dependencies ------------------------------------------------------
use rand;
use rand::Rng;
use graphics::Transformed;


// Internal Dependencies ------------------------------------------------------
use ::camera::Camera;
use ::renderer::{Renderer, SimplePolygon, Circle};
use ::shared::level::{Level, LightSource as SharedLightSource, LevelVisibility};
use ::shared::collision::line_segment_intersect_circle_test;

// Cached Light Source for Fast Rendering -------------------------------------
#[derive(Debug)]
pub struct LightSource {
    x: f32,
    y: f32,
    s: f64,
    aabb: [f32; 4],
    clipped_walls: usize,
    light_polygon: SimplePolygon,
    light_circle: Circle
}

impl LightSource {

    pub fn from_light(level: &Level, light: &SharedLightSource) -> LightSource {

        // Figure out if we actually intersect with any walls
        // if not we can render a simple circle instead of the visibility polygon
        // in the first pass of the stencil buffer
        let mut clipped_walls = 0;
        for w in &level.walls {
            if line_segment_intersect_circle_test(&w.points, light.x, light.y, light.radius) {
                clipped_walls += 1;
            }
        }

        let light_polygon = if clipped_walls > 0 {
            level.calculate_visibility(light.x, light.y, light.radius * 1.4)

        } else {
            Vec::new()
        };

        println!("[Level] Light clipped {} walls.", clipped_walls);

        LightSource {
            aabb: light.aabb,
            x: light.x,
            y: light.y,
            s: rand::thread_rng().next_f64(),
            clipped_walls: clipped_walls,
            light_polygon: SimplePolygon::new(light_polygon),
            light_circle: Circle::new(16, 0.0, 0.0, light.radius)
        }
    }

    pub fn render_visibility_stencil(
        &self,
        renderer: &mut Renderer,
        camera: &Camera
    ) {
        let bounds = camera.b2w();
        if aabb_intersect(&self.aabb, &bounds) {
            if self.clipped_walls > 0 {
                let context = camera.context();
                self.light_polygon.render(renderer, &context);

            } else {
                let context = camera.context().trans(
                    self.x as f64,
                    self.y as f64

                );
                self.light_circle.render(renderer, &context);
            }
        }
    }

    pub fn render_light_stencil(
        &self,
        renderer: &mut Renderer,
        camera: &Camera
    ) {
        let bounds = camera.b2w();
        if aabb_intersect(&self.aabb, &bounds) {
            let s = 1.0 - ((renderer.t() as f64 * 0.003 + self.s).cos() * 0.025).abs();
            let context = camera.context().trans(
                self.x as f64,
                self.y as f64

            ).scale(s, s);
            self.light_circle.render(renderer, &context);
        }
    }

    pub fn render_light_circle(
        &self,
        renderer: &mut Renderer,
        camera: &Camera
    ) {
        let bounds = camera.b2w();
        if aabb_intersect(&self.aabb, &bounds) {
            let s = 0.9 - ((renderer.t() as f64 * 0.003 + self.s).cos() * 0.035).abs();
            let context = camera.context().trans(
                self.x as f64,
                self.y as f64

            ).scale(s, s);
            self.light_circle.render(renderer, &context);
        }
    }

}


// Helpers --------------------------------------------------------------------
fn aabb_intersect(a: &[f32; 4], b: &[f32; 4]) -> bool {
    !(b[0] > a[2] || b[2] < a[0] || b[1] > a[3] || b[3] < a[1])
}


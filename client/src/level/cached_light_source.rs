// External Dependencies ------------------------------------------------------
use rand;
use rand::Rng;
use graphics::Transformed;


// Internal Dependencies ------------------------------------------------------
use ::camera::Camera;
use ::renderer::{Renderer, LightPoylgon, Circle};
use ::shared::level::{Level, LightSource, LevelVisibility};


// Cached Light Source for Fast Rendering -------------------------------------
#[derive(Debug)]
pub struct CachedLightSource {
    x: f32,
    y: f32,
    s: f64,
    aabb: [f32; 4],
    light_polygon: LightPoylgon,
    light_circle: Circle
}

impl CachedLightSource {

    pub fn from_light(level: &Level, light: &LightSource) -> CachedLightSource {
        CachedLightSource {
            aabb: light.aabb,
            x: light.x,
            y: light.y,
            s: rand::thread_rng().next_f64(),
            light_polygon: LightPoylgon::new(
                light.x,
                light.y,
                &level.calculate_visibility(light.x, light.y)
            ),
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
            let context = camera.context();
            self.light_polygon.render(renderer, &context);
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

}


// Helpers --------------------------------------------------------------------
fn aabb_intersect(a: &[f32; 4], b: &[f32; 4]) -> bool {
    !(b[0] > a[2] || b[2] < a[0] || b[1] > a[3] || b[3] < a[1])
}


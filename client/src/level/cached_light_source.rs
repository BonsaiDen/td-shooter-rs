// Internal Dependencies ------------------------------------------------------
use ::camera::Camera;
use ::renderer::Renderer;
use ::shared::level::{Level, LightSource, LevelVisibility};


// Cached Light Source for Fast Rendering -------------------------------------
#[derive(Debug)]
pub struct CachedLightSource {
    x: f64,
    y: f64,
    radius: f64,
    aabb: [f64; 4],
    light_polygon: Vec<(usize, (f64, f64), (f64, f64))>
}

impl CachedLightSource {

    pub fn from_light(level: &Level, light: &LightSource) -> CachedLightSource {
        CachedLightSource {
            x: light.x,
            y: light.y,
            radius: light.radius,
            aabb: light.aabb,
            light_polygon: level.calculate_visibility(light.x, light.y)
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
            renderer.light_polygon(&context, self.x, self.y, &self.light_polygon);
        }
    }

    pub fn render_light_stencil(
        &self,
        renderer: &mut Renderer,
        camera: &Camera
    ) {
        let bounds = camera.b2w();
        if aabb_intersect(&self.aabb, &bounds) {
            let context = camera.context();
            renderer.circle(&context, 12, self.x, self.y, self.radius);
        }
    }

}


// Helpers --------------------------------------------------------------------
fn aabb_intersect(a: &[f64; 4], b: &[f64; 4]) -> bool {
    !(b[0] > a[2] || b[2] < a[0] || b[1] > a[3] || b[3] < a[1])
}


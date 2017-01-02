// External Dependencies ------------------------------------------------------
use graphics::Context;


// Internal Dependencies ------------------------------------------------------
use ::Renderer;


// Light Polygon --------------------------------------------------------------
#[derive(Debug)]
pub struct LightPoylgon {
    vertices: Vec<f32>
}

impl LightPoylgon {

    pub fn new(
        x: f32,
        y: f32,
        endpoints: &[(usize, (f32, f32), (f32, f32))]

    ) -> LightPoylgon {
        LightPoylgon {
            vertices: LightPoylgon::vertices(x, y, endpoints)
        }
    }

    pub fn render(&self, renderer: &mut Renderer, context: &Context) {
        renderer.draw_triangle_list(&context.transform, &self.vertices);
    }

    pub fn vertices(
        mut x: f32,
        mut y: f32,
        endpoints: &[(usize, (f32, f32), (f32, f32))]

    ) -> Vec<f32> {

        // Snap points
        x = (x * 10000.0).round() * 0.0001;
        y = (y * 10000.0).round() * 0.0001;

        let mut vertices = Vec::new();
        for &(_, a, b) in endpoints {
            vertices.push(x);
            vertices.push(y);
            vertices.push((a.0 * 10000.0).round() * 0.0001);
            vertices.push((a.1 * 10000.0).round() * 0.0001);
            vertices.push((b.0 * 10000.0).round() * 0.0001);
            vertices.push((b.1 * 10000.0).round() * 0.0001);
        }
        vertices
    }

}


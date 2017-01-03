// External Dependencies ------------------------------------------------------
use graphics::Context;


// Internal Dependencies ------------------------------------------------------
use ::Renderer;
use super::util::triangulate;


// Concave Polygon ------------------------------------------------------------
#[derive(Debug)]
pub struct Polygon {
    pub aabb: [f32; 4],
    vertices: Vec<f32>
}

impl Polygon {

    pub fn new(points: &[[f32; 2]]) -> Polygon {

        let mut bounds = [1000000.0f32, 1000000.0, -100000.0, -1000000.0];
        for p in points {
            bounds[0] = bounds[0].min(p[0]);
            bounds[1] = bounds[1].min(p[1]);
            bounds[2] = bounds[2].max(p[0]);
            bounds[3] = bounds[3].max(p[1]);
        }

        Polygon {
            aabb: bounds,
            vertices: Polygon::vertices(points)
        }
    }

    pub fn render(&self, renderer: &mut Renderer, context: &Context) {
        renderer.draw_triangle_list(&context.transform, &self.vertices);
    }

    pub fn vertices(points: &[[f32; 2]]) -> Vec<f32> {
        triangulate(points.to_vec())
    }

}


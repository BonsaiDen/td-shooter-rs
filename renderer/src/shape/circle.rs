// STD Dependencies -----------------------------------------------------------
use std::f32::consts;


// External Dependencies ------------------------------------------------------
use graphics::Context;


// Internal Dependencies ------------------------------------------------------
use ::Renderer;


// Circle ---------------------------------------------------------------------
#[derive(Debug)]
pub struct Circle {
    vertices: Vec<f32>
}

impl Circle {

    pub fn new(
        segments: usize,
        x: f32,
        y: f32,
        r: f32

    ) -> Circle {
        Circle {
            vertices: Circle::vertices(segments, x, y, r)
        }
    }

    pub fn render(&self, renderer: &mut Renderer, context: &Context) {
        renderer.draw_triangle_list(&context.transform, &self.vertices);
    }

    pub fn vertices(
        segments: usize,
        x: f32,
        y: f32,
        r: f32

    ) -> Vec<f32> {

        let step = consts::PI * 2.0 / segments as f32;
        let mut vertices = Vec::new();
        for i in 0..segments {

            // Center
            vertices.push((x * 10000.0).round() * 0.0001);
            vertices.push((y * 10000.0).round() * 0.0001);

            // First outer point
            let ar = i as f32 * step;
            let (ax, ay) = (x + ar.cos() * r, y + ar.sin() * r);
            vertices.push((ax * 10000.0).round() * 0.0001);
            vertices.push((ay * 10000.0).round() * 0.0001);

            // Second outer point
            let br = ar + step;
            let (bx, by) = (x + br.cos() * r, y + br.sin() * r);
            vertices.push((bx * 10000.0).round() * 0.0001);
            vertices.push((by * 10000.0).round() * 0.0001);

        }

        vertices

    }

}


// STD Dependencies -----------------------------------------------------------
use std::f32::consts;


// External Dependencies ------------------------------------------------------
use graphics::Context;


// Internal Dependencies ------------------------------------------------------
use ::Renderer;


// Circle Arc -----------------------------------------------------------------
#[derive(Debug)]
pub struct CircleArc {
    vertices: Vec<f32>
}

impl CircleArc {

    pub fn new(
        segments: usize,
        x: f32,
        y: f32,
        r: f32,
        angle: f32,
        half_cone: f32

    ) -> CircleArc {
        CircleArc {
            vertices: CircleArc::vertices(segments, x, y, r, angle, half_cone)
        }
    }

    pub fn render(&self, renderer: &mut Renderer, context: &Context) {
        renderer.draw_triangle_list(&context.transform, &self.vertices);
    }

    pub fn vertices(
        segments: usize,
        x: f32,
        y: f32,
        r: f32,
        angle: f32,
        half_cone: f32

    ) -> Vec<f32> {

        let step = consts::PI * 2.0 / segments as f32;
        let mut vertices = Vec::new();
        for i in 0..segments {

            let mut ar = i as f32 * step;
            let mut br = ar + step;

            // Distance from center
            let adr = ar - angle;
            let adr = adr.sin().atan2(adr.cos()).abs();

            let bdr = br - angle;
            let bdr = bdr.sin().atan2(bdr.cos()).abs();

            // See if segments falls within cone
            if bdr < half_cone || adr < half_cone {

                // Limit angle of a
                if adr > half_cone {
                    ar = angle - half_cone;
                }

                // Limit angle of b
                if bdr > half_cone {
                    br = angle + half_cone;
                }

                // Center
                vertices.push(x);
                vertices.push(y);

                // First outer point
                let (ax, ay) = (x + ar.cos() * r, y + ar.sin() * r);
                vertices.push(ax);
                vertices.push(ay);

                // Second outer point
                let (bx, by) = (x + br.cos() * r, y + br.sin() * r);
                vertices.push(bx);
                vertices.push(by);

            }

        }

        vertices

    }

}


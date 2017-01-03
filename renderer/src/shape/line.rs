// STD Dependencies -----------------------------------------------------------
use std::f32::consts;


// External Dependencies ------------------------------------------------------
use graphics::Context;


// Internal Dependencies ------------------------------------------------------
use ::Renderer;


// Line -----------------------------------------------------------------------
#[derive(Debug)]
pub struct Line {
    pub aabb: [f32; 4],
    vertices: [f32; 12]
}

impl Line {

    // TODO draw these with an actual line buffer and shader instead
    pub fn new(points: &[f32; 4], width: f32) -> Line {
        Line {
            aabb: [
                points[0].min(points[2]),
                points[1].min(points[3]),
                points[0].max(points[2]),
                points[1].max(points[3])
            ],
            vertices: Line::vertices(points, width)
        }
    }

    pub fn render(&self, renderer: &mut Renderer, context: &Context) {
        renderer.draw_triangle_list(&context.transform, &self.vertices);
    }

    pub fn vertices(p: &[f32; 4], width: f32) -> [f32; 12] {

        let (dx, dy) = (p[0] - p[2], p[1] - p[3]);
        let pr = dy.atan2(dx) - consts::PI * 0.5;

        // |^
        let (ax, ay) = (p[0] + pr.cos() * width, p[1] + pr.sin() * width);

        // ^|
        let (bx, by) = (p[0] - pr.cos() * width, p[1] - pr.sin() * width);

        // _|
        let (cx, cy) = (p[2] + pr.cos() * width, p[3] + pr.sin() * width);

        // |_
        let (dx, dy) = (p[2] - pr.cos() * width, p[3] - pr.sin() * width);

        [

            // A B C
            ax, ay,
            bx, by,
            cx, cy,

            // A C D
            bx, by,
            dx, dy,
            cx, cy

        ]

    }

}


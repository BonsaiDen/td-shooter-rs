// STD Dependencies -----------------------------------------------------------
use std::f32::consts;


// External Dependencies ------------------------------------------------------
use graphics::Context;


// Internal Dependencies ------------------------------------------------------
use ::renderer::Renderer;


// Shapes with cached Vertices ------------------------------------------------
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

#[derive(Debug)]
pub struct Line {
    vertices: [f32; 12]
}

impl Line {

    pub fn new(points: &[f32; 4], width: f32) -> Line {
        Line {
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


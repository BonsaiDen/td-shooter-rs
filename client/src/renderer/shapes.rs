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

// Helpers --------------------------------------------------------------------
fn triangulate(mut points: Vec<[f32; 2]>) -> Vec<f32> {

    // TODO reverse if clockwise
    if is_clockwise(&points) {
        points = points.into_iter().rev().collect();
    }

    // Triangulate anti-clockwise polygons.
    let mut triangles = Vec::new();
    while points.len() >= 3 {
        if let Some(triangle) = get_ear(&mut points) {
            triangles.extend_from_slice(&triangle);
        }
    }

    triangles

}

fn get_ear(points: &mut Vec<[f32; 2]>) -> Option<[f32; 6]> {

    let size = points.len() as isize;
    if size < 3 {
        None

    } else if size == 3 {
        let triangle = [
             points[0][0], points[0][1],
             points[1][0], points[1][1],
             points[2][0], points[2][1]
        ];
        points.clear();
        Some(triangle)

    } else {

        for i in 0..size {

            let triangle = {

                let mut remove = true;

                let (p1, p2, p3) = (
                    &points[((i - 1 + size) % size) as usize],
                    &points[i as usize],
                    &points[((i + 1 + size) % size) as usize]
                );

                if is_convex(p1, p2, p3) {
                    for x in points.iter() {
                        if (x != p1 && x != p2 && x != p3) && in_triangle(p1, p2, p3, x) {
                            remove = false;
                            break;
                        }
                    }

                    if remove {
                        Some([p1[0], p1[1], p2[0], p2[1], p3[0], p3[1]])

                    } else {
                        None
                    }

                } else {
                    None
                }

            };

            if let Some(triangle) = triangle {
                points.remove(i as usize);
                return Some(triangle);
            }

        }

        None

    }

}

fn is_convex(a: &[f32; 2], b: &[f32; 2], c: &[f32; 2]) -> bool {
    let cross = (b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0]);
    cross >= 0.0
}


fn in_triangle(a: &[f32; 2], b: &[f32; 2], c: &[f32; 2], p: &[f32; 2]) -> bool {

    // Calculates the barycentric coefficients for point p.
    let eps = 0.0000001;

	let e = ((b[1] - c[1]) * (p[0] - c[0]) + (c[0] - b[0]) * (p[1] - c[1]))
		  / (((b[1] - c[1]) * (a[0] - c[0]) + (c[0] - b[0]) * (a[1] - c[1])) + eps);

    if e >= 1.0 || e <= 0.0 {
        return false
    }

	let f = ((c[1] - a[1]) * (p[0] - c[0]) + (a[0] - c[0]) * (p[1] - c[1]))
		  / (((b[1] - c[1]) * (a[0] - c[0]) + (c[0] - b[0]) * (a[1] - c[1])) + eps);

    if f >= 1.0 || f <= 0.0 {
        return false
    }

	let g = 1.0 - e - f;
    !(g >= 1.0 || g <= 0.0)

}

fn is_clockwise(points: &[[f32; 2]]) -> bool {

    let size = points.len();
	let mut sum = (points[0][0] - points[size - 1][0]) * (points[0][1] + points[size - 1][1]);

    for i in 0..size - 1 {
        sum += (points[i + 1][0] - points[i][0]) * (points[i + 1][1] + points[i][1]);
    }

    sum > 0.0

}


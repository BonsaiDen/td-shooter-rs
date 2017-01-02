// Geometry utilities ---------------------------------------------------------
pub fn triangulate(mut points: Vec<[f32; 2]>) -> Vec<f32> {

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


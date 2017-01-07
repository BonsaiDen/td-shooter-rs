// 2D Collision ---------------------------------------------------------------
#[inline(always)]
pub fn aabb_intersect_circle(aabb: &[f32; 4], x: f32, y: f32, r: f32) -> bool {

    // Find closest point
    let nx = aabb[0].max(x.min(aabb[2]));
    let ny = aabb[1].max(y.min(aabb[3]));

    // Circle / AABB center distance
    let (dx, dy) = (nx - x, ny - y);

    // Edge of circle lies within aabb
    (dx * dx + dy * dy) < r * r

}

pub fn line_intersect_circle(line: &[f32; 4], cx: f32, cy: f32, r: f32) -> Option<[f32; 8]> {

    let (ax, ay) = (line[0], line[1]);
    let (bx, by) = (line[2], line[3]);

    // compute the euclidean distance between A and B
    let lab = ((bx - ax).powf(2.0) + (by - ay).powf(2.0)).sqrt();

    // compute the direction vector D from A to B
    let (dx, dy) = ((bx - ax) / lab, (by - ay) / lab);

    // compute the value t of the closest point to the circle center (Cx, Cy)
    let t = dx * (cx - ax) + dy * (cy - ay);

    // compute the coordinates of the point E on line and closest to C
    let (ex, ey) = (t * dx + ax, t * dy + ay);

    // compute the euclidean distance from E to C
    let lec = ((ex - cx).powf(2.0) + (ey - cy).powf(2.0)).sqrt();

    // test if the line intersects the circle
    if lec < r {

        // compute distance from t to circle intersection point
        let dt = (r * r - lec * lec).sqrt();

        // compute first intersection point
        let (fx, fy) = ((t - dt).max(0.0) * dx + ax, (t - dt).max(0.0) * dy + ay);

        // compute second intersection point
        let (gx, gy) = ((t + dt).min(lab) * dx + ax, (t + dt).min(lab) * dy + ay);

        // projected end of intersection line
        let (hx, hy) = (fx + (gx - fx) * 0.5, fy + (gy - fy) * 0.5);

        // Overlap
        let (ox, oy) = (hx - cx, hy - cy);
        let o = r - (ox * ox + oy * oy).sqrt();

        Some([fx, fy, gx, gy, hx, hy, o, oy.atan2(ox)])

    } else {
        None
    }

}

pub fn line_segment_intersect_circle(line: &[f32; 4], cx: f32, cy: f32, r: f32) -> Option<[f32; 8]> {

    let (ax, ay) = (line[0], line[1]);
    let (bx, by) = (line[2], line[3]);
    let (dx, dy) = (bx - ax, by - ay);

    let a = dx * dx + dy * dy;
    let b = 2.0 * (dx * (ax - cx) + dy * (ay - cy));

    let c = (ax - cx) * (ax - cx) + (ay - cy) * (ay - cy) - r * r;
    let det = b * b - 4.0 * a * c;

    if det >= 0.0 {

        // Compute both intersection points
        let t1 = (-b - det.sqrt()) / (2.0 * a);
        let t2 = (-b + det.sqrt()) / (2.0 * a);

        // Check if one of the points lies within the circle
        if (t1 >= 0.0 && t1 <= 0.0) || (t2 >= 0.0 && t2 <= 1.0) {

            let (gx, gy) = (ax + t1 * dx, ay + t1 * dy);
            let (fx, fy) = (ax + t2 * dx, ay + t2 * dy);

            // projected end of intersection line
            let (hx, hy) = (fx + (gx - fx) * 0.5, fy + (gy - fy) * 0.5);

            // Overlap
            let (ox, oy) = (hx - cx, hy - cy);
            let o = r - (ox * ox + oy * oy).sqrt();

            Some([fx, fy, gx, gy, hx, hy, o, oy.atan2(ox)])

        } else {
            None
        }

    } else {
        None
    }

}

pub fn line_segment_intersect_circle_test(line: &[f32; 4], cx: f32, cy: f32, r: f32) -> bool {

    let (ax, ay) = (line[0], line[1]);
    let (bx, by) = (line[2], line[3]);
    let (dx, dy) = (bx - ax, by - ay);

    let a = dx * dx + dy * dy;
    let b = 2.0 * (dx * (ax - cx) + dy * (ay - cy));

    let c = (ax - cx) * (ax - cx) + (ay - cy) * (ay - cy) - r * r;
    let det = b * b - 4.0 * a * c;

    if det > 0.0 {
        let t = -b / (2.0 * a);
        t >= 0.0 && t <= 1.0

    } else {
        false
    }

}

pub fn line_intersect_line(line: &[f32; 4], other: &[f32; 4]) -> Option<[f32; 3]> {

    let (ax, ay) = ( line[2] -  line[0],  line[3] -  line[1]);
    let (bx, by) = (other[2] - other[0], other[3] - other[1]);
    let (cx, cy) = ( line[0] - other[0],  line[1] - other[1]);

    let d = ax * by - bx * ay;
    if d != 0.0 {

        let s = ax * cy - cx * ay;
        if s <= 0.0 && d < 0.0 && s >= d || s >= 0.0 && d > 0.0 && s <= d {

            let t = bx * cy - cx * by;
            if t <= 0.0 && d < 0.0 && t > d || t >= 0.0 && d > 0.0 && t < d {

                let t = t / d;
                let dx = line[0] + t * ax;
                let dy = line[1] + t * ay;
                let (ex, ey) = (line[0] - dx, line[1] - dy);

                return Some([dx, dy, (ex * ex + ey * ey).sqrt()]);

            }

        }

    }

    None

}


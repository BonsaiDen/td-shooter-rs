#[derive(Debug)]
pub struct LevelWall {
    pub points: [f32; 4],
    pub collision: [f32; 4],
    pub aabb: [f32; 4],
    pub is_vertical: bool,
    pub is_horizontal: bool,
    length: f32
}

impl LevelWall {

    pub fn new(a: f32, b: f32, c: f32, d: f32) -> LevelWall {

        // Shorten edges for less collision glitches
        let (dx, dy) = (a - c, b - d);
        let l = (dx * dx + dy * dy).sqrt();
        let r = dy.atan2(dx);

        let (cx, cy) = (a - r.cos() * l * 0.5, b - r.sin() * l * 0.5);
        let (ax, ay) = (cx + r.cos() * (l * 0.5 - 0.5), cy + r.sin() * (l * 0.5 - 0.5));
        let (bx, by) = (cx - r.cos() * (l * 0.5 - 0.5), cy - r.sin() * (l * 0.5 - 0.5));

        LevelWall {
            points: [a, b, c, d],
            collision: [ax, ay, bx, by],
            aabb: [a.min(c), b.min(d), a.max(c), b.max(d)],
            is_vertical: a == c,
            is_horizontal: b == d,
            length: (dx * dx + dy * dy)
        }

    }

    pub fn distance_from_point(&self, x: f32, y: f32) -> f32 {

        let (vx, vy) = (self.points[0], self.points[1]);
        let (wx, wy) = (self.points[2], self.points[3]);

        let t = ((x - vx) * (wx - vx) + (y - vy) * (wy - vy)) / self.length;
        let t = t.max(0.0).min(1.0);

        let (ox, oy) = (vx + t * (wx - vx), vy + t * (wy - vy));
        let (dx, dy) = (x - ox, y - oy);
        (dx * dx + dy * dy).sqrt()

    }

}


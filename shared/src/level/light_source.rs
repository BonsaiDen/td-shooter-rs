// Light Source ---------------------------------------------------------------
#[derive(Debug)]
pub struct LightSource {
    pub x: f32,
    pub y: f32,
    pub radius: f32,
    pub aabb: [f32; 4]
}

impl LightSource {

    pub fn new(x: f32, y: f32, radius: f32) -> LightSource {
        LightSource {
            x: x,
            y: y,
            radius: radius,
            aabb: [x - radius, y - radius, x + radius, y + radius]
        }
    }

    pub fn circle_intersect(&self, x: f32, y: f32, radius: f32) -> bool {
        let (dx, dy) = (self.x - x, self.y - y);
        let d = (dx * dx + dy * dy).sqrt();
        d < self.radius * 0.95 + radius
    }

}


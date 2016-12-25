// External Dependencies ------------------------------------------------------
use piston_window::RenderArgs;


// Client Camera Abstraction --------------------------------------------------
pub struct Camera {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    ratio: f64,
    center: (f64, f64),
    base_width: f64,
    base_height: f64
}

impl Camera {

    pub fn new(base_width: f64, base_height: f64) -> Camera {
        Camera {
            x: 0.0,
            y: 0.0,
            z: 1.0,
            ratio: 1.0,
            center: (0.0, 0.0),
            base_width: base_width,
            base_height: base_height
        }
    }

    pub fn update(&mut self, args: &RenderArgs) {

        let h_ratio = 1.0 / self.base_width * args.draw_width as f64;
        let v_ratio = 1.0 / self.base_height * args.draw_height as f64;

        // TODO stepped pertange zoom here? (25%, 50%, 100%, 200%)
        self.ratio = h_ratio.min(v_ratio) * (1.0 + self.z);
        self.center = (
            args.width as f64 * 0.5,
            args.height as f64 * 0.5
        );

    }

    pub fn s2s(&self, scalar: f64) -> f64 {
        scalar * self.ratio
    }

    pub fn w2s(&self, points: [f64; 4]) -> [f64; 4] {
        [
            self.center.0 + (points[0] - self.x) * self.ratio,
            self.center.1 + (points[1] - self.y) * self.ratio,
            points[2] * self.ratio,
            points[3] * self.ratio
        ]
    }

    pub fn w2p(&self, points: [f64; 4]) -> [f64; 4] {
        [
            self.center.0 + (points[0] - self.x) * self.ratio,
            self.center.1 + (points[1] - self.y) * self.ratio,
            self.center.0 + (points[2] - self.x) * self.ratio,
            self.center.1 + (points[3] - self.y) * self.ratio,
        ]
    }

    pub fn s2w(&self, points: (f64, f64)) -> (f64, f64) {
        let divisor = 1.0 / self.ratio;
        (
            (points.0 - self.center.0) * divisor + self.x,
            (points.1 - self.center.1) * divisor + self.y
        )
    }

}


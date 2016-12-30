// External Dependencies ------------------------------------------------------
use piston::input::RenderArgs;
use graphics::{Context, Transformed};


// Client Camera Abstraction --------------------------------------------------
pub struct Camera {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    ratio: f64,
    center: (f64, f64),
    base_width: f64,
    base_height: f64,
    draw_width: f64,
    draw_height: f64
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
            base_height: base_height,
            draw_width: base_width,
            draw_height: base_height
        }
    }

    pub fn update(&mut self, args: &RenderArgs) {

        self.draw_width = args.draw_width as f64;
        self.draw_height = args.draw_height as f64;

        let h_ratio = 1.0 / self.base_width * self.draw_width;
        let v_ratio = 1.0 / self.base_height * self.draw_height;

        // TODO stepped pertange zoom here? (25%, 50%, 100%, 200%)
        self.ratio = h_ratio.min(v_ratio) * (1.0 + self.z);
        self.center = (
            args.width as f64 * 0.5,
            args.height as f64 * 0.5
        );

    }

    pub fn apply(&self, c: Context) -> Context {
        c.trans(self.center.0, self.center.1).scale(self.ratio, self.ratio).trans(-self.x, -self.y)
    }

    pub fn s2w(&self, x: f64, y: f64) -> (f64, f64) {
        let divisor = 1.0 / self.ratio;
        (
            (x - self.center.0) * divisor + self.x,
            (y - self.center.1) * divisor + self.y
        )
    }

    pub fn b2w(&self) -> [f64; 4] {
        let top_left = self.s2w(0.0, 0.0);
        let bottom_right = self.s2w(self.draw_width, self.draw_height);
        [top_left.0, top_left.1, bottom_right.0, bottom_right.1]
    }

}


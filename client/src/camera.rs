// External Dependencies ------------------------------------------------------
use graphics::{Context, Transformed};


// Internal Dependencies ------------------------------------------------------
use ::renderer::Renderer;


// Client Camera Abstraction --------------------------------------------------
pub struct Camera {
    x: f32,
    y: f32,
    pub z: f32,
    ratio: f32,
    center: (f32, f32),
    context: Context,
    base_width: f32,
    base_height: f32,
    draw_width: f32,
    draw_height: f32,
    world_bounds: [f32; 4]
}

impl Camera {

    pub fn new(base_width: u32, base_height: u32) -> Camera {
        Camera {
            x: 0.0,
            y: 0.0,
            z: 1.5, // default is 1.5
            ratio: 1.0,
            center: (0.0, 0.0),
            context: Context::new(),
            base_width: base_width as f32,
            base_height: base_height as f32,
            draw_width: base_width as f32,
            draw_height: base_height as f32,
            world_bounds: [0f32; 4]
        }
    }

    pub fn apply(&mut self, renderer: &mut Renderer) {

        self.draw_width = renderer.width();
        self.draw_height = renderer.height();

        let h_ratio = 1.0 / self.base_width * self.draw_width;
        let v_ratio = 1.0 / self.base_height * self.draw_height;

        self.ratio = h_ratio.min(v_ratio) * (1.0 + self.z);
        self.center = (
            renderer.width() * 0.5,
            renderer.height() * 0.5
        );

        let top_left = self.s2w(0.0, 0.0);
        let bottom_right = self.s2w(self.draw_width, self.draw_height);

        self.world_bounds = [
            top_left.0,
            top_left.1,
            bottom_right.0,
            bottom_right.1
        ];

        self.context = renderer.context().trans(
            self.center.0 as f64,
            self.center.1 as f64

        ).scale(
            self.ratio as f64,
            self.ratio as f64

        ).trans(-self.x as f64, -self.y as f64);

    }

    pub fn context(&self) -> &Context {
        &self.context
    }

    pub fn center(&mut self, x: f32, y: f32) {
        self.x = x;
        self.y = y;
    }

    pub fn limit(&mut self, bounds: &[f32; 4]) {
        self.x = self.x.max(bounds[0]).min(bounds[2]);
        self.y = self.y.max(bounds[1]).min(bounds[3]);
    }

    pub fn s2w(&self, x: f32, y: f32) -> (f32, f32) {
        let divisor = 1.0 / self.ratio;
        (
            (x - self.center.0) * divisor + self.x,
            (y - self.center.1) * divisor + self.y
        )
    }

    pub fn b2w(&self) -> &[f32; 4] {
        &self.world_bounds
    }

}


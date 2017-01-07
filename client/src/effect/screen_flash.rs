// External Dependencies ------------------------------------------------------
use clock_ticks;


// Internal Dependencies ------------------------------------------------------
use ::effect::Effect;
use ::camera::Camera;
use ::renderer::Renderer;
use shared::color::{Color, ColorName};


// Screen Flash Effect --------------------------------------------------------
pub struct ScreenFlash {
    color: [f32; 4],
    start: u64,
    duration: u64
}

impl ScreenFlash {

    pub fn new(color: ColorName, duration: u64) -> ScreenFlash {
        ScreenFlash {
            color: Color::from_name(color).into_f32(),
            start: clock_ticks::precise_time_ms(),
            duration: duration
        }
    }

}

impl Effect for ScreenFlash {

    fn alive(&self, t: u64) -> bool {
        t < self.start + self.duration
    }

    fn render(&self, renderer: &mut Renderer, _: &Camera) {

        let context = renderer.context().clone();
        let exp = renderer.t() - self.start;
        let u = 1.0 - ((1.0 / (self.duration) as f32) * exp as f32).min(1.0).max(0.0);

        renderer.set_color([
            self.color[0],
            self.color[1],
            self.color[2],
            (u * u) * 0.5
        ]);

        let (w, h) = (renderer.width(), renderer.height());
        renderer.rectangle(&context, &[0.0, 0.0,   w, h]);

    }

}


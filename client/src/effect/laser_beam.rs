// STD Dependencies -----------------------------------------------------------
use std::f32::consts;


// External Dependencies ------------------------------------------------------
use clock_ticks;


// Internal Dependencies ------------------------------------------------------
use ::effect::Effect;
use ::camera::Camera;
use ::renderer::Renderer;
use shared::color::{Color, ColorName};


// Laser Effect ---------------------------------------------------------------
pub struct LaserBeam {
    pub line: [f32; 4],
    color_dark: [f32; 4],
    color_light: [f32; 4],
    start: u64,
    duration: u64
}

impl LaserBeam {

    pub fn new(color: ColorName, line: [f32; 4], duration: u64) -> LaserBeam {
        LaserBeam {
            line: line,
            color_dark: Color::from_name(color).darken(0.5).into_f32(),
            color_light: Color::from_name(color).into_f32(),
            start: clock_ticks::precise_time_ms(),
            duration: duration
        }
    }

    pub fn from_point(color: ColorName, x: f32, y: f32, r: f32, d: f32, l: f32, duration: u64) -> LaserBeam {
        LaserBeam::new(color, [
            x + r.cos() * d, y + r.sin() * d,
            x + r.cos() * (d + l), y + r.sin() * (d + l)

        ], duration)
    }

}

impl Effect for LaserBeam {

    fn alive(&self, t: u64) -> bool {
        t < self.start + self.duration
    }

    fn render(&self, renderer: &mut Renderer, camera: &Camera) {

        let context = camera.context();
        let exp = renderer.t() - self.start;
        let u = ((1.0 / self.duration as f32) * exp as f32).min(1.0).max(0.0);
        let a = 0.35 + u * 0.5;

        renderer.set_color([
            self.color_dark[0],
            self.color_dark[1],
            self.color_dark[2],
            1.0 - a
        ]);
        renderer.line(context, &self.line, u.sin() * 4.0);

        renderer.set_color([
            self.color_light[0],
            self.color_light[1],
            self.color_light[2],
            1.0 - a
        ]);
        renderer.line(context, &self.line, (u * consts::PI).sin() * 0.85)

    }

}


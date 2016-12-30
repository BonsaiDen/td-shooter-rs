// STD Dependencies -----------------------------------------------------------
use std::f64::consts;


// External Dependencies ------------------------------------------------------
use clock_ticks;
use graphics::Context;


// Internal Dependencies ------------------------------------------------------
use ::effect::Effect;
use shared::color::{Color, ColorName};


// Laser Effect ---------------------------------------------------------------
pub struct LaserBeam {
    pub line: [f64; 4],
    color_dark: [f32; 4],
    color_light: [f32; 4],
    start: u64,
    duration: u64
}

impl LaserBeam {

    pub fn new(color: ColorName, line: [f64; 4], duration: u64) -> LaserBeam {
        LaserBeam {
            line: line,
            color_dark: Color::from_name(color).darken(0.5).into_f32(),
            color_light: Color::from_name(color).into_f32(),
            start: clock_ticks::precise_time_ms(),
            duration: duration
        }
    }

    pub fn from_point(color: ColorName, x: f64, y: f64, r: f64, d: f64, l: f64, duration: u64) -> LaserBeam {
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

    fn draw_2d(
        &self,
        c: Context,
        //g: &mut G2d,
        t: u64
    ) {

        let exp = t - self.start;
        let u = ((1.0 / self.duration as f64) * exp as f64).min(1.0).max(0.0);
        let a = (0.35 + u * 0.5) as f32;

        // TODO re-enable
        //line(
        //    [self.color_dark[0], self.color_dark[1], self.color_dark[2], 1.0 - a],
        //    u.sin() * 4.0, self.line, c.transform, g
        //);

        //line(
        //    [self.color_light[0], self.color_light[1], self.color_light[2], a],
        //    (u * consts::PI).sin() * 0.75, self.line, c.transform, g
        //);

    }

}


// STD Dependencies -----------------------------------------------------------
use std::f32::consts;


// External Dependencies ------------------------------------------------------
use clock_ticks;


// Internal Dependencies ------------------------------------------------------
use ::camera::Camera;
use ::renderer::Renderer;
use ::effect::{Effect, ParticleSystem, particle};
use shared::action::LASER_BEAM_DURATION;
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

    pub fn new(color: ColorName, line: [f32; 4]) -> LaserBeam {
        LaserBeam {
            line: line,
            color_dark: Color::from_name(color).darken(0.5).into_f32(),
            color_light: Color::from_name(color).into_f32(),
            start: clock_ticks::precise_time_ms(),
            duration: LASER_BEAM_DURATION
        }
    }

    pub fn from_point(
        ps: &mut ParticleSystem,
        color: ColorName,
        x: f32, y: f32,
        r: f32, d: f32,
        l: f32,
        wall_angle: Option<f32>

    ) -> LaserBeam {

        // Particles along the beam
        particle::line(ps, color, x, y, r, l, 4.0);

        // Particles ejected from a wall impact
        if let Some(wr) = wall_angle {
            particle::impact(ps, color, x, y, r, wr, l, 15);
        }

        LaserBeam::new(color, [
            x + r.cos() * d, y + r.sin() * d,
            x + r.cos() * (d + l), y + r.sin() * (d + l)
        ])

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

        // Wide background beam
        renderer.set_color([
            self.color_dark[0],
            self.color_dark[1],
            self.color_dark[2],
            1.0 - a
        ]);
        renderer.line(context, &self.line, u.sin() * 3.0);

        // Focussed beam in the middle
        renderer.set_color([
            self.color_light[0],
            self.color_light[1],
            self.color_light[2],
            1.0 - a
        ]);
        renderer.line(context, &self.line, (u * consts::PI).sin() * 0.7)

    }

}


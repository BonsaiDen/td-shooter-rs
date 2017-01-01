// STD Dependencies -----------------------------------------------------------
use std::f32::consts;


// External Dependencies ------------------------------------------------------
use rand;
use rand::Rng;
use clock_ticks;


// Internal Dependencies ------------------------------------------------------
use ::effect::Effect;
use ::camera::Camera;
use ::renderer::Renderer;
use ::particle_system::ParticleSystem;
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
            duration: 300
        }
    }

    pub fn from_point(
        particle_system: &mut ParticleSystem,
        color_name: ColorName,
        x: f32, y: f32,
        r: f32, d: f32,
        l: f32

    ) -> LaserBeam {

        // TODO have a small sparkle / rotation / start effect at the source of the beam

        // Spawn particles along beam path
        let step = 4.0;
        let count = (l / step).floor() as usize;
        let particle_color = Color::from_name(color_name).into_f32();

        for i in 0..count {
            if let Some(p) = particle_system.get() {

                let a = rand::thread_rng().gen::<f32>();
                let b = rand::thread_rng().gen::<f32>() + 0.5;
                let c = rand::thread_rng().gen::<f32>() - 0.5;

                let o = i as f32 * step + step * 0.5;
                p.color = particle_color;
                p.x = x + r.cos() * o + c * 2.5;
                p.y = y + r.sin() * o + c * 2.5;
                p.direction = a * consts::PI * 2.0;
                p.size = 3.0 * b;
                p.size_ms = -1.0 * b;
                p.velocity = 3.0 * b;
                p.lifetime = (0.75 + 1.5 * a) * 0.8;
                p.remaining = p.lifetime;

            }
        }

        LaserBeam::new(color_name, [
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
        renderer.line(context, &self.line, u.sin() * 4.0);

        // Focussed beam in the middle
        renderer.set_color([
            self.color_light[0],
            self.color_light[1],
            self.color_light[2],
            1.0 - a
        ]);
        renderer.line(context, &self.line, (u * consts::PI).sin() * 0.85)

    }

}


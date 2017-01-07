// STD Dependencies -----------------------------------------------------------
use std::f32::consts;


// External Dependencies ------------------------------------------------------
use rand;
use rand::Rng;
use clock_ticks;
use graphics::Transformed;


// Internal Dependencies ------------------------------------------------------
use ::effect::Effect;
use ::camera::Camera;
use ::renderer::{Renderer, Circle};
use ::particle_system::ParticleSystem;
use shared::entity::PLAYER_RADIUS;
use shared::color::{Color, ColorName};


// Laser Hit Effect -----------------------------------------------------------
pub struct LaserBeamHit {
    x: f32,
    y: f32,
    color_light: [f32; 4],
    start: u64,
    duration: u64,
    circle: Circle
}

impl LaserBeamHit {

    pub fn new(color: ColorName, x: f32, y: f32, scale: f32) -> LaserBeamHit {
        LaserBeamHit {
            x: x,
            y: y,
            color_light: Color::from_name(color).into_f32(),
            start: clock_ticks::precise_time_ms(),
            duration: (250.0 * scale).round() as u64,
            circle: Circle::new(16, 0.0, 0.0, PLAYER_RADIUS * scale)
        }
    }

    pub fn from_point(
        particle_system: &mut ParticleSystem,
        color_name: ColorName,
        x: f32, y: f32,
        scale: f32

    ) -> LaserBeamHit {

        // TODO factor out into particles module
        let segments = (16.0 * scale).ceil() as usize;
        let step = (consts::PI * 2.0) / segments as f32;
        let particle_color = Color::from_name(color_name).into_f32();

        for i in 0..segments {
            if let Some(p) = particle_system.get() {

                let r = (i as f32) * step;
                let a = rand::thread_rng().gen::<f32>();
                let b = rand::thread_rng().gen::<f32>() + 0.5;
                let c = rand::thread_rng().gen::<f32>() - 0.5;

                p.color = particle_color;
                p.x = x + r.cos() * PLAYER_RADIUS + c * 2.5;
                p.y = y + r.sin() * PLAYER_RADIUS + c * 2.5;
                p.direction = r;
                p.size = 3.0 * b * scale;
                p.size_ms = -1.2 * b * scale;
                p.velocity = 7.5 * b * scale;
                p.lifetime = (0.75 + 1.5 * a) * 0.3 * scale;
                p.remaining = p.lifetime;

            }
        }


        LaserBeamHit::new(color_name, x, y, scale)

    }

}

impl Effect for LaserBeamHit {

    fn alive(&self, t: u64) -> bool {
        t < self.start + self.duration
    }

    fn render(&self, renderer: &mut Renderer, camera: &Camera) {

        let context = camera.context();
        let exp = renderer.t() - self.start;
        let u = ((1.0 / self.duration as f32) * exp as f32).min(1.0).max(0.0);
        let q = context.trans(self.x as f64, self.y as f64).scale(
            1.0 + u as f64 * 1.2,
            1.0 + u as f64 * 1.2
        );

        renderer.set_color([
            self.color_light[0],
            self.color_light[1],
            self.color_light[2],
            (1.0 - u) * 0.3
        ]);

        self.circle.render(renderer, &q);

    }

}


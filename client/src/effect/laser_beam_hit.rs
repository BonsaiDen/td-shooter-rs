// External Dependencies ------------------------------------------------------
use clock_ticks;
use graphics::Transformed;


// Internal Dependencies ------------------------------------------------------
use ::camera::Camera;
use ::renderer::{Renderer, Circle};
use ::effect::{Effect, ParticleSystem, particle};
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
        ps: &mut ParticleSystem,
        color: ColorName,
        x: f32, y: f32,
        scale: f32

    ) -> LaserBeamHit {
        particle::circle(ps, color, x, y, PLAYER_RADIUS, scale, 16);
        LaserBeamHit::new(color, x, y, scale)
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


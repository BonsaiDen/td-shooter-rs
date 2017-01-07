// STD Dependencies -----------------------------------------------------------
use std::cmp;


// External Dependencies ------------------------------------------------------
use graphics::math::Matrix2d;


// Internal Dependencies ------------------------------------------------------
use ::renderer::Renderer;
use ::shared::color::{Color, ColorName};


// ParticleSystem -------------------------------------------------------------
pub struct ParticleSystem {
    first_available_particle: usize,
    max_used_particle: usize,
    particles: Vec<Particle>
}

impl ParticleSystem {

    pub fn new(max_particles: usize) -> ParticleSystem {

        let mut particles = Vec::with_capacity(max_particles);
        for i in 0..max_particles {
            particles.push(Particle {
                active: false,
                color: Color::from_name(ColorName::White).into_f32(),
                x: 0.0,
                y: 0.0,
                size: 10.0,
                size_ms: 0.0,
                velocity: 0.0,
                velocity_ms: 0.0,
                direction: 0.0,
                direction_ms: 0.0,
                fadeout: 0.0,
                lifetime: 0.0,
                remaining: 0.0,
                id: i,
                next_available: i + 1,
            });
        }

        ParticleSystem {
            first_available_particle: 0,
            max_used_particle: 0,
            particles: particles
        }

    }

    pub fn get(&mut self) -> Option<&mut Particle> {

        if let Some(p) = self.particles.get_mut(self.first_available_particle) {
            p.active = true;
            p.x = 0.0;
            p.y = 0.0;
            p.size = 5.0;
            p.size_ms = -2.5;
            p.velocity = 0.0;
            p.velocity_ms = 0.0;
            p.direction = 0.0;
            p.direction_ms = 0.0;
            p.fadeout = 0.25;
            p.lifetime = 0.8;
            p.remaining = 0.8;
            self.first_available_particle = p.next_available;
            self.max_used_particle = cmp::max(self.max_used_particle, p.id + 1);
            Some(p)

        } else {
            None
        }

    }

    pub fn render(&mut self, scale: f32, m: &Matrix2d, renderer: &mut Renderer) {

        let mut max_used_particle = 0;
        let mut particle_index = 0;

        let dt = renderer.dt();
        for i in 0..self.max_used_particle {
            let particle = self.particles.get_mut(i).unwrap();
            if particle.is_active() {

                if particle.step(dt) == false {
                    particle.next_available = self.first_available_particle;
                    self.first_available_particle = particle.id;

                } else {

                    let lp = 1.0 / particle.lifetime * particle.remaining;
                    let a = if lp <= particle.fadeout {
                        1.0 / (particle.lifetime * particle.fadeout) * particle.remaining.max(0.0)

                    } else {
                        1.0
                    };

                    particle.color[3] = a;
                    renderer.add_particle(
                        scale, particle_index,
                        particle.x, particle.y,
                        particle.size,
                        &particle.color
                    );

                    particle_index += 1;
                    max_used_particle = cmp::max(
                        particle.id + 1,
                        max_used_particle
                    );

                }

            }
        }

        self.max_used_particle = max_used_particle;

        renderer.render_particles(m, particle_index);

    }

}


// Particle -------------------------------------------------------------------
pub struct Particle {

    active: bool,

    pub color: [f32; 4],

    // Position
    pub x: f32,
    pub y: f32,

    // Size
    pub size: f32,

    // Size modification per second
    pub size_ms: f32,

    // Velocity
    pub velocity: f32,

    // Velocity modification per seond
    pub velocity_ms: f32,

    // Direction Angle
    pub direction: f32,

    // Direction Angle modification per second
    pub direction_ms: f32,

    pub fadeout: f32,
    pub lifetime: f32,
    pub remaining: f32,

    pub id: usize,
    pub next_available: usize

}

impl Particle {

    fn is_active(&mut self) -> bool {
        self.active
    }

    fn step(&mut self, dt: f32) -> bool {
        if self.remaining <= 0.0 {
            self.active = false;
            false

        } else {
            self.x += self.direction.cos() * self.velocity * dt;
            self.y += self.direction.sin() * self.velocity * dt;
            self.size += self.size_ms * dt;
            self.direction += self.direction_ms * dt;
            self.velocity += self.velocity_ms * dt;
            self.remaining -= dt;
            true
        }
    }

}


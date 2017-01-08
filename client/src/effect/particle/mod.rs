// STD Dependencies -----------------------------------------------------------
use std::f32::consts;


// External Dependencies ------------------------------------------------------
use rand;
use rand::Rng;


// Internal Dependencies ------------------------------------------------------
use ::effect::ParticleSystem;
use shared::color::{Color, ColorName};


// Particle Helpers -----------------------------------------------------------
pub fn line(
    ps: &mut ParticleSystem,
    color: ColorName,
    x: f32, y: f32, r: f32, l: f32,
    step: f32
) {

    let count = (l / step).floor().max(0.0) as usize;
    let particle_color = Color::from_name(color).into_f32();

    for i in 0..count {
        if let Some(p) = ps.get() {

            let a = rand::thread_rng().gen::<f32>();
            let b = rand::thread_rng().gen::<f32>() + 0.5;
            let c = rand::thread_rng().gen::<f32>() - 0.5;

            let o = i as f32 * step + step * 0.5;
            p.color = particle_color;
            p.x = x + r.cos() * o + c * 2.5;
            p.y = y + r.sin() * o + c * 2.5;
            p.direction = a * consts::PI * 2.0;
            p.size = 3.0 * b;
            p.size_ms = -0.5 + -1.0 * b;
            p.velocity = 3.5 * b;
            p.lifetime = (0.75 + 1.5 * a) * 0.8;
            p.remaining = p.lifetime;

        }
    }

}

pub fn impact(
    ps: &mut ParticleSystem,
    color: ColorName,
    x: f32, y: f32, r: f32, wr: f32, l: f32,
    count: usize
) {

    let particle_color = Color::from_name(color).into_f32();

    for _ in 0..count {

        if let Some(p) = ps.get() {

            let a = rand::thread_rng().gen::<f32>();
            let b = rand::thread_rng().gen::<f32>() + 0.5;
            let c = rand::thread_rng().gen::<f32>() - 0.5;

            let ir = wr - (consts::PI * 0.4 * c) - consts::PI;
            p.color = particle_color;
            p.x = x + r.cos() * l;
            p.y = y + r.sin() * l;
            p.direction = ir;
            p.size = 4.5 * b;
            p.size_ms = -3.5 * b;
            p.velocity = 20.0 + 11.0 * b;
            p.lifetime = (0.75 + 1.5 * a) * 0.2;
            p.remaining = p.lifetime;

        }

    }

}

pub fn circle(
    ps: &mut ParticleSystem,
    color: ColorName,
    x: f32, y: f32,
    radius: f32,
    scale: f32,
    segments: usize
) {

    let segments = (segments as f32 * scale).ceil() as usize;
    let step = (consts::PI * 2.0) / segments as f32;
    let particle_color = Color::from_name(color).into_f32();

    for i in 0..segments {
        if let Some(p) = ps.get() {

            let r = (i as f32) * step;
            let a = rand::thread_rng().gen::<f32>();
            let b = rand::thread_rng().gen::<f32>() + 0.5;
            let c = rand::thread_rng().gen::<f32>() - 0.5;

            p.color = particle_color;
            p.x = x + r.cos() * radius + c * 2.5;
            p.y = y + r.sin() * radius + c * 2.5;
            p.direction = r;
            p.size = 4.0 * b * scale;
            p.size_ms = -1.0 + -2.5 * b * scale;
            p.velocity = 3.5 + 8.5 * b * scale;
            p.lifetime = (0.75 + 1.5 * a) * 0.3 * scale;
            p.remaining = p.lifetime;

        }
    }

}


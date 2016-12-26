// STD Dependencies -----------------------------------------------------------
use std::f32::consts;


// External Dependencies ------------------------------------------------------
use netsync::NetworkProperty;
use bincode::SizeLimit;
use bincode::rustc_serialize::{encode, decode};


// Internal Dependencies ------------------------------------------------------
use super::{PlayerInput, PLAYER_SPEED, PLAYER_RADIUS};
use ::level::LevelCollision;


// Statics --------------------------------------------------------------------
const TAU: f32 = consts::PI * 2.0;


// Player Network Position ----------------------------------------------------
#[derive(Debug, Clone, Default, RustcEncodable, RustcDecodable)]
pub struct PlayerPosition {
    pub x: f32,
    pub y: f32,
    pub r: f32
}

impl NetworkProperty for PlayerPosition {

    fn interpolate_from(&self, last: &Self, u: f64) -> Self {
        let dx = self.x - last.x;
        let dy = self.y - last.y;
        let r = self.r - last.r;
        let dr = r.sin().atan2(r.cos());

        PlayerPosition {
            x: last.x + (dx * u as f32),
            y: last.y + (dy * u as f32),
            r: last.r + (dr * u as f32)
        }
    }

    fn to_bytes(&self) -> Vec<u8> {
        encode(&PlayerNetworkPosition(
            self.x,
            self.y,
            ((self.r + consts::PI) * 2000.0).round() as u16

        ), SizeLimit::Infinite).unwrap()
    }

    fn from_bytes(bytes: &[u8]) -> Self where Self: Sized {
        let position = decode::<PlayerNetworkPosition>(bytes).unwrap();
        PlayerPosition {
            x: position.0,
            y: position.1,
            r: (position.2 as f32) / 2000.0 - consts::PI
        }
    }

}

impl PlayerPosition {

    pub fn update<L: LevelCollision>(dt: f64, state: &mut PlayerPosition, input: &PlayerInput, level: &L) {

        let (mut dx, mut dy) = (0.0, 0.0);
        if input.buttons & 1 == 1 {
            dy -= PLAYER_SPEED;
        }

        if input.buttons & 2 == 2 {
            dx += PLAYER_SPEED;
        }

        if input.buttons & 4 == 4 {
            dy += PLAYER_SPEED;
        }

        if input.buttons & 8 == 8 {
            dx -= PLAYER_SPEED;
        }

        // Limit diagonal speed
        let r = dy.atan2(dx);
        let dist = ((dx * dx) + (dy * dy)).sqrt();
        state.x += (r.cos() * dist.min(PLAYER_SPEED * dt)) as f32;
        state.y += (r.sin() * dist.min(PLAYER_SPEED * dt)) as f32;

        // Limit rotation speed
        let r = input.r - state.r;
        let dr = r.sin().atan2(r.cos());
        state.r += dr.min(consts::PI * 0.125).max(-consts::PI * 0.125);

        // Limit rotation to 0..TAU
        if state.r < 0.0 {
            state.r += TAU;

        } else if state.r > TAU {
            state.r -= TAU;
        }

        // Collision
        level.collide(&mut state.x, &mut state.y, PLAYER_RADIUS);

    }

}

#[derive(RustcEncodable, RustcDecodable)]
struct PlayerNetworkPosition(f32, f32, u16);


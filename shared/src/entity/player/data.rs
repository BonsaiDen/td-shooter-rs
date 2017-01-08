// STD Dependencies -----------------------------------------------------------
use std::f32::consts;


// External Dependencies ------------------------------------------------------
use netsync::NetworkProperty;
use bincode::SizeLimit;
use bincode::rustc_serialize::{encode, decode};


// Internal Dependencies ------------------------------------------------------
use super::{PlayerInput, PLAYER_SPEED, PLAYER_RADIUS};
use ::util::{rad_to_u16, u16_to_rad};
use ::level::LevelCollision;


// Statics --------------------------------------------------------------------
const TAU: f32 = consts::PI * 2.0;


// Player Network Data --------------------------------------------------------
#[derive(Debug, Clone, Default, RustcEncodable, RustcDecodable)]
pub struct PlayerData {
    pub x: f32,
    pub y: f32,
    pub r: f32,
    pub hp: u8,
    pub visible: bool,
    pub vx: f32,
    pub vy: f32
}

impl PlayerData {

    pub fn new(x: f32, y: f32, r: f32, hp: u8) -> PlayerData {
        PlayerData {
            x: x,
            y: y,
            r: r,
            hp: hp,
            visible: false,
            vx: 0.0,
            vy: 0.0
        }
    }

}

impl NetworkProperty for PlayerData {

    fn interpolate_from(&self, last: &Self, u: f32) -> Self {

        // Prevent interpolation glitches when a player entity becomes visible
        // again
        if !last.visible {
            self.clone()

        } else {
            let dx = (self.x - last.x).max(-PLAYER_SPEED * 3.0).min(PLAYER_SPEED * 3.0);
            let dy = (self.y - last.y).max(-PLAYER_SPEED * 3.0).min(PLAYER_SPEED * 3.0);
            let r = self.r - last.r;
            let dr = r.sin().atan2(r.cos());

            PlayerData {
                x: last.x + dx * u,
                y: last.y + dy * u,
                r: last.r + dr * u,
                hp: self.hp,
                visible: self.visible,
                vx: self.vx,
                vy: self.vy
            }
        }

    }

    fn to_bytes(&self) -> Vec<u8> {
        encode(&PlayerNetworkPosition(
            self.visible,
            self.x,
            self.y,
            rad_to_u16(self.r),
            self.hp

        ), SizeLimit::Bounded(12)).unwrap()
    }

    fn from_bytes(bytes: &[u8]) -> Self where Self: Sized {
        let position = decode::<PlayerNetworkPosition>(bytes).unwrap();
        PlayerData {
            x: position.1,
            y: position.2,
            r: u16_to_rad(position.3),
            visible: position.0,
            hp: position.4,
            vx: 0.0,
            vy: 0.0
        }
    }

}

impl PlayerData {

    pub fn update<L: LevelCollision>(dt: f32, state: &mut PlayerData, input: &PlayerInput, level: &L) {

        let (mut dx, mut dy) = (0.0, 0.0);
        let mut speed = PLAYER_SPEED;
        if input.buttons & 16 == 16 {
            speed *= 0.5;
        }

        if input.buttons & 1 == 1 {
            dy -= speed;
        }

        if input.buttons & 2 == 2 {
            dx += speed;
        }

        if input.buttons & 4 == 4 {
            dy += speed;
        }

        if input.buttons & 8 == 8 {
            dx -= speed;
        }

        // Limit diagonal speed
        let r = dy.atan2(dx);
        let dist = ((dx * dx) + (dy * dy)).sqrt();
        state.vx = r.cos() * dist.min(speed * dt);
        state.vy = r.sin() * dist.min(speed * dt);
        state.x += state.vx;
        state.y += state.vy;

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
        level.collide(&mut state.x, &mut state.y, PLAYER_RADIUS, dx != 0.0 || dy != 0.0);

    }

    pub fn update_extrapolated<L: LevelCollision>(state: &mut PlayerData, level: &L) {
        state.x += state.vx;
        state.y += state.vy;
        level.collide(&mut state.x, &mut state.y, PLAYER_RADIUS, true);
    }

    pub fn merge_client_angle(&mut self, client_r: f32) {
        let r = client_r - self.r;
        let dr = r.sin().atan2(r.cos());
        self.r += dr.min(consts::PI * 0.125).max(-consts::PI * 0.125);
    }

}

#[derive(RustcEncodable, RustcDecodable)]
struct PlayerNetworkPosition(bool, f32, f32, u16, u8);


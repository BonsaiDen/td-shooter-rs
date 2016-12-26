// STD Dependencies -----------------------------------------------------------
use std::f32::consts;


// External Dependencies ------------------------------------------------------
use netsync::NetworkInput;
use bincode::SizeLimit;
use bincode::rustc_serialize::{encode, decode};


// Player Network Input -------------------------------------------------------
#[derive(Debug, Default, Clone)]
pub struct PlayerInput {
    pub tick: u8,
    pub buttons: u8,
    pub r: f32,
    pub dt: f32
}

impl PlayerInput {
    pub fn new(tick: u8, buttons: u8, r: f32, dt: f32) -> PlayerInput {
        PlayerInput {
            tick: tick,
            buttons: buttons,
            r: r,
            dt: dt
        }
    }
}

impl NetworkInput for PlayerInput {

    fn tick(&self) -> u8 {
        self.tick
    }

    fn to_bytes(&self) -> Vec<u8> {
        encode(&PlayerNetworkInput(
            self.tick,
            self.buttons,
            ((self.r + consts::PI) * 2000.0).floor() as u16

        ), SizeLimit::Infinite).unwrap()
    }

    fn from_bytes(bytes: &[u8]) -> Option<(usize, Self)> where Self: Sized {
        if bytes.len() >= 4 {
            // TODO get encoded size automatically?
            // encoded_size::<PlayerInput>(&self)
            let input = decode::<PlayerNetworkInput>(bytes).unwrap();
            Some((4, PlayerInput {
                tick: input.0,
                buttons: input.1,
                r: (input.2 as f32) / 2000.0 - consts::PI,
                dt: 0.0
            }))

        } else {
            None
        }
    }

}

#[derive(RustcEncodable, RustcDecodable)]
struct PlayerNetworkInput(u8, u8, u16);


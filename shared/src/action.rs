// External Dependencies ------------------------------------------------------
use cobalt::ConnectionID;
use hexahydrate::NETWORK_BYTE_OFFSET;
use bincode::SizeLimit;
use bincode::rustc_serialize::{encode, decode, DecodingError};


// Internal Dependencies ------------------------------------------------------
use ::entity::PlayerData;


// Statics --------------------------------------------------------------------
pub const LASER_BEAM_DURATION: u64 = 150;


// Network Actions ------------------------------------------------------------
#[derive(Debug, RustcEncodable, RustcDecodable, Clone)]
pub enum Action {
    FiredLaserBeam(u8, f32),
    CreateLaserBeam(u8, f32, f32, f32, f32),
    LaserBeamHit(u8, u8, f32, f32),
    LaserBeamKill(u8, u8, f32, f32)
}

impl Action {

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = vec![NETWORK_BYTE_OFFSET];
        bytes.append(&mut encode(self, SizeLimit::Infinite).unwrap());
        bytes
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Action, DecodingError> {
        if bytes[0] == NETWORK_BYTE_OFFSET {
            decode::<Action>(&bytes[1..])

        } else {
            Err(DecodingError::SizeLimit)
        }
    }

}


// Network action visibility --------------------------------------------------
pub enum ActionVisibility {

    /// Always send to any connection
    Any,

    /// Only when a connection is the one specified by the ConnectionID
    Connection(ConnectionID),

    /// Only when a connection can see the entity owned by the specified ConnectionID.
    /// The second parameter is a optional exclusion of a specific connection.
    Entity(PlayerData, Option<ConnectionID>),

    /// Only when the specified radius around a connection's entity intersects with the specified bounds
    WithinRange {
        aabb: [f32; 4],
        r: f32
    }

}


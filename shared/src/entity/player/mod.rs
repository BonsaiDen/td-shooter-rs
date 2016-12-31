// STD Dependencies -----------------------------------------------------------
use std::f32::consts;


// Modules --------------------------------------------------------------------
mod entity;
mod input;
mod position;


// Statics --------------------------------------------------------------------
pub const PLAYER_SPEED: f32 = 90.0;
pub const PLAYER_RADIUS: f32 = 6.0;
pub const PLAYER_MAX_HP: u8 = 255;
pub const PLAYER_SERVER_VISIBILITY_SCALING: f32 = 1.0;
pub const PLAYER_VISBILITY_CONE: f32 = consts::PI * 0.20;
pub const PLAYER_VISBILITY_CONE_OFFSET: f32 = PLAYER_RADIUS * 3.0;


// Re-Exports -----------------------------------------------------------------
pub use self::entity::PlayerEntity;
pub use self::input::PlayerInput;
pub use self::position::PlayerData;


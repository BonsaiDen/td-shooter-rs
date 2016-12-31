// Modules --------------------------------------------------------------------
mod entity;
mod input;
mod position;


// Statics --------------------------------------------------------------------
pub const PLAYER_SPEED: f32 = 120.0;
pub const PLAYER_RADIUS: f32 = 6.0;
pub const PLAYER_SERVER_VISIBILITY_SCALING: f32 = 1.1;
pub const PLAYER_MAX_HP: u8 = 255;


// Re-Exports -----------------------------------------------------------------
pub use self::entity::PlayerEntity;
pub use self::input::PlayerInput;
pub use self::position::PlayerData;


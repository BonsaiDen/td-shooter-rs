// Modules --------------------------------------------------------------------
mod entity;
mod input;
mod position;


// Statics --------------------------------------------------------------------
pub static PLAYER_SPEED: f64 = 120.0;
pub static PLAYER_RADIUS: f64 = 6.0;


// Re-Exports -----------------------------------------------------------------
pub use self::entity::PlayerEntity;
pub use self::input::PlayerInput;
pub use self::position::PlayerPosition;


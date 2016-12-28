// External Dependencies ------------------------------------------------------
use hexahydrate;
use cobalt::ConnectionID;
use netsync::{ServerState, NetworkState};


// Internal Dependencies ------------------------------------------------------
use ::shared::color::ColorName;
use ::shared::level::Level;
use ::shared::entity::{PlayerInput, PlayerPosition, PlayerEntity};


// Server Entity --------------------------------------------------------------
type ServerPlayerEntity = PlayerEntity<ServerState<PlayerPosition, PlayerInput>>;

pub trait Entity: hexahydrate::Entity<ConnectionID> {
    fn update(&mut self, dt: f64, level: &Level);
    fn position(&self, tick: u8) -> PlayerPosition;
    fn color_name(&self) -> ColorName;
}

impl Entity for ServerPlayerEntity {

    fn update(&mut self, dt: f64, level: &Level) {
        self.state.update_with(|state, input| {
            PlayerPosition::update(dt, state, input, level);
        });
    }

    fn position(&self, tick: u8) -> PlayerPosition {
        self.state.get_absolute(tick)
    }

    fn color_name(&self) -> ColorName {
        self.color
    }

}


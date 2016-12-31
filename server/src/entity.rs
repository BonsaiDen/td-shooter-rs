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
    fn owner(&self) -> Option<ConnectionID>;
    fn update(&mut self, dt: f32, level: &Level);
    fn position(&self, tick: u8) -> PlayerPosition;
    fn current_position(&self) -> PlayerPosition;
    fn color_name(&self) -> ColorName;
    fn set_visibility(&mut self, ConnectionID, bool);
}

impl Entity for ServerPlayerEntity {

    fn owner(&self) -> Option<ConnectionID> {
        self.owner
    }

    fn update(&mut self, dt: f32, level: &Level) {
        self.state.update_with(|state, input| {
            PlayerPosition::update(dt, state, input, level);
        });
    }

    fn position(&self, tick: u8) -> PlayerPosition {
        self.state.get_absolute(tick)
    }

    fn current_position(&self) -> PlayerPosition {
        self.state.get_relative(0)
    }

    fn color_name(&self) -> ColorName {
        self.color
    }

    fn set_visibility(&mut self, connection_id: ConnectionID, visible: bool) {
        self.visibility_state.insert(connection_id, visible);
    }

}


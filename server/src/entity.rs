// External Dependencies ------------------------------------------------------
use hexahydrate;
use cobalt::ConnectionID;
use netsync::{ServerState, NetworkState};


// Internal Dependencies ------------------------------------------------------
use ::shared::color::ColorName;
use ::shared::level::Level;
use ::shared::entity::{
    PlayerInput, PlayerData, PlayerEntity,
    PLAYER_BEAM_FIRE_INTERVAL
};


// Server Entity --------------------------------------------------------------
type ServerPlayerEntity = PlayerEntity<ServerState<PlayerData, PlayerInput>>;

pub trait Entity: hexahydrate::Entity<ConnectionID> {
    fn owner(&self) -> Option<ConnectionID>;
    fn update(&mut self, dt: f32, level: &Level);
    fn client_data(&self, tick: u8, tick_delay: u8) -> PlayerData;
    fn damage(&mut self, amount: u8);
    fn current_data(&self) -> PlayerData;
    fn color_name(&self) -> ColorName;
    fn set_visibility(&mut self, ConnectionID, bool);
    fn get_visibility(&self, connection_id: ConnectionID) -> bool;
    fn fire_beam(&mut self, t: u64) -> bool;
}

impl Entity for ServerPlayerEntity {

    fn owner(&self) -> Option<ConnectionID> {
        self.owner
    }

    fn update(&mut self, dt: f32, level: &Level) {
        self.state.update_with(|state, _, input| {
            PlayerData::update(dt, state, input.unwrap(), level);
        });
    }

    fn client_data(&self, tick: u8, tick_delay: u8) -> PlayerData {
        self.state.get_absolute(tick, tick_delay)
    }

    fn damage(&mut self, amount: u8) {
        self.state.apply(|data| {
            data.hp = data.hp.saturating_sub(amount);
        });
    }

    fn current_data(&self) -> PlayerData {
        self.state.get_relative(0)
    }

    fn color_name(&self) -> ColorName {
        self.color
    }

    fn set_visibility(&mut self, connection_id: ConnectionID, visible: bool) {
        self.visibility_state.insert(connection_id, visible);
    }

    fn get_visibility(&self, connection_id: ConnectionID) -> bool {
        if let Some(visibility) = self.visibility_state.get(&connection_id) {
            *visibility

        } else {
            false
        }
    }

    fn fire_beam(&mut self, t: u64) -> bool {
        // The client also limits the firing rate, however we want to make sure
        // that we always accept the firing command if the client limited correclty
        self.fire_beam(PLAYER_BEAM_FIRE_INTERVAL - 15, t)
    }

}


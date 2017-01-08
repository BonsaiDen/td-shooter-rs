// External Dependencies ------------------------------------------------------
use hexahydrate;
use cobalt::ConnectionID;
use netsync::{ServerState, NetworkState};


// Internal Dependencies ------------------------------------------------------
use ::shared::color::ColorName;
use ::shared::level::{Level, LevelSpawn};
use ::shared::entity::{
    PlayerInput, PlayerData, PlayerEntity,
    PLAYER_MAX_HP,
    PLAYER_BEAM_FIRE_INTERVAL
};


// Server Entity --------------------------------------------------------------
type ServerPlayerEntity = PlayerEntity<ServerState<PlayerData, PlayerInput>>;

pub trait Entity: hexahydrate::Entity<ConnectionID> {
    fn owner(&self) -> Option<ConnectionID>;
    fn is_alive(&self) -> bool;
    fn tick_diff(&self, tick: u8, tick_delay: u8) -> u8;
    fn relative_data(&self, ticks_ago: u8) -> PlayerData;
    fn current_data(&self) -> PlayerData;
    fn color_name(&self) -> ColorName;
    fn set_visibility(&mut self, ConnectionID, bool);
    fn get_visibility(&self, connection_id: ConnectionID) -> bool;
    fn fire_beam(&mut self, t: u64) -> bool;
    fn damage(&mut self, amount: u8);
    fn respawn(&mut self, spawn: LevelSpawn);
    fn update(&mut self, dt: f32, level: &Level);
}

impl Entity for ServerPlayerEntity {

    fn owner(&self) -> Option<ConnectionID> {
        self.owner
    }

    fn is_alive(&self) -> bool {
        self.state.get_relative(0).hp > 0
    }

    fn tick_diff(&self, tick: u8, tick_delay: u8) -> u8 {
        self.state.get_client_tick_diff(tick, tick_delay)
    }

    fn relative_data(&self, ticks_ago: u8) -> PlayerData {
        self.state.get_relative(ticks_ago)
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

    fn damage(&mut self, amount: u8) {
        self.state.apply(|data| {
            data.hp = data.hp.saturating_sub(amount);
        });
    }

    fn respawn(&mut self, spawn: LevelSpawn) {
        self.state.apply(|data| {
            data.hp = PLAYER_MAX_HP;
            data.x = spawn.x;
            data.y = spawn.y;
        });
    }

    fn update(&mut self, dt: f32, level: &Level) {
        self.state.update_with(|state, _, input| {
            PlayerData::update(dt, state, input.unwrap(), level);
        });
    }

}


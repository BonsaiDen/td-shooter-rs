// External Dependencies ------------------------------------------------------
use hexahydrate;
use cobalt::ConnectionID;
use hexahydrate::Entity as EntityTrait;
use netsync::{ClientState, NetworkState};


// Internal Dependencies ------------------------------------------------------
use ::level::Level;
use shared::color::ColorName;
use shared::level::LevelVisibility;
use shared::entity::{PlayerInput, PlayerData, PlayerEntity};


// Statics --------------------------------------------------------------------
const PLAYER_FADE_DURATION: f32 = 150.0;
const PLAYER_EXTRAPOLATE_DURATION: f32 = PLAYER_FADE_DURATION * 4.0;


// Client Entity --------------------------------------------------------------
pub trait Entity: hexahydrate::Entity<ConnectionID> {
    fn is_new(&mut self) -> bool;
    fn is_local(&self) -> bool;
    fn is_alive(&self) -> bool;
    fn color_name(&self) -> ColorName;
    fn colors(&self) -> [[f32; 4]; 2];
    fn interpolate(&self, u: f32) -> PlayerData;
    fn update_remote(&mut self, level: &Level, t: u64);
    fn update_local(&mut self, level: &Level, input: PlayerInput);
    fn update_visibility(&mut self, level: &Level, data: &PlayerData, p: &PlayerData, t: u64) -> f32;
}

impl Entity for PlayerEntity<ClientState<PlayerData, PlayerInput>> {

    fn is_new(&mut self) -> bool {
        if self.is_new {
            self.is_new = false;
            true

        } else {
            false
        }
    }

    fn is_local(&self) -> bool {
        self.local
    }

    fn is_alive(&self) -> bool {
        self.state.interpolate(0.0).hp > 0
    }

    fn color_name(&self) -> ColorName {
        self.color
    }

    fn colors(&self) -> [[f32; 4]; 2] {
        [self.color_light, self.color_dark]
    }

    fn interpolate(&self, u: f32) -> PlayerData {
        self.state.interpolate(u)
    }

    fn update_remote(&mut self, level: &Level, t: u64) {

        let time_since_visible = ((t - self.last_visible) as f32).min(PLAYER_EXTRAPOLATE_DURATION);
        self.state.update_with(|state, last, _| {

            // Keep hidden entities moving at their last known velocity
            if !state.visible && time_since_visible < PLAYER_EXTRAPOLATE_DURATION {
                PlayerData::update_extrapolated(state, level);

            // Calculate velocity of remote entities
            } else if let Some(last) = last {
                state.vx = state.x - last.x;
                state.vy = state.y - last.y;
            }

        });

    }

    fn update_local(&mut self, level: &Level, input: PlayerInput) {
        self.state.input(input);
        self.state.update_with(|state, _, input| {
            PlayerData::update(input.unwrap().dt, state, input.unwrap(), level);
        });
    }

    fn update_visibility(
        &mut self,
        level: &Level,
        data: &PlayerData,
        p: &PlayerData,
        t: u64

    ) -> f32 {

        // Players not visible on the server are never visible on the client either
        let is_visible = if !p.visible {
            false

        // Otherwise we emulate the server side behavior in order to smooth out
        // any lag
        } else {
            level.player_within_visibility(data, p)
        };

        if is_visible {
            self.last_visible = t;
            let time_since_hidden = ((t - self.last_hidden) as f32).min(PLAYER_FADE_DURATION);
            let u = ((1.0 / PLAYER_FADE_DURATION) * time_since_hidden).min(1.0);
            u

        } else {
            self.last_hidden = t;
            let time_since_visible = ((t - self.last_visible) as f32).min(PLAYER_FADE_DURATION);
            let u = ((1.0 / PLAYER_FADE_DURATION) * time_since_visible).min(1.0);
            1.0 - u
        }

    }

}


// Entity Registry ------------------------------------------------------------
#[derive(Debug)]
pub struct Registry;

impl hexahydrate::EntityRegistry<Entity, ConnectionID> for Registry {
    fn entity_from_bytes(&self, kind: u8, bytes: &[u8]) -> Option<Box<Entity>> {
        match kind {
            1 => PlayerEntity::from_bytes(bytes).map(|p| Box::new(p) as Box<Entity>),
            _ => None
        }
    }
}


// External Dependencies ------------------------------------------------------
use hexahydrate;
use cobalt::ConnectionID;
use hexahydrate::Entity as EntityTrait;
use netsync::{ClientState, NetworkState};


// Internal Dependencies ------------------------------------------------------
use ::level::Level;
use shared::util;
use shared::color::ColorName;
use shared::level::{LevelVisibility, LEVEL_MAX_VISIBILITY_DISTANCE};
use shared::entity::{
    PlayerInput, PlayerData, PlayerEntity,
    PLAYER_RADIUS,
    PLAYER_VISBILITY_CONE,
    PLAYER_VISBILITY_CONE_OFFSET
};


// Statics --------------------------------------------------------------------
const PLAYER_FADE_DURATION: f32 = 150.0;
const PLAYER_EXTRAPOLATE_DURATION: f32 = PLAYER_FADE_DURATION * 4.0;


// Client Entity --------------------------------------------------------------
pub trait Entity: hexahydrate::Entity<ConnectionID> {
    fn is_local(&self) -> bool;
    fn interpolate(&self, u: f32) -> PlayerData;
    fn update_remote(&mut self, level: &Level, t: u64);
    fn update_local(&mut self, level: &Level, input: PlayerInput);
    fn update_visibility(&mut self, level: &Level, data: &PlayerData, p: &PlayerData, t: u64) -> f32;
    fn color_name(&self) -> ColorName;
    fn colors(&self) -> [[f32; 4]; 2];
    fn is_new(&mut self) -> bool;
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

        // TODO merge with server side visibility check

        // Players not visible on the server are never visible on the client either
        let is_visible = if !p.visible {
            false

        // Players standing in a light circle are always visible
        } else if level.circle_in_light(p.x, p.y, PLAYER_RADIUS) {
            true

        // Dead players cannot see any other players
        } else if data.hp == 0 {
            false

        // Players outside the maximum visibility distance are never visible
        } else if util::distance(p.x, p.y, data.x, data.y) > LEVEL_MAX_VISIBILITY_DISTANCE - PLAYER_VISBILITY_CONE_OFFSET + PLAYER_RADIUS * 0.5 {
            false

        } else {

            // Players outside the visibility cone are never visible
            if !util::angle_within_cone(data.x, data.y, data.r, p.x, p.y, PLAYER_VISBILITY_CONE_OFFSET, PLAYER_VISBILITY_CONE) {
                false

            } else {
                // Players within the visibility cone are only visible if sight is not blocked by a wall
                level.circle_visible_from(p.x, p.y, PLAYER_RADIUS, data.x, data.y)
            }

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

    fn color_name(&self) -> ColorName {
        self.color
    }

    fn colors(&self) -> [[f32; 4]; 2] {
        [self.color_light, self.color_dark]
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


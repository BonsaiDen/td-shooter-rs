// External Dependencies ------------------------------------------------------
use hexahydrate;
use cobalt::ConnectionID;
use hexahydrate::Entity as EntityTrait;
use netsync::{ClientState, NetworkState};


// Internal Dependencies ------------------------------------------------------
use ::level::Level;
use shared::color::ColorName;
use shared::level::LevelVisibility;
use shared::entity::{PlayerInput, PlayerPosition, PlayerEntity, PLAYER_RADIUS};


// Statics --------------------------------------------------------------------
const PLAYER_FADE_DURATION: f32 = 75.0;


// Client Entity --------------------------------------------------------------
pub trait Entity: hexahydrate::Entity<ConnectionID> {
    fn is_local(&self) -> bool;
    fn interpolate(&self, u: f32) -> PlayerPosition;
    fn update_remote(&mut self);
    fn update_local(&mut self, level: &Level, input: PlayerInput);
    fn update_visibility(&mut self, x: f32, y: f32, level: &Level, position: &PlayerPosition, t: u64) -> f32;
    fn color_name(&self) -> ColorName;
    fn colors(&self) -> [[f32; 4]; 2];
    fn is_new(&mut self) -> bool;
}

impl Entity for PlayerEntity<ClientState<PlayerPosition, PlayerInput>> {

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

    fn interpolate(&self, u: f32) -> PlayerPosition {
        self.state.interpolate(u)
    }

    fn update_remote(&mut self) {
        self.state.update_with(|_, _| {});
    }

    fn update_local(&mut self, level: &Level, input: PlayerInput) {
        self.state.input(input);
        self.state.update_with(|state, input| {
            PlayerPosition::update(input.dt, state, input, level);
        });
    }

    fn update_visibility(&mut self, x: f32, y: f32, level: &Level, p: &PlayerPosition, t: u64) -> f32 {

        let is_visible = p.visible && (level.circle_in_light(
            p.x, p.y,
            PLAYER_RADIUS

        ) || level.circle_visible_from(
            p.x, p.y,
            PLAYER_RADIUS,
            x,
            y,
        ));

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


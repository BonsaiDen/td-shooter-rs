// External Dependencies ------------------------------------------------------
use hexahydrate;
use cobalt::ConnectionID;
use hexahydrate::Entity as EntityTrait;
use netsync::{ClientState, NetworkState};


// Internal Dependencies ------------------------------------------------------
use ::level::Level;
use shared::color::ColorName;
use shared::entity::{PlayerInput, PlayerPosition, PlayerEntity};


// Client Entity --------------------------------------------------------------
pub trait Entity: hexahydrate::Entity<ConnectionID> {
    fn is_local(&self) -> bool;
    fn interpolate(&self, u: f64) -> PlayerPosition;
    fn update_remote(&mut self);
    fn update_local(&mut self, level: &Level, input: PlayerInput);
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

    fn interpolate(&self, u: f64) -> PlayerPosition {
        self.state.interpolate(u)
    }

    fn update_remote(&mut self) {
        // TODO do we need to do more things here with the remote entitis?
        // e.g. level collision?
        self.state.update_with(|_, _| {});
    }

    fn update_local(&mut self, level: &Level, input: PlayerInput) {
        self.state.input(input);
        self.state.update_with(|state, input| {
            PlayerPosition::update(input.dt as f64, state, input, level);
        });
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


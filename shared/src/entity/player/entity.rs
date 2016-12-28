// External Dependencies ------------------------------------------------------
use hexahydrate;
use cobalt::ConnectionID;
use netsync::{NetworkState, ClientState, ServerState, NetworkProperty};


// Internal Dependencies ------------------------------------------------------
use super::{PlayerPosition, PlayerInput};
use ::color::{Color, ColorName};


// Entities -------------------------------------------------------------------
#[derive(Debug)]
pub struct PlayerEntity<S: NetworkState<PlayerPosition, PlayerInput>> {
    pub color: ColorName,
    pub color_light: [f32; 4],
    pub color_dark: [f32; 4],
    pub local: bool,
    pub owner: Option<ConnectionID>,
    pub state: S,
    pub is_new: bool
}

impl<S: NetworkState<PlayerPosition, PlayerInput>> PlayerEntity<S> {

    pub fn new(
        owner: Option<ConnectionID>,
        local: bool,
        color: ColorName,
        position: PlayerPosition

    ) -> PlayerEntity<S> {

        let mut entity = PlayerEntity {
            color: color,
            color_light: Color::from_name(color).into_f32(),
            color_dark: Color::from_name(color).darken(0.5).into_f32(),
            local: local,
            owner: owner,
            state: S::new(30),
            is_new: true
        };

        entity.state.set(position);
        entity

    }

    pub fn is_owned_by(&self, connection_slot: Option<&hexahydrate::ConnectionSlot<ConnectionID>>) -> bool {
        if let Some(slot) = connection_slot.as_ref() {
            if let Some(owner) = self.owner.as_ref() {
                slot.user_data == *owner

            } else {
                false
            }

        } else {
            false
        }
    }

}

// Server Side Entity ---------------------------------------------------------
impl hexahydrate::Entity<ConnectionID> for PlayerEntity<ServerState<PlayerPosition, PlayerInput>> {

    fn part_bytes(&mut self, connection_slot: Option<&hexahydrate::ConnectionSlot<ConnectionID>>) -> Option<Vec<u8>> {
        if self.is_owned_by(connection_slot) {
            Some(self.state.send(None))

        } else {
            Some(self.state.send(Some(4))) // TODO make configurable
        }
    }

    fn merge_bytes(&mut self, connection_slot: Option<&hexahydrate::ConnectionSlot<ConnectionID>>, bytes: &[u8]) {
        if self.is_owned_by(connection_slot) {
            self.state.receive(bytes);
        }
    }

    fn kind(&self) -> u8 {
        1
    }

    fn to_bytes(&self, connection_slot: &hexahydrate::ConnectionSlot<ConnectionID>) -> Vec<u8> {
        let mut bytes = vec![
            if self.is_owned_by(Some(connection_slot)) { 1 } else { 0 },
            self.color.to_u8()
        ];
        bytes.append(&mut self.state.send(Some(0)));
        bytes
    }

}

// Client Side Entity ---------------------------------------------------------
impl hexahydrate::Entity<ConnectionID> for PlayerEntity<ClientState<PlayerPosition, PlayerInput>> {

    fn part_bytes(&mut self, _: Option<&hexahydrate::ConnectionSlot<ConnectionID>>) -> Option<Vec<u8>> {
        if self.local {
            Some(self.state.send())

        } else {
            None
        }
    }

    fn merge_bytes(&mut self, _: Option<&hexahydrate::ConnectionSlot<ConnectionID>>, bytes: &[u8]) {
        if self.local {
            self.state.receive(&bytes[1..], Some(bytes[0]));

        } else {
            self.state.receive(bytes, None);
        }
    }

    fn kind(&self) -> u8 {
        1
    }

    fn from_bytes(bytes: &[u8]) -> Option<PlayerEntity<ClientState<PlayerPosition, PlayerInput>>> {
        Some(PlayerEntity::new(
            None,
            bytes[0] == 1,
            ColorName::from_u8(bytes[1]),
            PlayerPosition::from_bytes(&bytes[2..])
        ))
    }

}


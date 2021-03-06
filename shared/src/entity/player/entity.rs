// STD Dependencies -----------------------------------------------------------
use std::collections::HashMap;


// External Dependencies ------------------------------------------------------
use hexahydrate;
use cobalt::ConnectionID;
use netsync::{NetworkState, ClientState, ServerState, NetworkProperty};


// Internal Dependencies ------------------------------------------------------
use super::{PlayerData, PlayerInput};
use ::color::{Color, ColorName};


// Statics --------------------------------------------------------------------
pub const ENTITY_STATE_DELAY: u8 = 4;


// Entities -------------------------------------------------------------------
#[derive(Debug)]
pub struct PlayerEntity<S: NetworkState<PlayerData, PlayerInput>> {
    pub color: ColorName,
    pub color_light: [f32; 4],
    pub color_dark: [f32; 4],
    pub local: bool,
    pub owner: Option<ConnectionID>,
    pub state: S,
    pub is_new: bool,
    pub visibility_state: HashMap<ConnectionID, bool>,
    pub last_visible: u64,
    pub last_hidden: u64,
    pub last_beam_fire: u64
}

impl<S: NetworkState<PlayerData, PlayerInput>> PlayerEntity<S> {

    pub fn new(
        owner: Option<ConnectionID>,
        local: bool,
        color: ColorName,
        data: PlayerData

    ) -> PlayerEntity<S> {

        println!("Player entity created: {:?}", color);

        let mut entity = PlayerEntity {
            color: color,
            color_light: Color::from_name(color).into_f32(),
            color_dark: Color::from_name(color).darken(0.5).into_f32(),
            local: local,
            owner: owner,
            state: S::new(30),
            is_new: true,
            visibility_state: HashMap::new(),
            last_hidden: 0,
            last_visible: 0,
            last_beam_fire: 0
        };

        entity.state.set(data);
        entity

    }

    pub fn is_visible_to(&self, connection_slot: Option<&hexahydrate::ConnectionSlot<ConnectionID>>) -> bool {
        if let Some(slot) = connection_slot {
            if let Some(state) = self.visibility_state.get(&slot.user_data) {
                *state

            } else {
                false
            }

        } else {
            false
        }
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

    pub fn fire_beam(&mut self, interval: u64, t: u64) -> bool {
        if t >= self.last_beam_fire + interval {
            self.last_beam_fire = t;
            true

        } else {
            false
        }
    }

}

impl<S: NetworkState<PlayerData, PlayerInput>> Drop for PlayerEntity<S> {
    fn drop(&mut self) {
        println!("Player entity dropped: {:?}", self.color);
    }
}


// Server Side Entity ---------------------------------------------------------
impl hexahydrate::Entity<ConnectionID> for PlayerEntity<ServerState<PlayerData, PlayerInput>> {

    fn part_bytes(&mut self, connection_slot: Option<&hexahydrate::ConnectionSlot<ConnectionID>>) -> Option<Vec<u8>> {
        if self.is_owned_by(connection_slot) {
            Some(self.state.send(None))

        } else {
            let bytes = self.state.send_with(Some(ENTITY_STATE_DELAY), |state| {

                // Hide dead entities
                let dead = state.hp == 0;

                // Never expose hp to other players
                state.hp = 0;

                if !dead && self.is_visible_to(connection_slot) {
                    state.visible = true;

                } else {
                    state.x = 0.0;
                    state.y = 0.0;
                    state.r = 0.0;
                    state.visible = false;
                }

            });
            Some(bytes)
        }
    }

    fn merge_bytes(&mut self, connection_slot: Option<&hexahydrate::ConnectionSlot<ConnectionID>>, bytes: &[u8]) {

        // Dead entities ignore any inputs
        if self.is_owned_by(connection_slot) && self.state.get_relative(0).hp > 0 {
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
impl hexahydrate::Entity<ConnectionID> for PlayerEntity<ClientState<PlayerData, PlayerInput>> {

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
            self.state.receive_with(bytes, None, |current, state| {
                if !state.visible {
                    state.x = current.x;
                    state.y = current.y;
                    state.r = current.r;
                    state.vx = current.vx;
                    state.vy = current.vy;
                }
            });
        }
    }

    fn kind(&self) -> u8 {
        1
    }

    fn from_bytes(bytes: &[u8]) -> Option<PlayerEntity<ClientState<PlayerData, PlayerInput>>> {
        Some(PlayerEntity::new(
            None,
            bytes[0] == 1,
            ColorName::from_u8(bytes[1]),
            PlayerData::from_bytes(&bytes[2..])
        ))
    }

}


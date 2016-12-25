// STD Dependencies -----------------------------------------------------------
use std::fmt;


// Internal Dependencies ------------------------------------------------------
use ::shared::{Entity, EntitySerializer};
use ::server::ConnectionSlot;
use super::{EntitySlot, NetworkState};


// Client Entity State --------------------------------------------------------
#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
pub enum LocalState {
    Unknown,
    Accept,
    Create,
    Update
}

state_machine!(LocalState, {
    create: LocalState::Unknown => LocalState::Create,
    accept: LocalState::Create => LocalState::Accept,
    update: LocalState::Accept => LocalState::Update,
    reset: LocalState::Create | LocalState::Accept | LocalState::Update => LocalState::Unknown,
});


// Client Entity Implementation -----------------------------------------------
pub struct Serializer;
impl<E: Entity<U> + ?Sized, U: fmt::Debug> EntitySerializer<E, LocalState, EntitySlot, U> for Serializer {

    fn to_bytes(
        entity: Option<&mut Box<E>>,
        slot: &EntitySlot,
        connection_slot: Option<&ConnectionSlot<U>>,
        state: &LocalState

    ) -> Vec<u8> {

        let index = slot.index as u8;
        if let Some(entity) = entity {
            match *state {

                LocalState::Create => {
                    vec![NetworkState::ConfirmCreateToServer as u8, index]
                },

                LocalState::Accept => {
                    vec![NetworkState::AcceptServerUpdate as u8, index]
                },

                LocalState::Update => if let Some(update_bytes) = entity.part_bytes(connection_slot) {

                    // TODO handle more than 255 bytes with bigger frames etc.
                    if update_bytes.len() > 255 {
                        panic!("More than 255 bytes in update!");
                    }

                    let mut bytes = vec![
                        NetworkState::SendUpdateToServer as u8,
                        index,
                        update_bytes.len() as u8
                    ];
                    bytes.extend_from_slice(&update_bytes);
                    bytes

                } else {
                    vec![]
                },

                _ => vec![]

            }

        } else {
            vec![NetworkState::ConfirmDestroyToServer as u8, index]
        }

    }

}


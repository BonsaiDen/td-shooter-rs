// STD Dependencies -----------------------------------------------------------
use std::fmt;


// Internal Dependencies ------------------------------------------------------
use ::shared::{Entity, EntitySerializer};
use super::{ConnectionSlot, EntitySlot, NetworkState};


// Server Entity State --------------------------------------------------------
#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Copy, Clone)]
pub enum RemoteState {
    Unknown,
    Accept,
    Create,
    Update,
    Destroy,
    Forget,
    Forgotten
}

state_machine!(RemoteState, {
    accept: RemoteState::Unknown => RemoteState::Accept,
    reset_accepted: RemoteState::Accept => RemoteState::Unknown,
    reset_destroyed: RemoteState::Destroy => RemoteState::Unknown,
    reset_forgotten: RemoteState::Forgotten => RemoteState::Unknown,
    create: RemoteState::Unknown => RemoteState::Create,
    update: RemoteState::Create => RemoteState::Update,
    destroy: RemoteState::Accept | RemoteState::Create | RemoteState::Update => RemoteState::Destroy,
    forget: RemoteState::Accept | RemoteState::Create | RemoteState::Update => RemoteState::Forget,
    forgotten: RemoteState::Forget => RemoteState::Forgotten,
});


// Server Entity Handle -------------------------------------------------------
pub struct Serializer;
impl<E: Entity<U> + ?Sized, U: fmt::Debug> EntitySerializer<E, RemoteState, EntitySlot, U> for Serializer {

    fn to_bytes(
        entity: Option<&mut Box<E>>,
        slot: &EntitySlot,
        connection_slot: Option<&ConnectionSlot<U>>,
        state: &RemoteState

    ) -> Vec<u8> {

        let index = slot.index as u8;
        if let Some(entity) = entity {
            match *state {

                RemoteState::Unknown => {
                    let create_bytes = entity.to_bytes(connection_slot.unwrap());

                    // TODO handle more than 255 bytes with bigger frames etc.
                    if create_bytes.len() > 255 {
                        panic!("More than 255 bytes in update!");
                    }

                    let mut bytes = vec![
                        NetworkState::SendCreateToClient as u8,
                        index,
                        create_bytes.len() as u8,
                        entity.kind()
                    ];
                    bytes.extend_from_slice(&create_bytes);
                    bytes
                },

                RemoteState::Create => {
                    vec![NetworkState::ConfirmClientCreate as u8, index]
                },

                RemoteState::Update => if let Some(update_bytes) = entity.part_bytes(connection_slot) {

                    // TODO handle more than 255 bytes with bigger frames etc.
                    if update_bytes.len() > 255 {
                        panic!("More than 255 bytes in update!");
                    }

                    let mut bytes = vec![
                        NetworkState::SendUpdateToClient as u8,
                        index,
                        update_bytes.len() as u8
                    ];
                    bytes.extend_from_slice(&update_bytes);
                    bytes

                } else {
                    vec![]
                },

                RemoteState::Forget => {
                    vec![NetworkState::SendForgetToClient as u8, index]
                },

                _ => vec![]

            }

        } else {
            vec![NetworkState::SendDestroyToClient as u8, index]
        }

    }

}


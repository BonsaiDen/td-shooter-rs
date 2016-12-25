// STD Dependencies -----------------------------------------------------------
use std::fmt;
use std::sync::atomic::{AtomicUsize, Ordering};


// Internal Dependencies ------------------------------------------------------
use ::server::NetworkState as ServerNetworkState;
use ::shared::{
    Entity, EntityHandle, EntityRegistry,
    PacketList,
    deserialize_entity_bytes
};


// Modules --------------------------------------------------------------------
mod entity;
use self::entity::{Serializer, LocalState};


// Client Entity Slot ---------------------------------------------------------
#[derive(Debug, Eq, PartialEq, Hash)]
pub struct EntitySlot {
    index: usize,
    client_index: usize
}

impl EntitySlot {
    fn new(index: usize, client_index: usize) -> EntitySlot {
        EntitySlot {
            index: index,
            client_index: client_index
        }
    }
}


// Client Errors --------------------------------------------------------------
#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
pub enum Error {
    InvalidPacketData
}


// Client Side Network State --------------------------------------------------
#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
pub enum NetworkState {
    ConfirmCreateToServer = 1,
    AcceptServerUpdate = 2,
    SendUpdateToServer = 3,
    ConfirmDestroyToServer = 4
}

impl NetworkState {
    pub fn from_u8(state: u8) -> Option<NetworkState> {
        match state {
            1 => Some(NetworkState::ConfirmCreateToServer),
            2 => Some(NetworkState::AcceptServerUpdate),
            3 => Some(NetworkState::SendUpdateToServer),
            4 => Some(NetworkState::ConfirmDestroyToServer),
            _ => None
        }
    }
}


// Client Implementation ------------------------------------------------------
lazy_static! {
    static ref CLIENT_INDEX: AtomicUsize = AtomicUsize::new(0);
}

type ClientEntityHandle<E, U> = Vec<
    Option<EntityHandle<E, Serializer, LocalState, EntitySlot, U>>
>;

pub struct Client<E: Entity<U> + ?Sized, U: fmt::Debug, R: EntityRegistry<E, U>> {
    index: usize,
    handles: ClientEntityHandle<E, U>,
    active_handles: Vec<(EntitySlot, Option<usize>, bool)>,
    local_states: [LocalState; 256],
    handle_timeout: usize,
    registry: R
}

impl<E: Entity<U> + ?Sized, U: fmt::Debug, R: EntityRegistry<E, U>> Client<E, U, R> {

    pub fn new(registry: R, handle_timeout: usize) -> Client<E, U, R> {
        Client {
            index: CLIENT_INDEX.fetch_add(1, Ordering::SeqCst),
            handles: vec_with_default![None; 256],
            local_states: [LocalState::Unknown; 256],
            active_handles: Vec::new(),
            handle_timeout: handle_timeout,
            registry: registry
        }
    }

    pub fn map_entities<T, F: FnMut(&EntitySlot, &mut Box<E>) -> T>(&mut self, mut callback: F) -> Vec<T> {
        let mut items: Vec<T> = Vec::new();
        for &mut (ref entity_slot, _, _) in &mut self.active_handles {
            let handle = &mut self.handles[entity_slot.index];
            if handle.is_some()  {
                if let Some(entity) = handle.as_mut().unwrap().mut_entity() {
                    items.push(callback(entity_slot, entity));
                }
            }
        }
        items
    }

    pub fn with_entities<F: FnMut(&EntitySlot, &mut Box<E>)>(&mut self, mut callback: F) {
        for &mut (ref entity_slot, _, _) in &mut self.active_handles {
            let handle = &mut self.handles[entity_slot.index];
            if handle.is_some()  {
                if let Some(entity) = handle.as_mut().unwrap().mut_entity() {
                    callback(entity_slot, entity);
                }
            }
        }
    }

    pub fn update_with<F: FnMut(&EntitySlot, &mut Box<E>)>(&mut self, mut callback: F) {

        for &mut(ref entity_slot, ref mut timeout, ref mut connected) in &mut self.active_handles {

            let handle = &mut self.handles[entity_slot.index];
            if handle.is_some() {

                if let Some(entity) = handle.as_mut().unwrap().mut_entity() {
                    callback(entity_slot, entity);

                // The server removes the entity once we confirmed receiving
                // the destruction so we need timeout the local handle to have
                // a buffer for the destruction confirmation getting through
                // from us to the server in the first place
                } else if timeout.is_none() {
                    *timeout = Some(self.handle_timeout);
                }

                // Safely reduce timeout until we hit 0
                if timeout.is_some() {
                    *timeout = Some(timeout.unwrap().saturating_sub(1));
                    if timeout.unwrap() == 0 {
                        *connected = false;
                    }
                }

            }

            // Drop the handle once it is no longer connected with the server
            if !*connected {
                self.local_states[entity_slot.index].reset();
                *handle = None
            }

        }

        // Remove disconnected handles
        self.active_handles.retain(|&(_, _, connected)| connected );

    }

    pub fn send(&mut self, max_bytes_per_packet: usize) -> Vec<Vec<u8>> {

        let mut packets = PacketList::new(max_bytes_per_packet);
        for &mut(ref entity_slot, _, _) in &mut self.active_handles {
            packets.append_bytes(self.handles[entity_slot.index].as_mut().unwrap().to_bytes(
                None,
                &self.local_states[entity_slot.index]
            ));
        }

        packets.into_vec()

    }

    pub fn reset(&mut self) {

        for &mut (ref entity_slot, _, _) in &mut self.active_handles {
            self.local_states[entity_slot.index].reset();
            self.handles[entity_slot.index] = None;
        }

        self.active_handles.clear();

    }

    pub fn receive(&mut self, bytes: Vec<u8>) -> Result<(), Error> {

        let (mut i, len) = (0, bytes.len());
        while i + 1 < len {

            let (state, index) = (bytes[i], bytes[i + 1] as usize);
            let local_state = &mut self.local_states[index];
            i += 2;

            match ServerNetworkState::from_u8(state) {
                Some(ServerNetworkState::SendCreateToClient) => if let Some((entity_bytes, length)) = deserialize_entity_bytes(&bytes[i..], 2) {

                    if self.handles[index].is_none() {

                        if let Some(entity) = self.registry.entity_from_bytes(entity_bytes[0], &entity_bytes[1..]) {
                            local_state.create();
                            self.handles[index] = Some(EntityHandle::new(EntitySlot::new(index, self.index), entity));
                            self.active_handles.push(
                                (EntitySlot::new(index, self.index), None, true)
                            );
                        }

                    // Replace handles in case the server sends new data and this slot is already
                    // occupied.
                    //
                    // We can end up in a situation where we never receive the destroy from
                    // the server due to a timeout on our side, in this case we'll still need
                    // to be able to respond to the creation of a new entity in an existing
                    // slot.
                    //
                    // However, we might also run into issues with mixed ordering of
                    // SendCreateToClient and ConfirmClientCreate packets.
                    //
                    // To work around these issues with establish the following rules:
                    // a. If the entity has a different kind replace it directly
                    // b. If the entity has the same kind as the existing one, replace it
                    //    only when NOT in the create state.
                    //
                    //    If it is in the create state we're already sending
                    //    a ConfirmCreateToServer and message ordering should
                    //    not be a problem.
                    //
                    //    If the entity is in a different state than create
                    //    we replace it and reset its state.
                    //
                    // In all other cases we'll do nothing.
                    } else {
                        let existing_kind = self.handles[index].as_mut().unwrap().mut_entity().map_or(entity_bytes[0], |entity| {
                            entity.kind()
                        });

                        if entity_bytes[0] != existing_kind || *local_state != LocalState::Create {
                            if let Some(entity) = self.registry.entity_from_bytes(entity_bytes[0], &entity_bytes[1..]) {
                                self.handles[index].as_mut().unwrap().replace_entity(entity);
                                local_state.reset();
                                local_state.create();
                            }
                        }
                    }

                    i += length;

                },
                Some(ServerNetworkState::ConfirmClientCreate) => if self.handles[index].is_some() && local_state.accept() {
                    self.handles[index].as_mut().unwrap().create();
                },
                Some(ServerNetworkState::SendUpdateToClient) => if let Some((entity_bytes, length)) = deserialize_entity_bytes(&bytes[i..], 1) {

                    if self.handles[index].is_some() {

                        local_state.update();

                        if *local_state == LocalState::Update {
                            self.handles[index].as_mut().unwrap().merge_bytes(
                                None,
                                entity_bytes
                            );
                        }

                    }

                    i += length;

                },
                Some(ServerNetworkState::SendDestroyToClient) => if self.handles[index].is_some() {
                    // Warning: This may cause previously created entities to be
                    // destroyed out of order if the underlying network stack
                    // does not guarantee that old packets are always
                    // dropped in case their follow ups were already received.
                    //
                    // Not however that we do not rely on full in-order receival of
                    // packets since we specifically support the case were create
                    // packets are received for not-yet destroyed entities.
                    self.handles[index].as_mut().unwrap().destroy();
                },

                Some(ServerNetworkState::SendForgetToClient) => if self.handles[index].is_some() {
                    // Warning: This may cause previously created entities to be
                    // destroyed out of order if the underlying network stack
                    // does not guarantee that old packets are always
                    // dropped in case their follow ups were already received.
                    //
                    // Not however that we do not rely on full in-order receival of
                    // packets since we specifically support the case were create
                    // packets are received for not-yet destroyed entities.
                    self.handles[index].as_mut().unwrap().forget();
                },

                None => return Err(Error::InvalidPacketData)
            }

        }

        Ok(())

    }

}


// Traits ---------------------------------------------------------------------
impl<E: EntityRegistry<Entity<U>, U>, U: fmt::Debug> fmt::Debug for Client<Entity<U>, U, E> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Client")
    }
}


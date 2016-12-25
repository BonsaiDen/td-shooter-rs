// STD Dependencies -----------------------------------------------------------
use std::fmt;
use std::sync::atomic::{AtomicUsize, Ordering};


// Internal Dependencies ------------------------------------------------------
use ::client::NetworkState as ClientNetworkState;
use ::shared::{
    Entity, EntityHandle,
    PacketList,
    deserialize_entity_bytes
};


// Modules --------------------------------------------------------------------
mod entity;
use self::entity::{Serializer, RemoteState};


// Server Connection Slot -----------------------------------------------------
#[derive(Debug, Eq, PartialEq, Hash)]
pub struct ConnectionSlot<U: fmt::Debug> {
    pub user_data: U,
    index: usize,
    server_index: usize
}


// Server Entity Slot ---------------------------------------------------------
#[derive(Debug, Eq, PartialEq, Hash)]
pub struct EntitySlot {
    index: usize,
    server_index: usize
}

impl EntitySlot {
    fn new(index: usize, server_index: usize) -> EntitySlot {
        EntitySlot {
            index: index,
            server_index: server_index
        }
    }
}

// Server Errors --------------------------------------------------------------
#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
pub enum Error {
    AllEntitySlotsInUse,
    EntityAlreadyDestroy,
    EntityDoesNotExist,
    AllConnectionSlotsInUse,
    ConnectionDoesNotExist,
    InvalidPacketData
}


// Server Side Network State --------------------------------------------------
#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
pub enum NetworkState {
    SendCreateToClient = 0,
    ConfirmClientCreate = 1,
    SendUpdateToClient = 3,
    SendDestroyToClient = 4,
    SendForgetToClient = 5
}

impl NetworkState {
    pub fn from_u8(state: u8) -> Option<NetworkState> {
        match state {
            0 => Some(NetworkState::SendCreateToClient),
            1 => Some(NetworkState::ConfirmClientCreate),
            3 => Some(NetworkState::SendUpdateToClient),
            4 => Some(NetworkState::SendDestroyToClient),
            5 => Some(NetworkState::SendForgetToClient),
            _ => None
        }
    }
}


// Server Implementation ------------------------------------------------------
lazy_static! {
    static ref SERVER_INDEX: AtomicUsize = AtomicUsize::new(0);
}

type ServerEntityHandle<E, U> = Vec<
    Option<EntityHandle<E, Serializer, RemoteState, EntitySlot, U>>
>;

pub struct Server<E: Entity<U> + ?Sized, U: fmt::Debug> {
    index: usize,
    handles: ServerEntityHandle<E, U>,
    active_handles: Vec<(EntitySlot, Option<usize>, usize, bool)>,
    active_connections: Vec<usize>,
    connections: Vec<Option<[RemoteState; 256]>>,
    handle_timeout: usize
}

impl<E: Entity<U> + ?Sized, U: fmt::Debug> Server<E, U> {

    pub fn new(handle_timeout: usize) -> Server<E, U> {
        Server {
            index: SERVER_INDEX.fetch_add(1, Ordering::SeqCst),
            handles: vec_with_default![None; 256],
            active_handles: Vec::new(),
            active_connections: Vec::new(),
            connections: vec_with_default![None; 256],
            handle_timeout: handle_timeout
        }
    }

    pub fn entity_create_with<F: FnOnce() -> Box<E>>(&mut self, callback: F) -> Result<EntitySlot, Error> {

        if let Some(index) = self.find_free_entity_slot_index() {

            // Create entity handle which encapsulates the actual entity
            let mut handle = EntityHandle::new(
                EntitySlot::new(index, self.index),
                callback()
            );

            handle.create();

            self.handles[index] = Some(handle);

            // Add to list of active slots
            self.active_handles.push((
                EntitySlot::new(index, self.index),
                None,
                self.active_connections.len(),
                true
            ));

            // Return a unique handle which cannot be copied
            Ok(EntitySlot::new(index, self.index))

        } else {
            Err(Error::AllEntitySlotsInUse)
        }

    }

    pub fn entity_destroy(&mut self, entity_slot: EntitySlot) -> Result<(), Error> {

        if entity_slot.server_index != self.index {
            Err(Error::EntityDoesNotExist)

        } else if let Some(handle) = self.handles[entity_slot.index].as_mut() {
            if handle.is_alive() {
                handle.destroy();
                Ok(())

            } else {
                Err(Error::EntityAlreadyDestroy)
            }

        } else {
            Err(Error::EntityDoesNotExist)
        }
    }

    pub fn map_entities<T, F: FnMut(&EntitySlot, &mut Box<E>) -> T>(&mut self, mut callback: F) -> Vec<T> {
        let mut items: Vec<T> = Vec::new();
        for &mut (ref entity_slot, _, _, _) in &mut self.active_handles {
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
        for &mut (ref entity_slot, _, _, _) in &mut self.active_handles {
            let handle = &mut self.handles[entity_slot.index];
            if handle.is_some()  {
                if let Some(entity) = handle.as_mut().unwrap().mut_entity() {
                    callback(entity_slot, entity);
                }
            }
        }
    }

    pub fn update_with<F: FnMut(&EntitySlot, &mut Box<E>)>(&mut self, mut callback: F) {

        for &mut (ref entity_slot, ref mut timeout, ref mut connection_count, ref mut connected) in &mut self.active_handles {

            let handle = &mut self.handles[entity_slot.index];
            let is_alive = handle.is_some()
                        && handle.as_ref().unwrap().is_alive();

            if is_alive {
                callback(entity_slot, handle.as_mut().unwrap().mut_entity().unwrap())

            } else if *connection_count > 0 {

                // If the entity is destroyed we timeout all open connections
                // that don't respond with a ConfirmDestroyToServer packet
                // within the given number of update calls.
                if timeout.is_none() {
                    *timeout = Some(self.handle_timeout);
                }

                // Safely reduce timeout until we hit 0
                if timeout.is_some() {
                    *timeout = Some(timeout.unwrap().saturating_sub(1));
                    if timeout.unwrap() == 0 {
                        *connection_count = 0;
                    }
                }
            }

            // Drop handlers of destroyed entities in case there are no
            // more connected client
            if !is_alive && *connection_count == 0 {

                // Reset entity state for all open client connections
                self.connections.iter_mut().filter(|r| r.is_some()).map(|r| r.unwrap()).all(|mut remote_states| {
                    remote_states[entity_slot.index].destroy();
                    remote_states[entity_slot.index].reset_destroyed();
                    true
                });

                *connected = false;
                *handle = None;

            }

        }

        // Remove destroy handles without any connected clients
        self.active_handles.retain(|&(_, _, _, connected)| {
            connected
        });

    }

    pub fn connection_add(&mut self, user_data: U) -> Result<ConnectionSlot<U>, Error> {

        if let Some(index) = self.find_free_connection_slot_index() {

            // Put active handles into the accept state for the new connection
            let mut remote_states = [RemoteState::Unknown; 256];
            for &(ref entity_slot, _, _, _) in &self.active_handles {
                remote_states[entity_slot.index].accept();
            }

            self.connections[index] = Some(remote_states);
            self.active_connections.push(index);

            // Return a unique handle which cannot be copied
            Ok(ConnectionSlot {
                user_data: user_data,
                index: index,
                server_index: self.index
            })

        } else {
            Err(Error::AllConnectionSlotsInUse)
        }

    }

    pub fn connection_remove(&mut self, connection_slot: ConnectionSlot<U>) -> Result<U, Error> {

        if connection_slot.server_index != self.index {
            Err(Error::ConnectionDoesNotExist)

        } else if let Some(remote_states) = self.connections[connection_slot.index].take() {

            // Decrease connection counts for all active handles this connection had
            // state for
            for &mut(_, _, ref mut connection_count, _) in &mut self.active_handles {
                if remote_states[connection_slot.index] > RemoteState::Accept {
                    *connection_count -= 1;
                }
            }

            // Remove internal connection
            self.connections[connection_slot.index] = None;
            self.active_connections.retain(|index| *index != connection_slot.index);

            // Return connection slot data
            Ok(connection_slot.user_data)

        } else {
            Err(Error::ConnectionDoesNotExist)
        }

    }

    pub fn connection_send(&mut self, connection_slot: &ConnectionSlot<U>, max_bytes_per_packet: usize) -> Result<Vec<Vec<u8>>, Error> {

        if let Some(remote_states) = self.connections[connection_slot.index].as_mut() {

            let mut packets = PacketList::new(max_bytes_per_packet);
            for &mut(ref slot, _, ref mut connection_count, _) in &mut self.active_handles {

                let handle = &mut self.handles[slot.index];
                let remote_state = &mut remote_states[slot.index];

                if handle.as_ref().unwrap().is_alive() {

                    // Increase the entities connection count for newly established connections
                    if remote_state.reset_accepted() {
                        *connection_count += 1;
                    }

                    // Check if the entity should no longer be send to the connection.
                    // The client should simply forget about the entity and drop it
                    // without running its destroyed() method.
                    if !handle.as_ref().unwrap().filter(connection_slot) {
                        if *remote_state < RemoteState::Forget {
                            remote_state.forget();
                        }

                    // If the entity should be send to the client again,
                    // reset its state so we tell the client to create it again
                    } else {
                        remote_state.reset_forgotten();
                    }

                // Reduce the entities connection count if a client has confirmed destruction
                } else if *connection_count > 0 && remote_state.reset_destroyed() {
                    *connection_count -= 1;
                }

                // Only serialize entities which have open client connections
                if *connection_count > 0 {
                    packets.append_bytes(handle.as_mut().unwrap().to_bytes(
                        Some(connection_slot),
                        remote_state
                    ));
                }

            }

            Ok(packets.into_vec())

        } else {
            Err(Error::ConnectionDoesNotExist)
        }

    }

    pub fn connection_receive(&mut self, connection_slot: &ConnectionSlot<U>, bytes: Vec<u8>) -> Result<(), Error> {

        if let Some(remote_states) = self.connections[connection_slot.index].as_mut() {

            let (mut i, len) = (0, bytes.len());
            while i + 1 < len {

                let (state, index) = (bytes[i], bytes[i + 1] as usize);
                let remote_state = &mut remote_states[index];
                i += 2;

                match ClientNetworkState::from_u8(state) {
                    Some(ClientNetworkState::ConfirmCreateToServer) => if self.handles[index].is_some() {
                        remote_state.create();
                    },
                    Some(ClientNetworkState::AcceptServerUpdate) => if self.handles[index].is_some() {
                        remote_state.update();
                    },
                    Some(ClientNetworkState::SendUpdateToServer) => if let Some((entity_bytes, length)) = deserialize_entity_bytes(&bytes[i..], 1) {

                        if self.handles[index].is_some() && *remote_state == RemoteState::Update {
                            self.handles[index].as_mut().unwrap().merge_bytes(
                                Some(connection_slot),
                                entity_bytes
                            );
                        }

                        i += length;

                    },
                    Some(ClientNetworkState::ConfirmDestroyToServer) => if self.handles[index].is_some() {
                        if !self.handles[index].as_ref().unwrap().is_alive() {
                            remote_state.destroy();

                        } else {
                            remote_state.forgotten();
                        }
                    },
                    None => return Err(Error::InvalidPacketData)
                }

            }

            Ok(())

        } else {
            Err(Error::ConnectionDoesNotExist)
        }

    }

    // Internal

    fn find_free_entity_slot_index(&self) -> Option<usize> {
        for i in 0..256 {
            if self.handles[i].is_none() {
                return Some(i);
            }
        }
        None
    }

    fn find_free_connection_slot_index(&self) -> Option<usize> {
        for i in 0..256 {
            if self.connections[i].is_none() {
                return Some(i);
            }
        }
        None
    }

}


// Traits ---------------------------------------------------------------------
impl<U: fmt::Debug> fmt::Debug for Server<Entity<U>, U> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Server")
    }
}


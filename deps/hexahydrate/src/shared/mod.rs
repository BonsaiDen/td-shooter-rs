// STD Dependencies -----------------------------------------------------------
use std::fmt;
use std::marker::PhantomData;


// Internal Dependencies ------------------------------------------------------
use ::server::ConnectionSlot;


// Entity Traits ----------------------------------------------------------------
pub trait Entity<U: fmt::Debug>: fmt::Debug {

    fn kind(&self) -> u8;

    fn created(&mut self) {
    }

    fn filter(&self, &ConnectionSlot<U>) -> bool {
        true
    }

    fn destroyed(&mut self) {
    }

    fn part_bytes(&mut self, Option<&ConnectionSlot<U>>) -> Option<Vec<u8>>;
    fn merge_bytes(&mut self, Option<&ConnectionSlot<U>>, &[u8]);

    fn to_bytes(&self, &ConnectionSlot<U>) -> Vec<u8> {
        vec![]
    }

    fn from_bytes(&[u8]) -> Option<Self> where Self: Sized {
        None
    }

}

pub trait EntityRegistry<E: Entity<U> + ?Sized, U: fmt::Debug>: fmt::Debug {
    fn entity_from_bytes(&self, u8, &[u8]) -> Option<Box<E>>;
}

pub trait EntitySerializer<E: Entity<U> + ?Sized, S, O, U: fmt::Debug> {
    fn to_bytes(Option<&mut Box<E>>, &O, Option<&ConnectionSlot<U>>, &S) -> Vec<u8>;
}


// Entity Network Handle ------------------------------------------------------
pub struct EntityHandle<E: Entity<U> + ?Sized, R: EntitySerializer<E, S, O, U>, S, O, U: fmt::Debug> {
    slot: O,
    entity: Option<Box<E>>,
    handler: PhantomData<R>,
    state: PhantomData<S>,
    connection_id: PhantomData<U>
}

impl<E: Entity<U> + ?Sized, R: EntitySerializer<E, S, O, U>, S, O, U: fmt::Debug> EntityHandle<E, R, S, O, U> {

    pub fn new(slot: O, entity: Box<E>) -> EntityHandle<E, R, S, O, U> {
        EntityHandle {
            slot: slot,
            entity: Some(entity),
            handler: PhantomData,
            state: PhantomData,
            connection_id: PhantomData
        }
    }

    pub fn is_alive(&self) -> bool {
        self.entity.is_some()
    }

    pub fn mut_entity(&mut self) -> Option<&mut Box<E>> {
        self.entity.as_mut()
    }

    pub fn filter(&self, connection_slot: &ConnectionSlot<U>) -> bool {
        self.entity.as_ref().unwrap().filter(connection_slot)
    }

    pub fn merge_bytes(&mut self, connection_slot: Option<&ConnectionSlot<U>>, bytes: &[u8]) {
        if let Some(ref mut entity) = self.entity {
            entity.merge_bytes(connection_slot, bytes);
        }
    }

    pub fn replace_entity(&mut self, entity: Box<E>) {
        self.forget();
        self.entity = Some(entity);
    }

    pub fn to_bytes(&mut self, connection_slot: Option<&ConnectionSlot<U>>, state: &S) -> Vec<u8> {
        R::to_bytes(self.entity.as_mut(), &self.slot, connection_slot, state)
    }

    pub fn create(&mut self) {
        if let Some(entity) = self.entity.as_mut() {
            entity.created();
        }
    }

    pub fn destroy(&mut self) {
        if let Some(mut entity) = self.entity.take() {
            entity.destroyed();
        }
    }

    pub fn forget(&mut self) {
        self.entity.take();
    }

}

// Packet Chunk List ----------------------------------------------------------
pub struct PacketList {
    max_bytes_per_packet: usize,
    packet_bytes: Vec<u8>,
    packets: Vec<Vec<u8>>
}

impl PacketList {

    pub fn new(max_bytes_per_packet: usize) -> PacketList {
        PacketList {
            max_bytes_per_packet: max_bytes_per_packet,
            packet_bytes: Vec::with_capacity(max_bytes_per_packet),
            packets: Vec::new()
        }
    }

    pub fn append_bytes(&mut self, mut bytes: Vec<u8>) {

        // Append the bytes to the current packet if they won't overflow...
        if self.packet_bytes.len() + bytes.len() <= self.max_bytes_per_packet {
            self.packet_bytes.append(&mut bytes);

        // ...otherwise use them to start the next packet
        } else {

            // Push the next packet with the previous packet bytes
            if !self.packet_bytes.is_empty() {
                self.packets.push(self.packet_bytes.drain(0..).collect());
            }

            // Start a new packet containing the overflowing entity bytes
            self.packet_bytes.append(&mut bytes);

        }

    }

    pub fn into_vec(mut self) -> Vec<Vec<u8>> {

        if !self.packet_bytes.is_empty() {
            self.packets.push(self.packet_bytes);
        }

        self.packets

    }

}


// Generic Helpers ------------------------------------------------------------
pub fn deserialize_entity_bytes(bytes: &[u8], overhead: usize) -> Option<(&[u8], usize)> {
    let bytes_length = bytes.len();
    if bytes_length < overhead {
        None

    } else {
        let entity_length = bytes[0] as usize;
        if bytes_length < entity_length + overhead {
            None

        } else {
            Some((&bytes[1..entity_length + overhead], entity_length + overhead))
        }
    }
}


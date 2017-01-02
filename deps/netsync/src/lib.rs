//! **netsync**
#![cfg_attr(feature="clippy", feature(plugin))]
#![cfg_attr(feature="clippy", plugin(clippy))]
#![deny(
    missing_debug_implementations,
    trivial_casts, trivial_numeric_casts,
    unsafe_code,
    unused_import_braces, unused_qualifications
)]


// STD Dependencies -----------------------------------------------------------
use std::fmt;
use std::cmp;
use std::collections::VecDeque;


// Traits ---------------------------------------------------------------------
pub trait NetworkInput: Default + fmt::Debug {
    fn tick(&self) -> u8;
    fn to_bytes(&self) -> Vec<u8>;
    fn from_bytes(&[u8]) -> Option<(usize, Self)> where Self: Sized;
}

pub trait NetworkProperty: Clone + Default + fmt::Debug {
    fn interpolate_from(&self, &Self, f32) -> Self;
    fn to_bytes(&self) -> Vec<u8>;
    fn from_bytes(&[u8]) -> Self where Self: Sized;
}

pub trait NetworkState<P: NetworkProperty, I: NetworkInput>: fmt::Debug + Default {
    fn new(usize) -> Self where Self: Sized;
    fn set(&mut self, base: P);
    fn update_with<F: FnMut(&mut P, Option<&P>, Option<&I>)>(
        &mut self,
        callback: F

    ) where Self: Sized;
}


// Client Side Logic ----------------------------------------------------------
#[derive(Default, Debug)]
pub struct ClientState<P: NetworkProperty, I: NetworkInput> {
    current: P,
    last: P,
    base: P,
    confirmed_tick: u8,
    confirmed_state: Option<P>,
    received_remote: bool,
    buffered_inputs: VecDeque<I>,
    input_buffer_size: usize
}

impl<P: NetworkProperty, I: NetworkInput> ClientState<P, I> {

    pub fn interpolate(&self, u: f32) -> P {
        self.current.interpolate_from(&self.last, u)
    }

    pub fn input(&mut self, input: I) {

        self.buffered_inputs.push_back(input);

        if self.buffered_inputs.len() > self.input_buffer_size {
            self.buffered_inputs.pop_front();
        }

    }

    pub fn send(&mut self) -> Vec<u8> {

        let mut serialized_inputs = Vec::new();
        for input in &self.buffered_inputs {
            serialized_inputs.extend(input.to_bytes());
        }

        serialized_inputs

    }

    pub fn receive(&mut self, bytes: &[u8], tick: Option<u8>) {

        let state = P::from_bytes(bytes);

        // Confirmed remote state for locally controlled state
        if let Some(tick) = tick {
            // TODO compare with local last tick to avoid receiving outdated state?
            // tick_is_more_recent(tick, self.confirmed_tick)
            self.confirmed_state = Some(state);
            self.confirmed_tick = tick;

        // Remote controlled state
        } else {
            self.confirmed_state = Some(state);
            self.confirmed_tick = 0;
        }

    }

    pub fn receive_with<F: FnMut(&P, &mut P)>(&mut self, bytes: &[u8], tick: Option<u8>, mut modifier: F) {

        let mut state = P::from_bytes(bytes);
        modifier(&self.current, &mut state);

        // Confirmed remote state for locally controlled state
        if let Some(tick) = tick {
            // TODO compare with local last tick to avoid receiving outdated state?
            // tick_is_more_recent(tick, self.confirmed_tick)
            self.confirmed_state = Some(state);
            self.confirmed_tick = tick;

        // Remote controlled state
        } else {
            self.received_remote = true;
            self.confirmed_state = Some(state);
            self.confirmed_tick = 0;
        }

    }

    pub fn force_update_with<F: FnMut(&mut P)>(
        &mut self,
        mut callback: F
    ) {

        self.last = self.current.clone();
        self.current = self.base.clone();

        // Apply unconfirmed inputs on top of last state confirmed by the server
        let mut new_state = self.base.clone();

        callback(&mut new_state);

        // Assign calculated state
        self.current = new_state;

    }

}

impl<P: NetworkProperty, I: NetworkInput> NetworkState<P, I> for ClientState<P, I> {

    fn new(buffer_size: usize) -> ClientState<P, I> {
        ClientState {
            current: P::default(),
            last: P::default(),
            base: P::default(),
            confirmed_tick: 0,
            confirmed_state: None,
            received_remote: false,
            buffered_inputs: VecDeque::new(),
            input_buffer_size: buffer_size
        }
    }

    fn set(&mut self, base: P) {
        self.base = base;
        self.current = self.base.clone();
        self.last = self.current.clone();
    }

    fn update_with<F: FnMut(&mut P, Option<&P>, Option<&I>)>(
        &mut self,
        mut callback: F
    ) {

        // Check if we have a newly confirmed server state
        if let Some(confirmed_state) = self.confirmed_state.take() {

            // Set the current state as the last state and wtaketkae over the
            // confirmed state as new base state
            self.last = self.current.clone();
            self.base = confirmed_state.clone();
            self.current = confirmed_state;

            // Drop all inputs confirmed by the server so the remaining ones
            // get applied on top of the new base state
            let confirmed_tick = self.confirmed_tick;
            self.buffered_inputs.retain(|input| {
                tick_is_more_recent(input.tick(), confirmed_tick)
            });

        // Otherwise reset the local state and re-apply the inputs on top of it
        } else {
            self.last = self.current.clone();
            self.current = self.base.clone();
        }

        let mut new_state = self.base.clone();

        // Update remote entities
        if self.received_remote {
            callback(&mut new_state, Some(&self.last), None);

        // Apply unconfirmed inputs on top of last state confirmed by the server
        } else {
            for input in &self.buffered_inputs {
                callback(&mut new_state, Some(&self.last), Some(input));
            }
        }

        // Assign calculated state
        self.current = new_state;

    }

}


// Server Side Logic ----------------------------------------------------------
#[derive(Default, Debug)]
pub struct ServerState<P: NetworkProperty, I: NetworkInput> {
    current: P,
    confirmed_tick: u8,
    buffered_inputs: VecDeque<I>,
    buffered_states: VecDeque<P>,
    state_buffer_size: usize,
    first_input: bool,
    last_input_tick: u8
}

impl<P: NetworkProperty, I: NetworkInput> ServerState<P, I> {

    pub fn get_relative(&self, ticks_ago: usize) -> P {

        let len = self.buffered_states.len();
        let ticks_ago = cmp::min(len, ticks_ago);
        if let Some(state) = self.buffered_states.get(len - ticks_ago) {
            state.clone()

        } else {
            self.current.clone()
        }

    }

    pub fn get_absolute(&self, tick: u8) -> P {
        let ticks_ago = cmp::max(0, self.last_input_tick as isize - tick as isize) as usize;
        self.get_relative(ticks_ago)
    }

    pub fn receive(&mut self, bytes: &[u8]) {
        let mut offset = 0;
        while let Some((size, input)) = I::from_bytes(&bytes[offset..]) {
            self.receive_input(input);
            offset += size;
        }
    }

    pub fn send(&self, delay: Option<usize>) -> Vec<u8> {

        if let Some(delay) = delay {
            self.get_relative(delay).to_bytes()

        } else {
            let mut bytes = vec![self.confirmed_tick];
            bytes.append(&mut self.current.to_bytes());
            bytes
        }

    }

    pub fn send_with<F: FnMut(&mut P)>(&self, delay: Option<usize>, mut modifier: F) -> Vec<u8> {

        if let Some(delay) = delay {
            let mut state = self.get_relative(delay);
            modifier(&mut state);
            state.to_bytes()

        } else {
            let mut bytes = vec![self.confirmed_tick];
            let mut state = self.current.clone();
            modifier(&mut state);
            bytes.append(&mut state.to_bytes());
            bytes
        }

    }

    fn receive_input(&mut self, input: I) {
        if self.first_input || tick_is_more_recent(input.tick(), self.last_input_tick) {
            self.first_input = false;
            self.last_input_tick = input.tick();
            self.buffered_inputs.push_back(input);
        }
    }

}

impl<P: NetworkProperty, I: NetworkInput> NetworkState<P, I> for ServerState<P, I> {

    fn new(buffer_size: usize) -> ServerState<P, I> {
        ServerState {
            current: P::default(),
            confirmed_tick: 0,
            buffered_inputs: VecDeque::new(),
            buffered_states: VecDeque::new(),
            state_buffer_size: buffer_size,
            first_input: true,
            last_input_tick: 0
        }
    }

    fn set(&mut self, state: P) {
        self.current = state;
    }

    fn update_with<F: FnMut(&mut P, Option<&P>, Option<&I>)>(
        &mut self,
        mut callback: F
    ) {

        // TODO this will never run if we don't have any input
        // TODO make input optional?
        while let Some(input) = self.buffered_inputs.pop_front() {
            self.confirmed_tick = input.tick();
            callback(&mut self.current, None, Some(&input));
        }

        self.buffered_states.push_back(self.current.clone());

        if self.buffered_states.len() > self.state_buffer_size {
            self.buffered_states.pop_front();
        }

    }

}


// Helpers --------------------------------------------------------------------
pub fn tick_is_more_recent(a: u8, b: u8) -> bool {
    (a > b) && (a - b <= 128) || (b > a) && (b - a > 128)
}


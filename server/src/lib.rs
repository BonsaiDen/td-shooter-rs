//! **server**
#![cfg_attr(feature="clippy", feature(plugin))]
#![cfg_attr(feature="clippy", plugin(clippy))]
#![deny(
    missing_debug_implementations,
    trivial_casts, trivial_numeric_casts,
    unsafe_code,
    unused_import_braces, unused_qualifications
)]


// Crates ---------------------------------------------------------------------
extern crate hexahydrate;
extern crate netsync;
extern crate cobalt;
extern crate shared;


// STD Dependencies -----------------------------------------------------------
use std::thread;


// External Dependencies ------------------------------------------------------
use cobalt::ConnectionID;


// Internal Dependencies ------------------------------------------------------
use ::entity::Entity;
use ::server::Server;
use shared::level::Level;


// Modules --------------------------------------------------------------------
mod server;
mod entity;


// Server Runner ---------------------------------------------------------------
pub fn run(
    updates_per_second: u64,
    addr: String,
    level: Level

) -> thread::JoinHandle<()> {

    thread::spawn(move || {

        let config = cobalt::Config {
            send_rate: updates_per_second as u32,
            .. cobalt::Config::default()
        };

        let mut network = cobalt::ServerStream::new(config);
        network.bind(addr.as_str()).expect("Failed to bind to address.");

        let mut server = Server::new(addr, updates_per_second);
        let mut entity_server = hexahydrate::Server::<Entity, ConnectionID>::new(
            (updates_per_second * 2) as usize
        );

        loop {
            server.update(&mut entity_server, &mut network, &level);
        }

    })

}


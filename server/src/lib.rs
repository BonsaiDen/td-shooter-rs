//! **server**
#![cfg_attr(feature="clippy", feature(plugin))]
#![cfg_attr(feature="clippy", plugin(clippy))]
#![deny(
    trivial_casts, trivial_numeric_casts,
    unsafe_code,
    unused_import_braces, unused_qualifications
)]


// Crates ---------------------------------------------------------------------
extern crate rand;
extern crate hyper;
extern crate clock_ticks;
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
use shared::Timer as SharedTimer;
use shared::level::Level;


// Modules --------------------------------------------------------------------
mod entity;
mod laser_beam;
mod server;


// Types ----------------------------------------------------------------------
pub type Timer = SharedTimer<Server, hexahydrate::Server<Entity, ConnectionID>, cobalt::ServerStream, Level>;


// Statics --------------------------------------------------------------------
pub const DEFAULT_LEVEL_DATA: &'static str = include_str!("../../editor/map.toml");


// Server Runner ---------------------------------------------------------------
pub fn run(addr: String) -> thread::JoinHandle<()> {

    let http_addr = addr.clone();
    thread::spawn(move || {
        let server = hyper::server::Server::http(http_addr.as_str()).unwrap();
        server.handle(|_: hyper::server::Request, res: hyper::server::Response| {
            res.send(&DEFAULT_LEVEL_DATA.to_string().into_bytes()).ok();

        }).ok();
    });

    thread::spawn(move || {

        let config = cobalt::Config {
            send_rate: shared::UPDATES_PER_SECOND as u32,
            packet_drop_threshold: 1500,
            connection_drop_threshold: 2000,
            .. cobalt::Config::default()
        };

        let level = Level::from_toml_string(DEFAULT_LEVEL_DATA);

        let mut network = cobalt::ServerStream::new(config);
        network.bind(addr.as_str()).expect("Failed to bind to address.");

        let mut timer = Timer::new();
        let mut server = Server::new(addr, shared::UPDATES_PER_SECOND);
        let mut entity_server = hexahydrate::Server::<Entity, ConnectionID>::new(
            (shared::UPDATES_PER_SECOND * 2) as usize
        );

        loop {
            server.update(&mut timer, &mut entity_server, &mut network, &level);
            timer.run(&mut server, &mut entity_server, &mut network, &level);
        }

    })

}


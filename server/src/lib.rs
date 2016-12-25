// Crates ---------------------------------------------------------------------
extern crate hexahydrate;
extern crate netsync;
extern crate cobalt;
extern crate shared;


// STD Dependencies -----------------------------------------------------------
use std::thread;


// Internal Dependencies ------------------------------------------------------
use ::server::Server;
use shared::level::Level;


// Modules --------------------------------------------------------------------
mod server;
mod entity;


// Server Runner ---------------------------------------------------------------
pub fn run(updates_per_second: u64, addr: String, level: Level) -> thread::JoinHandle<()> {

    thread::spawn(move || {

        let config = cobalt::Config {
            send_rate: updates_per_second as u32,
            .. cobalt::Config::default()
        };

        let mut handler = Server::new(updates_per_second, level);
        let mut server = cobalt::Server::new(config);
        server.bind(&mut handler, addr.as_str()).unwrap();

    })

}


// Crates ---------------------------------------------------------------------
#[cfg(feature = "loopback")]
extern crate server;
extern crate client;
extern crate shared;
extern crate cobalt;
extern crate clap;


// External Dependencies ------------------------------------------------------
use clap::{Arg, App};


// Internal Dependencies ------------------------------------------------------
use shared::level::Level;
use shared::UPDATES_PER_SECOND;


// Main Loop ------------------------------------------------------------------
fn main() {

    let matches = App::new("Shooter Client")
        .version("0.1")
        .author("Ivo Wetzel <ivo.wetzel@googlemail.com>")
        .about("A shooter game")
        .arg(Arg::with_name("client")
            .short("c")
            .long("client")
            .help("Starts the game in client mode.")
        )
        .arg(Arg::with_name("local")
            .short("l")
            .long("local")
            .help("Starts the game in local mode.")
        )
        .arg(Arg::with_name("addr")
            .short("a")
            .long("addr")
            .value_name("LISTENING_ADDR")
            .help("Specifies the host:port on which to listen for network traffic.")
            .takes_value(true)
            .default_value("127.0.0.1:7156")
        )
        .get_matches();

    if matches.occurrences_of("local") == 1 {
        #[cfg(feature = "loopback")]
        server::run(UPDATES_PER_SECOND, matches.value_of("addr").unwrap().to_string(), Level::load());
    }

    let mut network = cobalt::ClientStream::new(cobalt::Config {
        send_rate: UPDATES_PER_SECOND as u32,
        connection_init_threshold: 1000,
        .. Default::default()
    });

    network.connect(matches.value_of("addr").unwrap()).expect("Already connected!");

    client::run(UPDATES_PER_SECOND, network);


}


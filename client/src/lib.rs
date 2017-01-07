//! **client**
#![cfg_attr(feature="clippy", feature(plugin))]
#![cfg_attr(feature="clippy", plugin(clippy))]
#![deny(
    trivial_numeric_casts,
    unused_import_braces
)]


// Crates ---------------------------------------------------------------------
extern crate graphics;
extern crate piston;
extern crate renderer;

extern crate rand;
extern crate clock_ticks;

extern crate hexahydrate;
extern crate netsync;
extern crate cobalt;
extern crate shared;


// Modules --------------------------------------------------------------------
mod camera;
mod client;
mod effect;
mod entity;
mod level;


// Statics --------------------------------------------------------------------
const BASE_WIDTH: u32 = 800;
const BASE_HEIGHT: u32 = 600;


// External Dependencies ------------------------------------------------------
use cobalt::ConnectionID;
use piston::input::*;
use piston::event_loop::*;


// Internal Dependencies ------------------------------------------------------
use ::entity::{Entity, Registry};
use ::level::Level;
use ::renderer::Renderer;
use shared::UPDATES_PER_SECOND;
use shared::Timer as SharedTimer;
use shared::level::Level as SharedLevel;


// Re-Exports -----------------------------------------------------------------
pub use self::client::*;


// Types ----------------------------------------------------------------------
pub type Timer = SharedTimer<
    Client,
    hexahydrate::Client<Entity, ConnectionID, Registry>,
    cobalt::ClientStream,
    Level
>;


// Client Runner --------------------------------------------------------------
pub fn run(updates_per_second: u64, addr: &str) {

    // Create window and renderer
    let mut renderer = Renderer::new(
        "Shooter",
        BASE_WIDTH,
        BASE_HEIGHT,
        UPDATES_PER_SECOND
    );

    // Events
    let mut events = renderer.events();
    events.set_ups(UPDATES_PER_SECOND);
    events.set_max_fps(60);

    // Timer
    let mut timer = Timer::new();

    // Level
    let level = Level::new(SharedLevel::load());

    // Game Client
    let mut client = Client::new(addr, BASE_WIDTH, BASE_HEIGHT);
    let mut entity_client = hexahydrate::Client::<Entity, ConnectionID, Registry>::new(
        Registry,
        (updates_per_second * 2) as usize
    );

    // Network
    let mut network = cobalt::ClientStream::new(cobalt::Config {
        send_rate: UPDATES_PER_SECOND as u32,
        packet_drop_threshold: 2000,
        connection_drop_threshold: 5000,
        connection_init_threshold: 1500,
        .. Default::default()
    });

    network.connect(addr).expect("Already connected!");

    // Main Loop
    while let Some(e) = events.next(&mut renderer) {
        match e {
            Event::Input(ref event) => client.input(
                &mut timer,
                &mut renderer,
                &mut entity_client,
                &level,
                event
            ),
            Event::Update(update) => {
                client.update(
                    &mut timer,
                    &mut entity_client,
                    &mut network,
                    &level,
                    update.dt as f32
                );
                timer.run(&mut client, &mut entity_client, &mut network, &level);
            },
            Event::Render(args) => {
                renderer.begin(args);
                client.render(
                    &mut renderer, &mut entity_client, &level
                );
                renderer.end();
            },
            _ => { }
        }
    }

}


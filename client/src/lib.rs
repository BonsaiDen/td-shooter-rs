//! **client**
#![cfg_attr(feature="clippy", feature(plugin))]
#![cfg_attr(feature="clippy", plugin(clippy))]
#![deny(
    trivial_numeric_casts,
    unused_import_braces
)]


// Crates ---------------------------------------------------------------------
#[macro_use]
extern crate gfx;
extern crate gfx_window_glutin;
extern crate gfx_device_gl;
extern crate glutin_window;
extern crate shaders_graphics2d;
extern crate shader_version;
extern crate draw_state;
extern crate graphics;
extern crate piston;

extern crate clock_ticks;
extern crate hexahydrate;
extern crate netsync;
extern crate cobalt;
extern crate shared;
extern crate glutin;


// Modules --------------------------------------------------------------------
mod camera;
mod client;
mod effect;
mod entity;
mod level;
mod renderer;


// Statics --------------------------------------------------------------------
static BASE_WIDTH: f64 = 640.0;
static BASE_HEIGHT: f64 = 480.0;


// External Dependencies ------------------------------------------------------
use cobalt::ConnectionID;
use piston::input::*;
use piston::event_loop::*;


// Internal Dependencies ------------------------------------------------------
use renderer::Renderer;
use ::entity::{Entity, Registry};
use shared::UPDATES_PER_SECOND;
use shared::level::Level as SharedLevel;
use level::Level;


// Re-Exports -----------------------------------------------------------------
pub use self::client::*;


// Client Runner --------------------------------------------------------------
pub fn run(updates_per_second: u64, mut network: cobalt::ClientStream) {

    // Create window and renderer
    let mut renderer = Renderer::new(
        "Shooter",
        BASE_WIDTH as u32,
        BASE_HEIGHT as u32,
        UPDATES_PER_SECOND
    );

    // Events
    let mut events = renderer.events();
    events.set_ups(UPDATES_PER_SECOND);
    events.set_max_fps(60);

    // Level
    let level = Level::new(SharedLevel::load());

    // Game Client
    let mut client = Client::new(BASE_WIDTH, BASE_HEIGHT);
    let mut entity_client = hexahydrate::Client::<Entity, ConnectionID, Registry>::new(
        Registry,
        (updates_per_second * 2) as usize
    );

    // Main Loop
    while let Some(e) = events.next(&mut renderer) {
        match e {
            Event::Input(ref event) => client.input(
                &mut entity_client, &level, event
            ),
            Event::Update(update) => client.update(
                &mut entity_client, &mut network, &level, update.dt
            ),
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


//! **client**
#![cfg_attr(feature="clippy", feature(plugin))]
#![cfg_attr(feature="clippy", plugin(clippy))]
#![deny(
    trivial_numeric_casts,
    unsafe_code,
    unused_import_braces, unused_qualifications
)]


// Crates ---------------------------------------------------------------------
extern crate piston_window;
extern crate hexahydrate;
extern crate netsync;
extern crate cobalt;
extern crate shared;
extern crate glutin;


// Modules --------------------------------------------------------------------
mod camera;
mod client;
mod level;
mod entity;


// Statics --------------------------------------------------------------------
static BASE_WIDTH: f64 = 640.0;
static BASE_HEIGHT: f64 = 480.0;


// External Dependencies ------------------------------------------------------
use piston_window::{Event, Events, EventLoop, PistonWindow, WindowSettings};


// Internal Dependencies ------------------------------------------------------
use shared::UPDATES_PER_SECOND;
use shared::level::Level as SharedLevel;
use level::Level;


// Re-Exports -----------------------------------------------------------------
pub use self::client::*;


// Client Runner --------------------------------------------------------------
pub fn run(updates_per_second: u64, mut network: cobalt::ClientStream) {

    // Create Window
    let mut window: PistonWindow = WindowSettings::new(
            "Shooter",
            [BASE_WIDTH as u32, BASE_HEIGHT as u32]
        )
        .vsync(false)
        .exit_on_esc(true)
        .build()
        .unwrap();

    // Hide Cursor
    window.window.window.set_cursor_state(glutin::CursorState::Hide).ok();

    // Events
    let mut events = window.events();
    events.set_ups(UPDATES_PER_SECOND);

    // Level and Game
    let level = Level::new(SharedLevel::load());
    let mut client = Client::new(updates_per_second, BASE_WIDTH, BASE_HEIGHT);

    // Gameloop
    while let Some(e) = events.next(&mut window) {
        match e {
            Event::Input(ref event) => client.input(event),
            Event::Update(update) => client.update(update.dt, &level, &mut network),
            Event::Render(args) => client.draw_2d(&mut window, &e, &args, &level),
            _ => { }
        }
        window.event(&e);
    }

}


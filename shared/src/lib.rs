//! **shared**
#![cfg_attr(feature="clippy", feature(plugin))]
#![cfg_attr(feature="clippy", plugin(clippy))]
#![deny(
    missing_debug_implementations,
    trivial_casts, trivial_numeric_casts,
    unsafe_code,
    unused_import_braces, unused_qualifications
)]


// Crates ---------------------------------------------------------------------
extern crate rustc_serialize;
extern crate clock_ticks;
extern crate bincode;
extern crate toml;

extern crate hexahydrate;
extern crate netsync;
extern crate cobalt;


// Statics --------------------------------------------------------------------
pub static UPDATES_PER_SECOND: u64 = 30;


// Modules --------------------------------------------------------------------
pub mod util;
pub mod level;
pub mod color;
pub mod action;
pub mod entity;


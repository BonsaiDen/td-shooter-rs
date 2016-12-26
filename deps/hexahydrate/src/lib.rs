//! **hexahydrate**
#![cfg_attr(feature="clippy", feature(plugin))]
#![cfg_attr(feature="clippy", plugin(clippy))]
#![deny(
    missing_debug_implementations,
    trivial_casts, trivial_numeric_casts,
    unsafe_code,
    unused_import_braces, unused_qualifications
)]


// Crates ---------------------------------------------------------------------
#[macro_use]
extern crate lazy_static;


// Macros ---------------------------------------------------------------------
macro_rules! state_machine {
    (
        $name:ident,
        {
            $( $method_name:ident : $($x:path)|* => $y:path ),*,
        }
    ) => (
        impl $name {$(
            pub fn $method_name(&mut self) -> bool {
                match *self {
                    $(
                        $x => {
                            *self = $y;
                            true
                        }
                     )*
                    _ => false
                }
            }
        )*}
    );
}

macro_rules! vec_with_default {
    ($value:expr ; $size:expr) => {
        {
            let mut items = Vec::with_capacity($size);
            for _ in 0..$size {
                items.push($value);
            }
            items
        }
    };
}


// Modules --------------------------------------------------------------------
mod shared;
mod server;
mod client;


// Re-Exports -----------------------------------------------------------------
pub const NETWORK_BYTE_OFFSET: u8 = 6;
pub use self::shared::{Entity, EntityRegistry};
pub use server::{Server, ConnectionSlot, EntitySlot as ServerEntitySlot, Error as ServerError};
pub use client::{Client, EntitySlot as ClientEntitySlot, Error as ClientError};


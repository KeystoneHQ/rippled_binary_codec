#![no_std]
#![feature(error_in_core)]
#[macro_use]
extern crate alloc;
extern crate core;

#[cfg(test)]
#[macro_use]
extern crate std;

pub mod definition_fields;
pub mod types;
pub mod serialize;
pub mod errors;
pub mod ripple_address_codec;

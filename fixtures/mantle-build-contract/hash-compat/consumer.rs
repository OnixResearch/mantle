//! Independent test consumers. No host capability or runtime is exposed.
#![no_std]

pub use mantle_build_contract::REQUEST_SCHEMA;

#[cfg(test)]
extern crate std;
#[cfg(test)]
mod controls;

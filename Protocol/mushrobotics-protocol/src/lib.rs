//!
//! The Mushrobotics-protocol library contains necessary traits and helpers to make
//! utilizing the protocol outlined in the README.md easier to implement.
//! 

#![no_std]

extern crate alloc;

pub mod packet;
pub use packet::*;


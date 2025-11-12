#![no_std]
#![forbid(unsafe_code)]
#![doc = include_str!("../README.md")]

extern crate alloc;

#[cfg(doc)]
extern crate std;

mod generic;

pub mod rc;
pub mod sync;


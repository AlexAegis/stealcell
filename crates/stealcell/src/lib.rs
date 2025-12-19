#![cfg_attr(all(feature = "no_std", not(test)), no_std)]

mod stealcell;

pub use stealcell::*;

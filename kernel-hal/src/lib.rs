//! Hardware Abstraction Layer

#![cfg_attr(not(feature = "libos"), no_std)]
#![feature(asm)]
#![deny(warnings)]

extern crate alloc;

mod common;

pub use common::{defs::*, future, user};

cfg_if::cfg_if! {
    if #[cfg(feature = "libos")] {
        #[macro_use]
        extern crate log;
        mod libos;
        pub use self::libos::*;
    } else {
        mod unimp;
        pub use self::unimp::*;
    }
}

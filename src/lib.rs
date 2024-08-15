#![cfg_attr(not(test), no_std)]

mod smtp;
pub use smtp::*;

pub mod auth;

mod io;
mod nb_fut;

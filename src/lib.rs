#![cfg_attr(not(test), no_std)]

pub mod auth;
pub mod smtp;

mod io;
mod nb_fut;

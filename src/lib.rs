#![cfg_attr(not(test), no_std)]

pub mod auth;
pub mod message;
pub mod smtp;

mod io;

#[allow(unused)]
mod nb_fut;

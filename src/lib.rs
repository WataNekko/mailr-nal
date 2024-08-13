#![no_std]

mod smtp;
pub use smtp::*;

pub mod auth;

mod nb_fut;

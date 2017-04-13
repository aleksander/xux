#![feature(associated_consts)]
#![recursion_limit = "1024"]

#[macro_use]
extern crate log;
extern crate byteorder;
extern crate openssl;
#[macro_use]
extern crate error_chain;
extern crate ncurses;
extern crate rustc_serialize;
extern crate image;

pub mod errors;
pub mod proto;
pub mod state;

pub mod driver;
pub mod driver_std;
// FIXME pub mod driver_mio;

pub mod ai;
pub mod ai_decl;
// FIXME pub mod ai_lua;

pub mod client;
mod render;
mod web;
mod util;
mod shift_to_unsigned;

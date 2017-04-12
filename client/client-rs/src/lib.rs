#![feature(associated_consts)]
#![recursion_limit = "1024"]
#![feature(zero_one)]

#[macro_use]
extern crate log;
extern crate byteorder;

pub mod proto;
pub mod state;

pub mod driver;
pub mod driver_std;
// FIXME pub mod driver_mio;

pub mod ai;
pub mod ai_decl;
// FIXME pub mod ai_lua;

//pub mod error;
//pub use error::Error;
#[macro_use]
extern crate error_chain;
mod errors {
    // Create the Error, ErrorKind, ResultExt, and Result types
    error_chain! { }
}
use errors::*;

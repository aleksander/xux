#![feature(convert)]
#![feature(read_exact)]
//#![feature(zero_one)]
#![feature(ip_addr)]

#[macro_use]
extern crate log;

pub mod message;
pub mod state;

pub mod driver;
pub mod driver_std;
//FIXME pub mod driver_mio;

pub mod ai;
pub mod ai_decl;
//FIXME pub mod ai_lua;


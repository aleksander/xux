#![feature(associated_consts)]
#![recursion_limit = "1024"]

extern crate openssl;
extern crate rustc_serialize;
extern crate byteorder;
#[macro_use]
extern crate error_chain;
extern crate image;
extern crate ncurses;
// extern crate cgmath;
// extern crate camera_controllers;
// #[macro_use]
// extern crate glium;
// extern crate lua;

mod state;
mod proto;
mod ai;
mod ai_decl;
mod errors;
// TODO #[cfg(driver = "std")]
// #[cfg(feature = "driver_std")]
mod driver_std;
mod web;
mod render;
mod shift_to_unsigned;
mod driver;
mod util;
mod client;

use ai::Ai;
use errors::*;

// TODO #[cfg(ai = "lua")]

// #[cfg(ai_lua)]
//mod ai_lua;
// #[cfg(ai_lua)]
// FIXME use ai_lua::LuaAi;
// #[cfg(ai_lua)]
// type AiImpl = LuaAi;

// TODO #[cfg(ai = "decl")]

// #[cfg(feature = "ai_decl")]
// #[cfg(feature = "ai_decl")]
use ai_decl::AiDecl;
// #[cfg(feature = "ai_decl")]
// type AiImpl = AiDecl;

// TODO #[cfg(driver = "mio")]
// #[cfg(driver_mio)]
//FIXME BROKEN! mod driver_mio;

// #[cfg(feature = "driver_std")]
use driver_std::DriverStd;

// TODO
// extern crate nix;
// use nix::sys::socket::setsockopt;
// use nix::sys::socket::SockLevel;
// use nix::sys::socket::SockOpt;
//
// #[derive(Debug,Copy,Clone)]
// struct BindToDevice {
//     dev_name: &'static str
// }
//
// impl BindToDevice {
//     fn new (dev_name: &'static str) -> BindToDevice {
//         BindToDevice{ dev_name: dev_name}
//     }
// }
//
// impl SockOpt for BindToDevice {
//     type Val = &'static str;
//     fn get (&self, fd: RawFd, level: c_int) -> Result<&'static str> { ... }
//     fn set () -> ? { ... }
// }
//
// //char *opt;
// //opt = "eth0";
// //setsockopt(sd, SOL_SOCKET, SO_BINDTODEVICE, opt, 4);
// nix::sys::socket::setsockopt(sock.as_raw_fd, SockLevel::Socket, BindToDevice::new("wlan0"));

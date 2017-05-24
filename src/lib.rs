#![feature(inclusive_range_syntax)]
#![feature(associated_consts)]
#![recursion_limit = "1024"]
#![feature(struct_field_attributes)]

#![warn(trivial_casts)]
#![warn(trivial_numeric_casts)]
#![warn(unused_import_braces)]
#![warn(unused_qualifications)]
//#![warn(unused_results)]
#![warn(unused_extern_crates)]
//#![warn(variant_size_differences)]

#[macro_use]
extern crate log;
extern crate byteorder;
extern crate openssl;
#[macro_use]
extern crate error_chain;
extern crate ncurses;
extern crate rustc_serialize;
extern crate image;
// extern crate cgmath;
// extern crate camera_controllers;
// #[macro_use]
// extern crate glium;
// extern crate lua;
extern crate piston_window;
//extern crate crossbeam;
extern crate deque;
extern crate chrono;

pub mod errors;
pub mod proto;
pub mod state;

pub mod driver;
// TODO #[cfg(driver = "std")]
// #[cfg(feature = "driver_std")]
pub mod driver_std;
// FIXME pub mod driver_mio;

pub mod ai;
pub mod ai_decl;
// FIXME pub mod ai_lua;

pub mod client;
pub mod render;
mod web;
mod util;
mod shift_to_unsigned;

// TODO #[cfg(ai = "lua")]
// #[cfg(ai_lua)]
//mod ai_lua;

// #[cfg(ai_lua)]
// FIXME use ai_lua::LuaAi;
// #[cfg(ai_lua)]
// type AiImpl = LuaAi;

// TODO #[cfg(ai = "decl")]
// #[cfg(feature = "ai_decl")]
//use ai_decl::AiDecl;
// #[cfg(feature = "ai_decl")]
// type AiImpl = AiDecl;

// TODO #[cfg(driver = "mio")]
// #[cfg(driver_mio)]
//FIXME BROKEN! mod driver_mio;

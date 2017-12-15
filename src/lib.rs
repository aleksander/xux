#![feature(inclusive_range_syntax)]
#![warn(unused_import_braces)]
#![warn(unused_qualifications)]
#![warn(unused_extern_crates)]

#[macro_use]
extern crate failure;
#[macro_use]
extern crate log;
extern crate byteorder;
extern crate openssl;
#[cfg(feature = "render_text")]
extern crate ncurses;
extern crate image;
#[cfg(feature = "render_2d_piston")]
extern crate piston_window;
#[cfg(feature = "render_2d_piston")]
extern crate gfx_graphics;
extern crate chrono;
#[macro_use]
extern crate serde_derive;
#[cfg(feature = "dump_events")]
extern crate bincode;
#[cfg(feature = "render_2d_gfx")]
#[macro_use]
extern crate gfx;
#[cfg(feature = "render_2d_gfx")]
extern crate gfx_window_glutin;
#[cfg(feature = "render_2d_gfx")]
extern crate glutin;
#[cfg(feature = "render_2d_gfx")]
extern crate cgmath;
#[cfg(feature = "render_2d_gfx")]
extern crate imgui;
#[cfg(feature = "render_2d_gfx")]
extern crate imgui_gfx_renderer;
#[cfg(feature = "render_2d_gfx")]
extern crate gfx_device_gl;
#[cfg(feature = "render_2d_gfx")]
extern crate ron;
extern crate flate2;

pub mod proto;
mod state;

mod driver;

mod ai;

pub mod client;
mod render;
mod util;
mod shift_to_unsigned;
mod widgets;

pub type Result<T> = std::result::Result<T, failure::Error>;

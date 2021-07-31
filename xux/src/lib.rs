#![warn(unused_import_braces)]
#![warn(unused_qualifications)]
#![warn(unused_extern_crates)]
#![feature(buf_read_has_data_left)]

pub mod proto;
pub mod state;
pub mod driver;
mod ai;
pub mod client;
//mod render;
mod util;
//mod shift_to_unsigned;
mod widgets;

pub type Result<T> = std::result::Result<T, failure::Error>;
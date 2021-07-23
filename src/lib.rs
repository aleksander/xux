#![warn(unused_import_braces)]
#![warn(unused_qualifications)]
#![warn(unused_extern_crates)]

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

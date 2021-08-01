#![warn(unused_import_braces)]
#![warn(unused_qualifications)]
#![warn(unused_extern_crates)]
#![feature(buf_read_has_data_left)]

pub mod proto;
pub mod state;
pub mod driver;
mod ai;
pub mod client;
mod widgets;

pub type Result<T> = anyhow::Result<T>;
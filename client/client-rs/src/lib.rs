//<<<<<<< 4b4fc349b887fbcbfa197fe2b798f0d378433edf
//#![feature(convert)]
//#![feature(read_exact)]
//#![feature(zero_one)]
//#![feature(ip_addr)]
//
//=======
//>>>>>>> compilation fix
#[macro_use]
extern crate log;
extern crate byteorder;

pub mod message;
pub mod state;

pub mod driver;
pub mod driver_std;
//FIXME pub mod driver_mio;

pub mod ai;
pub mod ai_decl;
//FIXME pub mod ai_lua;


pub mod msg_list;
pub mod serialization;
pub mod message;
pub mod message_sess;
pub mod message_rel;
pub mod message_ack;
pub mod message_beat;
pub mod message_mapreq;
pub mod message_mapdata;
pub mod message_objdata;
pub mod message_objack;
pub mod message_close;

pub use proto::msg_list::*;
pub use proto::serialization::*;
pub use proto::message::*;
pub use proto::message_sess::*;
pub use proto::message_rel::*;
pub use proto::message_ack::*;
pub use proto::message_beat::*;
pub use proto::message_mapreq::*;
pub use proto::message_mapdata::*;
pub use proto::message_objdata::*;
pub use proto::message_objack::*;
pub use proto::message_close::*;

pub type Coord = (i32, i32);
//TODO
//#[cfg(feature = "salem")]
//ObjXY = (i32,i32)
//GridXY = (i32,i32)
//#[cfg(feature = "hafen")]
//ObjXY = (f64,f64)
//GridXY = (i32,i32)
pub type Color = (u8, u8, u8, u8);
pub type ObjID = u32;
pub type ResID = u16;

pub const POSRES: f64 = 1.0 / 1024.0 * 11.0;


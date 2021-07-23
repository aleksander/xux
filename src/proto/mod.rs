use serde::{Serialize, Deserialize};

pub use crate::proto::list::*;
pub use crate::proto::serialization::*;
pub use crate::proto::message::*;
pub use crate::proto::message_sess::*;
pub use crate::proto::message_rel::*;
pub use crate::proto::message_ack::*;
pub use crate::proto::message_beat::*;
pub use crate::proto::message_mapreq::*;
pub use crate::proto::message_mapdata::*;
pub use crate::proto::message_objdata::*;
pub use crate::proto::message_objack::*;
pub use crate::proto::message_close::*;

pub mod list;
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

#[cfg(feature = "salem")]
#[derive(Serialize,Deserialize,Debug,Clone,Copy)]
pub struct ObjXY(pub i32, pub i32);
#[cfg(feature = "hafen")]
#[derive(Serialize,Deserialize,Debug,Clone,Copy)]
pub struct ObjXY(pub f64, pub f64);

impl ObjXY {
    #[cfg(feature = "salem")]
    pub fn new() -> ObjXY {
        ObjXY(0, 0)
    }

    #[cfg(feature = "hafen")]
    pub fn new() -> ObjXY {
        ObjXY(0.0, 0.0)
    }

    #[cfg(feature = "salem")]
    pub fn grid(self) -> GridXY {
        let gx = self.0 / 1100;
        let gy = self.1 / 1100;
        (if self.0 < 0 { gx - 1 } else { gx }, if self.1 < 0 { gy - 1 } else { gy })
    }

    #[cfg(feature = "hafen")]
    pub fn grid(self) -> GridXY {
        let gx = (self.0 / 1100.0) as i32;
        let gy = (self.1 / 1100.0) as i32;
        (if self.0 < 0.0 { gx - 1 } else { gx }, if self.1 < 0.0 { gy - 1 } else { gy })
    }
}

impl From<(i32,i32)> for ObjXY {
    #[cfg(feature = "salem")]
    fn from((x,y): (i32,i32)) -> Self {
        ObjXY(x,y)
    }

    #[cfg(feature = "hafen")]
    fn from((x,y): (i32,i32)) -> Self {
        ObjXY(x as f64 * POSRES,y as f64 * POSRES)
    }
}

impl Into<(i32,i32)> for ObjXY {
    #[cfg(feature = "salem")]
    fn into(self) -> (i32,i32) {
        (self.0, self.1)
    }

    #[cfg(feature = "hafen")]
    fn into(self) -> (i32,i32) {
        ((self.0 / POSRES) as i32, (self.1 / POSRES) as i32)
    }
}
pub type GridXY = (i32,i32);
pub type Color = (u8, u8, u8, u8);
pub type ObjID = u32;
pub type ResID = u16;

pub const POSRES: f64 = 1.0 / 1024.0 * 11.0;


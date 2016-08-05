pub mod msg_list;
pub mod serialization;
pub mod message_sess;
pub mod message_rel;

pub mod message_ack {
    #[derive(Debug)]
    pub struct Ack {
        pub seq: u16,
    }
}

pub mod message_mapreq {
    #[derive(Debug)]
    pub struct MapReq {
        pub x: i32,
        pub y: i32,
    }
}

pub mod message_mapdata {
    use std::fmt;

    pub struct MapData {
        pub pktid: i32,
        pub off: u16,
        pub len: u16,
        pub buf: Vec<u8>,
    }

    impl fmt::Debug for MapData {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "MAPDATA pktid:{} offset:{} len:{} buf:[..{}]", self.pktid, self.off, self.len, self.buf.len())
        }
    }
}

pub mod message_objdata;
pub mod message_objack;
pub mod message;

//pub use proto::message_ack::*;
//pub use proto::message_mapreq::*;
//pub use proto::message_mapdata::*;
//pub use proto::message_objdata::*;
//pub use proto::message_objack::*;
//pub use proto::message::*;

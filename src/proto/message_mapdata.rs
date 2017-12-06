use std::fmt;
use proto::serialization::*;
use Result;

pub struct MapData {
    pub pktid: i32,
    pub off: u16,
    pub len: u16,
    pub buf: Vec<u8>,
}

impl MapData {
    pub const ID: u8 = 5;

    pub fn new (pktid: i32, off: u16, len: u16, buf: Vec<u8>) -> MapData {
        MapData {
            pktid: pktid,
            off: off,
            len: len,
            buf: buf
        }
    }

    // TODO impl FromBuf for MapData {}
    pub fn from_buf <R:ReadBytesSac> (r: &mut R) -> Result<MapData> {
        Ok(MapData {
            pktid: r.i32()?,
            off: r.u16()?,
            len: r.u16()?,
            buf: {
                let mut buf = Vec::new();
                r.read_to_end(&mut buf)?;
                buf
            },
        })
    }
}

impl fmt::Debug for MapData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "MAPDATA pktid:{} offset:{} len:{} buf:[..{}]", self.pktid, self.off, self.len, self.buf.len())
    }
}

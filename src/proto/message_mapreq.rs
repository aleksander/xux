use proto::serialization::*;
use Result;

#[derive(Debug)]
pub struct MapReq {
    pub x: i32,
    pub y: i32,
}

impl MapReq {
    pub const ID: u8 = 4;

    pub fn new(x: i32, y: i32) -> MapReq {
        MapReq { x: x, y: y }
    }

    // TODO impl FromBuf for MapReq {}
    pub fn from_buf <R:ReadBytesSac> (r: &mut R) -> Result<MapReq> {
        Ok(MapReq {
            x: r.i32()?,
            y: r.i32()?,
        })
    }
}

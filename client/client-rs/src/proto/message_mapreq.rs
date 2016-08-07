use proto::serialization::*;
use Error;

#[derive(Debug)]
pub struct MapReq {
    pub x: i32,
    pub y: i32,
}

impl MapReq {
    // TODO impl FromBuf for MapReq {}
    pub fn from_buf <R:ReadBytesSac> (r: &mut R) -> Result<MapReq,Error> {
        Ok(MapReq {
            x: r.i32()?,
            y: r.i32()?,
        })
    }
}

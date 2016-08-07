use proto::serialization::*;
use Error;

#[derive(Debug)]
pub struct Ack {
    pub seq: u16,
}

impl Ack {
    // TODO impl FromBuf for Ack {}
    pub fn from_buf <R:ReadBytesSac> (r: &mut R) -> Result<Ack,Error> {
        Ok(Ack { seq: r.u16()? })
    }
}

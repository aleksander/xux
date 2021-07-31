use crate::proto::serialization::*;
use crate::Result;

#[derive(Debug)]
pub struct Ack {
    pub seq: u16,
}

impl Ack {
    pub const ID: u8 = 2;

    pub fn new(seq: u16) -> Ack {
        Ack { seq: seq }
    }

    // TODO impl FromBuf for Ack {}
    pub fn from_buf <R:ReadBytesSac> (r: &mut R) -> Result<Ack> {
        Ok(Ack { seq: r.u16()? })
    }

    pub fn to_buf <W:WriteBytesSac> (&self, w: &mut W) -> Result<()> {
        w.u8(Self::ID)?;
        w.u16(self.seq)?;
        Ok(())
    }
}

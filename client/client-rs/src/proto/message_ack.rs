use proto::serialization::*;
use Error;

#[derive(Debug)]
pub struct Ack {
    pub seq: u16,
}

impl Ack {
    pub const ID: u8 = 2;

    // TODO impl FromBuf for Ack {}
    pub fn from_buf <R:ReadBytesSac> (r: &mut R) -> Result<Ack,Error> {
        Ok(Ack { seq: r.u16()? })
    }

    pub fn to_buf <W:WriteBytesSac> (&self, w: &mut W) -> Result<(), Error> {
        w.u8(Self::ID)?;
        w.u16(self.seq)?;
        Ok(())
    }
}

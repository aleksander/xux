use proto::message_sess::*;
use proto::message_rel::*;
use proto::message_ack::*;
use proto::message_beat::*;
use proto::message_mapreq::*;
use proto::message_mapdata::*;
use proto::message_objdata::*;
use proto::message_objack::*;
use proto::message_close::*;
use proto::serialization::*;
use Error;

#[derive(Debug)]
pub enum ClientMessage {
    SESS( cSess ),
    REL( Rel ),
    ACK( Ack ),
    BEAT( Beat ),
    MAPREQ( MapReq ),
    OBJACK( ObjAck ),
    CLOSE( Close ),
}

#[derive(Debug)]
pub enum ServerMessage {
    SESS( sSess ),
    REL( Rel ),
    ACK( Ack ),
    MAPDATA( MapData ),
    OBJDATA( ObjData ),
    CLOSE( Close ),
}

//pub const SESS: u8 = 0;
//pub const REL: u8 = 1;
//pub const ACK: u8 = 2;
//pub const BEAT: u8 = 3;
//pub const MAPREQ: u8 = 4;
//pub const MAPDATA: u8 = 5;
//pub const OBJDATA: u8 = 6;
//pub const OBJACK: u8 = 7;
//pub const CLOSE: u8 = 8;

impl ClientMessage {
    // TODO impl FromBuf for ClientMessage
    pub fn from_buf <R:ReadBytesSac> (r: &mut R) -> Result<(ClientMessage, Option<Vec<u8>>), Error> {
        let msg = match r.u8()? {
            cSess::ID => Ok(ClientMessage::SESS(cSess::from_buf(r)?)),
            Rel::ID => Ok(ClientMessage::REL(Rel::from_buf(r)?)),
            Ack::ID => Ok(ClientMessage::ACK(Ack::from_buf(r)?)),
            Beat::ID => Ok(ClientMessage::BEAT(Beat)),
            MapReq::ID => Ok(ClientMessage::MAPREQ(MapReq::from_buf(r)?)),
            ObjAck::ID => Ok(ClientMessage::OBJACK(ObjAck::from_buf(r)?)),
            Close::ID => Ok(ClientMessage::CLOSE(Close)),
            _ => {
                Err(Error {
                    source: "unknown message type",
                    detail: None,
                })
            }
        }?;

        let mut tmp = Vec::new();
        r.read_to_end(&mut tmp)?;
        let remains = if tmp.is_empty() {
            None
        } else {
            Some(tmp)
        };

        Ok((msg, remains))
    }

    pub fn to_buf <W:WriteBytesSac> (&self, w: &mut W) -> Result<(), Error> {
        match *self {
            ClientMessage::SESS(ref sess) => sess.to_buf(w),
            ClientMessage::ACK(ref ack) => ack.to_buf(w),
            ClientMessage::BEAT(_) => {
                w.u8(Beat::ID)?;
                Ok(())
            }
            ClientMessage::REL(ref rel) => {
                w.u8(Rel::ID)?;
                w.u16(rel.seq)?; // sequence
                for i in 0 .. rel.rel.len() {
                    let rel_elem = &rel.rel[i];
                    let last_one = i == (rel.rel.len() - 1);
                    let rel_elem_buf = rel_elem.to_buf(last_one)?;
                    w.write(&rel_elem_buf)?;
                }
                Ok(())
            }
            ClientMessage::MAPREQ(ref mapreq) => {
                w.u8(MapReq::ID)?;
                w.i32(mapreq.x)?;
                w.i32(mapreq.y)?;
                Ok(())
            }
            ClientMessage::OBJACK(ref objack) => {
                w.u8(ObjAck::ID)?;
                for o in &objack.obj {
                    w.u32(o.id)?;
                    w.i32(o.frame)?;
                }
                Ok(())
            }
            ClientMessage::CLOSE(_) => {
                w.u8(Close::ID)?;
                Ok(())
            }
        }
    }
}

impl ServerMessage {
    // TODO impl FromBuf for ServerMessage
    pub fn from_buf <R:ReadBytesSac> (r: &mut R) -> Result<(ServerMessage, Option<Vec<u8>>), Error> {
        let mtype = r.u8()?;
        let res = match mtype {
            sSess::ID => Ok(ServerMessage::SESS(sSess::from_buf(r)?)),
            Rel::ID => Ok(ServerMessage::REL(Rel::from_buf(r)?)),
            Ack::ID => Ok(ServerMessage::ACK(Ack::from_buf(r)?)),
            MapData::ID => Ok(ServerMessage::MAPDATA(MapData::from_buf(r)?)),
            ObjData::ID => Ok(ServerMessage::OBJDATA(ObjData::from_buf(r)?)),
            Close::ID => Ok(ServerMessage::CLOSE(Close)),
            _ => {
                Err(Error {
                    source: "unknown message type",
                    detail: None,
                })
            }
        };

        let mut tmp = Vec::new();
        r.read_to_end(&mut tmp)?;
        let remains = if tmp.is_empty() {
            None
        } else {
            Some(tmp)
        };

        match res {
            Ok(msg) => Ok((msg, remains)),
            Err(e) => Err(e),
        }
    }

    /*
    pub fn to_buf <W:WriteBytesSac> (&self, w: &mut W) -> Result<(), Error> {
        match *self {
            ServerMessage::SESS(ref sess) => sess.to_buf(w),
            ServerMessage::ACK(ref ack) => {
                w.u8(ACK)?;
                w.u16(ack.seq)?;
                Ok(())
            }
            ServerMessage::REL(ref rel) => {
                w.u8(REL)?;
                w.u16(rel.seq)?; // sequence
                for i in 0 .. rel.rel.len() {
                    let rel_elem = &rel.rel[i];
                    let last_one = i == (rel.rel.len() - 1);
                    let rel_elem_buf = rel_elem.to_buf(last_one)?;
                    w.write(&rel_elem_buf)?;
                }
                Ok(())
            }
            ServerMessage::CLOSE => {
                w.u8(CLOSE)?;
                Ok(())
            }
            _ => {
                Err( Error{ source:"unknown message type", detail:None } )
            }
        }
    }
    */
}

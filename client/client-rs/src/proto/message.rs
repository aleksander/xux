use proto::message_sess::*;
use proto::message_rel::*;
use proto::message_ack::*;
use proto::message_mapreq::*;
use proto::message_mapdata::*;
use proto::message_objdata::*;
use proto::message_objack::*;
use Error;
//use std::io::Cursor;
use proto::serialization::*;
//use std::io::Read;

/*
#[allow(non_camel_case_types)]
#[derive(Debug)]
pub enum Message {
    C_SESS(cSess),
    S_SESS(sSess),
    REL(Rel),
    ACK(Ack),
    BEAT,
    MAPREQ(MapReq),
    MAPDATA(MapData),
    OBJDATA(ObjData),
    OBJACK(ObjAck),
    CLOSE,
}
*/

#[derive(Debug)]
pub enum ClientMessage {
    SESS( cSess ),
    REL( Rel ),
    ACK( Ack ),
    BEAT,
    MAPREQ( MapReq ),
    OBJACK( ObjAck ),
    CLOSE,
}

#[derive(Debug)]
pub enum ServerMessage {
    SESS( sSess ),
    REL( Rel ),
    ACK( Ack ),
    MAPDATA( MapData ),
    OBJDATA( ObjData ),
    CLOSE,
}

// TODO impl FromBuf for ServerMessage
// TODO impl ToBuf for ClientMessage

pub const SESS: u8 = ID;
const REL: u8 = 1;
const ACK: u8 = 2;
const BEAT: u8 = 3;
const MAPREQ: u8 = 4;
const MAPDATA: u8 = 5;
const OBJDATA: u8 = 6;
const OBJACK: u8 = 7;
const CLOSE: u8 = 8;

impl ClientMessage {
    pub fn from_buf <R:ReadBytesSac> (r: &mut R) -> Result<(ClientMessage, Option<Vec<u8>>), Error> {
        let mtype = r.u8()?;
        let res = match mtype {
            SESS => Ok(ClientMessage::SESS(cSess::from_buf(r)?)),
            REL => Ok(ClientMessage::REL(Rel::from_buf(r)?)),
            ACK => Ok(ClientMessage::ACK(Ack::from_buf(r)?)),
            BEAT => Ok(ClientMessage::BEAT),
            MAPREQ => Ok(ClientMessage::MAPREQ(MapReq::from_buf(r)?)),
            OBJACK => Ok(ClientMessage::OBJACK(ObjAck::from_buf(r)?)),
            CLOSE => Ok(ClientMessage::CLOSE),
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

    pub fn to_buf <W:WriteBytesSac> (&self, w: &mut W) -> Result<(), Error> {
        match *self {
            ClientMessage::SESS(ref sess) => sess.to_buf(w),
            //ClientMessage::S_SESS(ref s_sess) => s_sess.to_buf(w),
            ClientMessage::ACK(ref ack) => {
                w.u8(ACK)?;
                w.u16(ack.seq)?;
                Ok(())
            }
            ClientMessage::BEAT => {
                w.u8(BEAT)?;
                Ok(())
            }
            ClientMessage::REL(ref rel) => {
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
            ClientMessage::MAPREQ(ref mapreq) => {
                w.u8(MAPREQ)?;
                w.i32(mapreq.x)?;
                w.i32(mapreq.y)?;
                Ok(())
            }
            ClientMessage::OBJACK(ref objack) => {
                w.u8(OBJACK)?;
                for o in &objack.obj {
                    w.u32(o.id)?;
                    w.i32(o.frame)?;
                }
                Ok(())
            }
            ClientMessage::CLOSE => {
                w.u8(CLOSE)?;
                Ok(())
            }
        }
    }
}

impl ServerMessage {
    pub fn from_buf <R:ReadBytesSac> (r: &mut R) -> Result<(ServerMessage, Option<Vec<u8>>), Error> {
        //let mut r = Cursor::new(buf);
        let mtype = r.u8()?;
        let res = match mtype {
            SESS => Ok(ServerMessage::SESS(sSess::from_buf(r)?)),
            REL => Ok(ServerMessage::REL(Rel::from_buf(r)?)),
            ACK => Ok(ServerMessage::ACK(Ack::from_buf(r)?)),
            MAPDATA => Ok(ServerMessage::MAPDATA(MapData::from_buf(r)?)),
            OBJDATA => Ok(ServerMessage::OBJDATA(ObjData::from_buf(r)?)),
            CLOSE => Ok(ServerMessage::CLOSE),
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

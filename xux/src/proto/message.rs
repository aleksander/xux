use crate::proto::message_sess::*;
use crate::proto::message_rel::*;
use crate::proto::message_ack::*;
use crate::proto::message_beat::*;
use crate::proto::message_mapreq::*;
use crate::proto::message_mapdata::*;
use crate::proto::message_objdata::*;
use crate::proto::message_objack::*;
use crate::proto::message_close::*;
use crate::proto::serialization::*;
use crate::Result;
use anyhow::anyhow;

#[derive(Debug)]
pub enum ClientMessage {
    SESS( cSess ),
    REL( Rels ),
    ACK( Ack ),
    BEAT( Beat ),
    MAPREQ( MapReq ),
    OBJACK( ObjAck ),
    CLOSE( Close ),
}

#[derive(Debug)]
pub enum ServerMessage {
    SESS( sSess ),
    REL( Rels ),
    ACK( Ack ),
    MAPDATA( MapData ),
    OBJDATA( ObjData ),
    CLOSE( Close ),
}

impl ClientMessage {
    // TODO impl FromBuf for ClientMessage
    pub fn from_buf <R:ReadBytesSac> (r: &mut R) -> Result<(ClientMessage, Option<Vec<u8>>)> {
        let msg = match r.u8()? {
            cSess::ID => ClientMessage::SESS(cSess::from_buf(r)?),
            Rels::ID => ClientMessage::REL(Rels::from_buf(r)?),
            Ack::ID => ClientMessage::ACK(Ack::from_buf(r)?),
            Beat::ID => ClientMessage::BEAT(Beat),
            MapReq::ID => ClientMessage::MAPREQ(MapReq::from_buf(r)?),
            ObjAck::ID => ClientMessage::OBJACK(ObjAck::from_buf(r)?),
            Close::ID => ClientMessage::CLOSE(Close),
            id => { return Err(anyhow!("cmsg.from wrong message type: {}", id)); }
        };

        let mut tmp = Vec::new();
        r.read_to_end(&mut tmp)?;
        let remains = if tmp.is_empty() {
            None
        } else {
            Some(tmp)
        };

        Ok((msg, remains))
    }

    pub fn to_buf <W:WriteBytesSac> (&self, w: &mut W) -> Result<()> {
        match *self {
            ClientMessage::SESS(ref sess) => sess.to_buf(w),
            ClientMessage::ACK(ref ack) => ack.to_buf(w),
            ClientMessage::BEAT(_) => {
                w.u8(Beat::ID)?;
                Ok(())
            }
            ClientMessage::REL(ref rel) => {
                w.u8(Rels::ID)?;
                w.u16(rel.seq)?;
                for i in 0 .. rel.rels.len() {
                    let rel_elem = &rel.rels[i];
                    let last_one = i == (rel.rels.len() - 1);
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
    pub fn from_buf <R:ReadBytesSac> (r: &mut R) -> Result<(ServerMessage, Option<Vec<u8>>)> {
        let mtype = r.u8()?;
        let msg = match mtype {
            sSess::ID => ServerMessage::SESS(sSess::from_buf(r)?),
            Rels::ID => ServerMessage::REL(Rels::from_buf(r)?),
            Ack::ID => ServerMessage::ACK(Ack::from_buf(r)?),
            MapData::ID => ServerMessage::MAPDATA(MapData::from_buf(r)?),
            ObjData::ID => ServerMessage::OBJDATA(ObjData::from_buf(r)?),
            Close::ID => ServerMessage::CLOSE(Close),
            id => { return Err(anyhow!("smsg.from wrong message type: {}", id)); }
        };

        let mut tmp = Vec::new();
        r.read_to_end(&mut tmp)?;
        let remains = if tmp.is_empty() {
            None
        } else {
            Some(tmp)
        };

        Ok((msg, remains))
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

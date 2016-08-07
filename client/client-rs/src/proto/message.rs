use proto::message_sess::*;
use proto::message_rel::*;
use proto::message_ack::*;
use proto::message_mapreq::*;
use proto::message_mapdata::*;
use proto::message_objdata::*;
use proto::message_objack::*;
use Error;
use std::io::Cursor;
use proto::serialization::*;
use std::io::Read;
use std::io::Write;

#[derive(Clone,Copy)]
pub enum MessageDirection {
    FromClient,
    FromServer,
}

#[allow(non_camel_case_types)]
#[derive(Debug)]
// TODO replace with plain struct variants
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

// TODO maybe:
// pub enum ClientMessage {
//    SESS( cSess ),
//    REL( Rel ),
//    ACK( Ack ),
//    BEAT,
//    MAPREQ( MapReq ),
//    OBJACK( ObjAck ),
//    CLOSE/*( Close )*/,
// }
// pub enum ServerMessage {
//    SESS( sSess ),
//    REL( Rel ),
//    ACK( Ack ),
//    MAPDATA( MapData ),
//    OBJDATA( ObjData ),
//    CLOSE/*( Close )*/,
// }
// TODO impl FromBuf for ServerMessage
// TODO impl ToBuf for ClientMessage

const SESS: u8 = 0;
const REL: u8 = 1;
const ACK: u8 = 2;
const BEAT: u8 = 3;
const MAPREQ: u8 = 4;
const MAPDATA: u8 = 5;
const OBJDATA: u8 = 6;
const OBJACK: u8 = 7;
const CLOSE: u8 = 8;

impl Message {
    pub fn from_buf(buf: &[u8], dir: MessageDirection) -> Result<(Message, Option<Vec<u8>>), Error> {
        let mut r = Cursor::new(buf);
        let mtype = r.u8()?;
        let res = match mtype {
            SESS => {
                match dir {
                    MessageDirection::FromClient => Ok(Message::C_SESS(cSess::from_buf(&mut r)?)),
                    MessageDirection::FromServer => Ok(Message::S_SESS(sSess{ err: SessError::new(r.u8()?) })),
                }
            }
            REL => Ok(Message::REL(Rel::from_buf(&mut r)?)),
            ACK => Ok(Message::ACK(Ack { seq: r.u16()? })),
            BEAT => Ok(Message::BEAT),
            MAPREQ => {
                Ok(Message::MAPREQ(MapReq {
                    x: r.i32()?,
                    y: r.i32()?,
                }))
            }
            MAPDATA => {
                Ok(Message::MAPDATA(MapData {
                    pktid: r.i32()?,
                    off: r.u16()?,
                    len: r.u16()?,
                    buf: {
                        let mut buf = Vec::new();
                        r.read_to_end(&mut buf)?;
                        buf
                    },
                }))
            }
            OBJDATA => {
                let mut obj = Vec::new();
                loop {
                    let fl = match r.u8() {
                        Ok(b) => b,
                        Err(_) => {
                            break;
                        }
                    };
                    let id = r.u32()?;
                    let frame = r.i32()?;
                    let mut prop = Vec::new();
                    while let Some(p) = ObjDataElemProp::from_buf(&mut r)? {
                        prop.push(p)
                    }
                    obj.push(ObjDataElem {
                        fl: fl,
                        id: id,
                        frame: frame,
                        prop: prop,
                    });
                }
                Ok(Message::OBJDATA(ObjData { obj: obj }))
            }
            OBJACK => {
                // TODO FIXME parse ObjAck instead of empty return
                Ok(Message::OBJACK(ObjAck { obj: Vec::new() }))
            }
            CLOSE => {
                Ok(Message::CLOSE /* (Close) */)
            }
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

    pub fn to_buf(&self) -> Result<Vec<u8>, Error> {
        match *self {
            // !!! this is client session message, not server !!!
            Message::C_SESS(ref sess) => /*(name: &str, cookie: &[u8]) -> Vec<u8>*/ {
                let mut w = vec![];
                w.u8(SESS)?;
                w.u16(2)?; // unknown
                w.write("Salem".as_bytes())?; // proto
                w.u8(0)?;
                w.u16(36)?; // version
                w.write(sess.login.as_bytes())?; // login
                w.u8(0)?;
                w.u16(32)?; // cookie length
                w.write(sess.cookie.as_slice())?; // cookie
                Ok(w)
            }
            Message::S_SESS(/*ref sess*/ _ ) => {
                Err( Error{ source:"sSess.to_buf is not implemented yet", detail:None } )
            }
            Message::ACK(ref ack) => /*ack (seq: u16) -> Vec<u8>*/ {
                let mut w = vec![];
                w.u8(ACK)?;
                w.u16(ack.seq)?;
                Ok(w)
            }
            Message::BEAT => /* beat () -> Vec<u8> */ {
                let mut w = vec![];
                w.u8(BEAT)?;
                Ok(w)
            }
            Message::REL(ref rel) => /* rel_wdgmsg_play (seq: u16, name: &str) -> Vec<u8> */ {
                let mut w = vec![];
                w.u8(REL)?;
                w.u16(rel.seq)?;// sequence
                for i in 0 .. rel.rel.len() {
                    let rel_elem = &rel.rel[i];
                    let last_one = i == (rel.rel.len() - 1);
                    let rel_elem_buf = rel_elem.to_buf(last_one)?;
                    w.write(&rel_elem_buf)?;
                }
                Ok(w)
            }
            Message::MAPREQ(ref mapreq) => /* mapreq (x:i32, y:i32) -> Vec<u8> */ {
                let mut w = vec![];
                w.u8(MAPREQ)?;
                w.i32(mapreq.x)?;
                w.i32(mapreq.y)?;
                Ok(w)
            }
            Message::OBJACK(ref objack) => {
                let mut w = vec![];
                w.u8(OBJACK)?;
                for o in &objack.obj {
                    w.u32(o.id)?;
                    w.i32(o.frame)?;
                }
                Ok(w)
            }
            Message::CLOSE => {
                let mut w = vec![];
                w.u8(CLOSE)?;
                Ok(w)
            }
            _ => {
                Err( Error{ source:"unknown message type", detail:None } )
            }
        }
    }
}

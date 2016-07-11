use std::vec::Vec;
use std::fmt::Debug;
use std::fmt::Formatter;
use std::io::Cursor;
use std::io::Read;
use std::io::Write;
use std::io::BufRead;

use ::byteorder::{LittleEndian, BigEndian, ReadBytesExt, WriteBytesExt};
#[allow(non_camel_case_types)]
type le = LittleEndian;
#[allow(non_camel_case_types)]
type be = BigEndian;

// TODO move to salem::error mod
#[derive(Debug)]
pub struct Error {
    pub source: &'static str,
    pub detail: Option<String>,
}

impl From<::std::io::Error> for Error {
    fn from(_: ::std::io::Error) -> Error {
        Error {
            source: "TODO: Io error",
            detail: None,
        }
    }
}

impl From<::std::string::FromUtf8Error> for Error {
    fn from(_: ::std::string::FromUtf8Error) -> Error {
        Error {
            source: "TODO: FromUtf8 error",
            detail: None,
        }
    }
}

#[derive(Debug)]
pub struct NewWdg {
    pub id: u16,
    pub name: String,
    pub parent: u16,
    pub pargs: Vec<MsgList>,
    pub cargs: Vec<MsgList>,
}
#[derive(Debug)]
pub struct WdgMsg {
    pub id: u16,
    pub name: String,
    pub args: Vec<MsgList>,
}
#[derive(Debug)]
pub struct DstWdg {
    pub id: u16,
}
#[derive(Debug)]
pub struct MapIv;
#[derive(Debug)]
pub enum Glob {
    Time {
        time: i32,
        season: u8,
        inc: u8,
    },
    Light {
        amb: (u8, u8, u8, u8), // TODO Color type
        dif: (u8, u8, u8, u8), // TODO Color type
        spc: (u8, u8, u8, u8), // TODO Color type
        ang: i32,
        ele: i32,
        inc: u8,
    },
    Sky(Option<(u16, Option<(u16, i32)>)>), // (resid1,resid2,blend)
}
#[derive(Debug)]
pub struct Paginae;
#[derive(Debug)]
pub struct ResId {
    pub id: u16,
    pub name: String,
    pub ver: u16,
}
#[derive(Debug)]
pub struct Party;
#[derive(Debug)]
pub struct Sfx;
#[derive(Debug)]
pub struct Cattr;
#[derive(Debug)]
pub struct Music;
pub struct Tiles {
    pub tiles: Vec<TilesElem>,
}
impl Debug for Tiles {
    fn fmt(&self, f: &mut Formatter) -> ::std::fmt::Result {
        writeln!(f, "")?;
        for tile in &self.tiles {
            writeln!(f, "      {:?}", tile)?;
        }
        Ok(())
    }
}
#[derive(Debug)]
pub struct TilesElem {
    pub id: u8,
    pub name: String,
    pub ver: u16,
}
#[derive(Debug)]
pub struct Buff;
#[derive(Debug)]
pub struct SessKey;

#[derive(Debug)]
// TODO replace with plain struct variants
pub enum RelElem {
    NEWWDG(NewWdg),
    WDGMSG(WdgMsg),
    DSTWDG(DstWdg),
    MAPIV(MapIv),
    GLOBLOB(Vec<Glob>),
    PAGINAE(Paginae),
    RESID(ResId),
    PARTY(Party),
    SFX(Sfx),
    CATTR(Cattr),
    MUSIC(Music),
    TILES(Tiles),
    BUFF(Buff),
    SESSKEY(SessKey),
}

#[allow(non_camel_case_types)]
#[derive(Debug)]
// TODO replace with plain struct variants
pub enum MsgList {
    tINT(i32),
    tSTR(String),
    tCOORD((i32, i32)),
    tUINT8(u8),
    tUINT16(u16),
    tCOLOR((u8, u8, u8, u8)),
    tTTOL(Vec<MsgList>),
    tINT8(i8),
    tINT16(i16),
    tNIL, // (this is null)
    tBYTES(Vec<u8>),
    tFLOAT32(f32),
    tFLOAT64(f64),
}

const T_END: u8 = 0;
const T_TTOL: u8 = 8;
const T_INT: u8 = 1;
const T_STR: u8 = 2;
const T_COORD: u8 = 3;
const T_UINT8: u8 = 4;
const T_UINT16: u8 = 5;
const T_COLOR: u8 = 6;
const T_INT8: u8 = 9;
const T_INT16: u8 = 10;
const T_NIL: u8 = 12;
const T_BYTES: u8 = 14;
const T_FLOAT32: u8 = 15;
const T_FLOAT64: u8 = 16;

pub fn write_list(list: &[MsgList]) -> Result<Vec<u8>, Error> {
    let mut w = vec![];
    for l in list {
        let tmp = l;
        match *tmp {
            MsgList::tINT(i) => {
                w.write_u8(T_INT)?;
                w.write_i32::<le>(i)?;
            }
            MsgList::tSTR(ref s) => {
                w.write_u8(T_STR)?;
                w.write(s.as_bytes())?;
                w.write_u8(0)?; //'\0'
            }
            MsgList::tCOORD((x, y)) => {
                w.write_u8(T_COORD)?;
                w.write_i32::<le>(x)?;
                w.write_i32::<le>(y)?;
            }
            MsgList::tUINT8(u) => {
                w.write_u8(T_UINT8)?;
                w.write_u8(u)?;
            }
            MsgList::tUINT16(u) => {
                w.write_u8(T_UINT16)?;
                w.write_u16::<le>(u)?;
            }
            MsgList::tCOLOR((r, g, b, a)) => {
                w.write_u8(T_COLOR)?;
                w.write_u8(r)?;
                w.write_u8(g)?;
                w.write_u8(b)?;
                w.write_u8(a)?;
            }
            MsgList::tTTOL(_) => {
                return Err(Error {
                    source: "write_list is NOT implemented for tTTOL",
                    detail: None,
                });
            }
            MsgList::tINT8(i) => {
                w.write_u8(T_INT8)?;
                w.write_i8(i)?;
            }
            MsgList::tINT16(i) => {
                w.write_u8(T_INT16)?;
                w.write_i16::<le>(i)?;
            }
            MsgList::tNIL => {
                w.write_u8(T_NIL)?;
            }
            MsgList::tBYTES(_) => {
                return Err(Error {
                    source: "write_list is NOT implemented for tBYTES",
                    detail: None,
                });
            }
            MsgList::tFLOAT32(f) => {
                w.write_u8(T_FLOAT32)?;
                w.write_f32::<le>(f)?;
            }
            MsgList::tFLOAT64(f) => {
                w.write_u8(T_FLOAT64)?;
                w.write_f64::<le>(f)?;
            }
        }
    }
    w.write_u8(T_END)?;
    Ok(w)
}

pub fn read_list(r: &mut Cursor<&[u8]>) -> Vec<MsgList> /*TODO return Result instead*/ {
    let mut deep = 0;
    let mut list: Vec<Vec<MsgList>> = Vec::new();
    list.push(Vec::new());
    loop {
        let t = match r.read_u8() {
            Ok(b) => b,
            Err(_) => {
                while deep > 0 {
                    let tmp = list.remove(deep);
                    deep -= 1;
                    list[deep].push(MsgList::tTTOL(tmp));
                }
                return list.remove(0);
            }
        };
        match t {
            T_END => {
                if deep > 0 {
                    let tmp = list.remove(deep);
                    deep -= 1;
                    list[deep].push(MsgList::tTTOL(tmp));
                } else {
                    return list.remove(0);
                }
            }
            T_TTOL => {
                list.push(Vec::new());
                deep += 1;
            }
            T_INT => {
                list[deep].push(MsgList::tINT(r.read_i32::<le>().unwrap()));
            }
            T_STR => {
                let tmp = r.read_strz().unwrap();
                list[deep].push(MsgList::tSTR(tmp));
            }
            T_COORD => {
                list[deep].push(MsgList::tCOORD((r.read_i32::<le>().unwrap(), r.read_i32::<le>().unwrap())));
            }
            T_UINT8 => {
                list[deep].push(MsgList::tUINT8(r.read_u8().unwrap()));
            }
            T_UINT16 => {
                list[deep].push(MsgList::tUINT16(r.read_u16::<le>().unwrap()));
            }
            T_COLOR => {
                list[deep].push(MsgList::tCOLOR((r.read_u8().unwrap(), r.read_u8().unwrap(), r.read_u8().unwrap(), r.read_u8().unwrap())));
            }
            T_INT8 => {
                list[deep].push(MsgList::tINT8(r.read_i8().unwrap()));
            }
            T_INT16 => {
                list[deep].push(MsgList::tINT16(r.read_i16::<le>().unwrap()));
            }
            T_NIL => {
                list[deep].push(MsgList::tNIL);
            }
            T_BYTES => {
                let len = r.read_u8().unwrap();
                if (len & 128) != 0 {
                    let len = r.read_i32::<le>().unwrap();
                    assert!(len > 0);
                    let mut bytes = vec![0; len as usize];
                    r.read_exact(&mut bytes).unwrap();
                    list[deep].push(MsgList::tBYTES(bytes));
                } else {
                    let mut bytes = vec![0; len as usize];
                    r.read_exact(&mut bytes).unwrap();
                    list[deep].push(MsgList::tBYTES(bytes));
                }
            }
            T_FLOAT32 => {
                list[deep].push(MsgList::tFLOAT32(r.read_f32::<le>().unwrap()));
            }
            T_FLOAT64 => {
                list[deep].push(MsgList::tFLOAT64(r.read_f64::<le>().unwrap()));
            }
            _ => {
                info!("    !!! UNKNOWN LIST ELEMENT !!!");
                return list.remove(0); /*TODO return Error instead*/
            }
        }
    }
}

use std::io;

pub trait ReadExtExt: BufRead {
    #[inline]
    fn read_strz(&mut self) -> io::Result<String> {
        let mut tmp = Vec::new();
        self.read_until(0, &mut tmp)?;
        tmp.pop();
        Ok(String::from_utf8(tmp).unwrap())
    }
}

impl<R: BufRead + ?Sized> ReadExtExt for R {}

const NEWWDG: u8 = 0;
const WDGMSG: u8 = 1;
const DSTWDG: u8 = 2;
const MAPIV: u8 = 3;
const GLOBLOB: u8 = 4;
const PAGINAE: u8 = 5;
const RESID: u8 = 6;
const PARTY: u8 = 7;
const SFX: u8 = 8;
const CATTR: u8 = 9;
const MUSIC: u8 = 10;
const TILES: u8 = 11;
const BUFF: u8 = 12;
const SESSKEY: u8 = 13;

const GMSG_TIME: u8 = 0;
//const GMSG_ASTRO: u8 = 1; //TODO
const GMSG_LIGHT: u8 = 2;
const GMSG_SKY: u8 = 3;

const MORE_RELS_ATTACHED_BIT: u8 = 0x80;

impl RelElem {
    pub fn from_buf(kind: u8, buf: &[u8]) -> Result<RelElem, Error> {
        let mut r = Cursor::new(buf);
        // XXX RemoteUI.java +53
        match kind {
            NEWWDG => {
                let id = r.read_u16::<le>()?;
                let name = r.read_strz().unwrap();
                let parent = r.read_u16::<le>()?;
                let pargs = read_list(&mut r);
                let cargs = read_list(&mut r);
                Ok(RelElem::NEWWDG(NewWdg {
                    id: id,
                    name: name,
                    parent: parent,
                    pargs: pargs,
                    cargs: cargs,
                }))
            }
            WDGMSG => {
                let id = r.read_u16::<le>()?;
                let name = r.read_strz().unwrap();
                let args = read_list(&mut r);
                Ok(RelElem::WDGMSG(WdgMsg {
                    id: id,
                    name: name,
                    args: args,
                }))
            }
            DSTWDG => {
                let id = r.read_u16::<le>()?;
                Ok(RelElem::DSTWDG(DstWdg { id: id }))
            }
            MAPIV => Ok(RelElem::MAPIV(MapIv)),
            GLOBLOB => {
                let mut globs = Vec::new();
                let inc = r.read_u8().unwrap();
                loop {
                    let t = match r.read_u8() {
                        Ok(b) => b,
                        Err(_) => break, //TODO check error type
                    };
                    globs.push(match t {
                        GMSG_TIME => {
                            Glob::Time {
                                time: r.read_i32::<le>().unwrap(),
                                season: r.read_u8().unwrap(),
                                inc: inc,
                            }
                        }
                        // GMSG_ASTRO =>
                        GMSG_LIGHT => {
                            Glob::Light {
                                amb: (r.read_u8().unwrap(), r.read_u8().unwrap(), r.read_u8().unwrap(), r.read_u8().unwrap()),
                                dif: (r.read_u8().unwrap(), r.read_u8().unwrap(), r.read_u8().unwrap(), r.read_u8().unwrap()),
                                spc: (r.read_u8().unwrap(), r.read_u8().unwrap(), r.read_u8().unwrap(), r.read_u8().unwrap()),
                                ang: r.read_i32::<le>().unwrap(),
                                ele: r.read_i32::<le>().unwrap(),
                                inc: inc,
                            }
                        }
                        GMSG_SKY => {
                            use std::u16;
                            let id1 = r.read_u16::<le>().unwrap();
                            Glob::Sky(if id1 == u16::MAX {
                                None
                            } else {
                                let id2 = r.read_u16::<le>().unwrap();
                                if id2 == u16::MAX {
                                    Some((id1, None))
                                } else {
                                    Some((id1, Some((id2, r.read_i32::<le>().unwrap()))))
                                }
                            })
                        }
                        _ => {
                            return Err(Error {
                                source: "unknown GLOBLOB type",
                                detail: None,
                            })
                        }
                    });
                }
                Ok(RelElem::GLOBLOB(globs))
            }
            PAGINAE => Ok(RelElem::PAGINAE(Paginae)),
            RESID => {
                let id = r.read_u16::<le>()?;
                let name = r.read_strz().unwrap();
                let ver = r.read_u16::<le>()?;
                Ok(RelElem::RESID(ResId {
                    id: id,
                    name: name,
                    ver: ver,
                }))
            }
            PARTY => Ok(RelElem::PARTY(Party)),
            SFX => Ok(RelElem::SFX(Sfx)),
            CATTR => Ok(RelElem::CATTR(Cattr)),
            MUSIC => Ok(RelElem::MUSIC(Music)),
            TILES => {
                let mut tiles = Vec::new();
                loop {
                    let id = match r.read_u8() {
                        Ok(b) => b,
                        Err(_) => break, //TODO check error type
                    };
                    let name = r.read_strz().unwrap();
                    let ver = r.read_u16::<le>()?;
                    tiles.push(TilesElem {
                        id: id,
                        name: name,
                        ver: ver,
                    });
                }
                Ok(RelElem::TILES(Tiles { tiles: tiles }))
            }
            BUFF => Ok(RelElem::BUFF(Buff)),
            SESSKEY => Ok(RelElem::SESSKEY(SessKey)),
            _ => {
                Err(Error {
                    source: "unknown REL type",
                    detail: None,
                })
            }
        }
    }

    pub fn to_buf(&self, last: bool) -> Result<Vec<u8>, Error> {
        let mut w = vec![];
        match *self {
            RelElem::WDGMSG(ref msg) => {
                let mut tmp = vec![];
                tmp.write_u16::<le>(msg.id)?; // widget ID
                tmp.write(msg.name.as_bytes())?; // message name
                tmp.write_u8(0)?; // '\0'
                let args_buf = write_list(&msg.args)?;
                tmp.write(&args_buf)?;
                if last {
                    w.write_u8(WDGMSG)?;
                } else {
                    w.write_u8(WDGMSG & MORE_RELS_ATTACHED_BIT)?;
                    w.write_u16::<le>(tmp.len() as u16)?; // rel length
                }
                w.write(&tmp)?;

                Ok(w)
            }
            _ => {
                Err(Error {
                    source: "RelElem.to_buf is not implemented for that elem type",
                    detail: None,
                })
            }
        }
    }
}

#[derive(Debug)]
pub enum SessError {
    OK,
    AUTH,
    BUSY,
    CONN,
    PVER,
    EXPR,
    UNKNOWN(u8),
}

impl SessError {
    pub fn new(t: u8) -> SessError {
        match t {
            0 => SessError::OK,
            1 => SessError::AUTH,
            2 => SessError::BUSY,
            3 => SessError::CONN,
            4 => SessError::PVER,
            5 => SessError::EXPR,
            _ => SessError::UNKNOWN(t),
        }
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug)]
pub struct sSess {
    pub err: SessError,
}

#[allow(non_camel_case_types)]
#[derive(Debug)]
pub struct cSess {
    pub login: String,
    pub cookie: Vec<u8>,
}

pub struct Rel {
    pub seq: u16,
    pub rel: Vec<RelElem>,
}

#[allow(dead_code)]
impl Rel {
    fn new(seq: u16) -> Rel {
        Rel {
            seq: seq,
            rel: Vec::new(),
        }
    }
    fn append(&mut self, elem: RelElem) {
        self.rel.push(elem);
    }
}

impl Debug for Rel {
    fn fmt(&self, f: &mut Formatter) -> ::std::fmt::Result {
        writeln!(f, "REL seq={}", self.seq)?;
        for r in &self.rel {
            writeln!(f, "      {:?}", r)?;
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct Ack {
    pub seq: u16,
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct Beat;

#[derive(Debug)]
pub struct MapReq {
    pub x: i32,
    pub y: i32,
}

pub struct MapData {
    pub pktid: i32,
    pub off: u16,
    pub len: u16,
    pub buf: Vec<u8>,
}

impl Debug for MapData {
    fn fmt(&self, f: &mut Formatter) -> ::std::fmt::Result {
        write!(f, "MAPDATA pktid:{} offset:{} len:{} buf:[..{}]", self.pktid, self.off, self.len, self.buf.len())
    }
}

pub struct ObjData {
    pub obj: Vec<ObjDataElem>,
}

impl Debug for ObjData {
    fn fmt(&self, f: &mut Formatter) -> ::std::fmt::Result {
        writeln!(f, "OBJDATA")?;
        for o in &self.obj {
            writeln!(f, "      {:?}", o)?;
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct ObjDataElem {
    pub fl: u8,
    pub id: u32,
    pub frame: i32,
    pub prop: Vec<ObjDataElemProp>,
}

#[derive(Debug)]
pub struct ObjAck {
    pub obj: Vec<ObjAckElem>,
}

impl ObjAck {
    pub fn new(objdata: &ObjData) -> ObjAck {
        let mut objack = ObjAck { obj: Vec::new() };
        for o in &objdata.obj {
            objack.obj.push(ObjAckElem {
                id: o.id,
                frame: o.frame,
            });
        }
        objack
    }
}

#[derive(Debug)]
pub struct ObjAckElem {
    pub id: u32,
    pub frame: i32,
}

// #[derive(Debug)]
// pub struct Close;

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
    CLOSE, // ( Close )
}

// TODO maybe:
// pub enum ClientMessage {
//    C_SESS( cSess ),
//    REL( Rel ),
//    ACK( Ack ),
//    BEAT,
//    MAPREQ( MapReq ),
//    OBJACK( ObjAck ),
//    CLOSE/*( Close )*/,
// }
// pub enum ServerMessage {
//    S_SESS( sSess ),
//    REL( Rel ),
//    ACK( Ack ),
//    MAPDATA( MapData ),
//    OBJDATA( ObjData ),
//    CLOSE/*( Close )*/,
// }

#[allow(non_camel_case_types)]
#[derive(Debug)]
// TODO replace with plain struct variants
pub enum ObjDataElemProp {
    odREM,
    odMOVE((i32, i32), u16),
    odRES(u16),
    odLINBEG((i32, i32), (i32, i32), i32),
    odLINSTEP(i32),
    odSPEECH(u16, String),
    odCOMPOSE(u16),
    odDRAWOFF((i32, i32)),
    odLUMIN((i32, i32), u16, u8),
    odAVATAR(Vec<u16>),
    odFOLLOW(odFOLLOW),
    odHOMING(odHOMING),
    odOVERLAY(u16),
    odAUTH,
    odHEALTH(u8),
    odBUDDY(odBUDDY),
    odCMPPOSE(u8, Option<Vec<u16>>, Option<(Option<Vec<u16>>, u8)>),
    odCMPMOD(Option<Vec<Vec<u16>>>),
    odCMPEQU(Option<Vec<(u8, String, u16, Option<(u16, u16, u16)>)>>),
    odICON(odICON),
}

#[allow(non_camel_case_types)]
#[derive(Debug)]
pub enum odFOLLOW {
    Stop,
    To(u32, u16, String),
}

#[allow(non_camel_case_types)]
#[derive(Debug)]
pub enum odHOMING {
    New((i32, i32), u16),
    Change((i32, i32), u16),
    Delete,
}

#[allow(non_camel_case_types)]
#[derive(Debug)]
pub enum odBUDDY {
    Update(String, u8, u8),
    Delete,
}

#[allow(non_camel_case_types)]
#[derive(Debug)]
pub enum odICON {
    Set(u16),
    Del,
}

const OD_REM: u8 = 0;
const OD_MOVE: u8 = 1;
const OD_RES: u8 = 2;
const OD_LINBEG: u8 = 3;
const OD_LINSTEP: u8 = 4;
const OD_SPEECH: u8 = 5;
const OD_COMPOSE: u8 = 6;
const OD_DRAWOFF: u8 = 7;
const OD_LUMIN: u8 = 8;
const OD_AVATAR: u8 = 9;
const OD_FOLLOW: u8 = 10;
const OD_HOMING: u8 = 11;
const OD_OVERLAY: u8 = 12;
const OD_AUTH: u8 = 13;
const OD_HEALTH: u8 = 14;
const OD_BUDDY: u8 = 15;
const OD_CMPPOSE: u8 = 16;
const OD_CMPMOD: u8 = 17;
const OD_CMPEQU: u8 = 18;
const OD_ICON: u8 = 19;
const OD_END: u8 = 255;

impl ObjDataElemProp {
    pub fn from_buf(r: &mut Cursor<&[u8]>) -> Result<Option<ObjDataElemProp>, Error> {
        let t = r.read_u8()?;
        match t {
            OD_REM => Ok(Some(ObjDataElemProp::odREM)),
            OD_MOVE => {
                let xy = (r.read_i32::<le>()?, r.read_i32::<le>()?);
                let ia = r.read_u16::<le>()?;
                Ok(Some(ObjDataElemProp::odMOVE(xy, ia)))
            }
            OD_RES => {
                let mut resid = r.read_u16::<le>()?;
                if (resid & 0x8000) != 0 {
                    resid &= !0x8000;
                    let sdt_len = r.read_u8().unwrap();
                    let /*sdt*/ _ = {
                        let mut tmp = vec![0; sdt_len as usize];
                        r.read_exact(&mut tmp).unwrap();
                        tmp
                    };
                }
                Ok(Some(ObjDataElemProp::odRES(resid)))
            }
            OD_LINBEG => {
                let s = (r.read_i32::<le>()?, r.read_i32::<le>()?);
                let t = (r.read_i32::<le>()?, r.read_i32::<le>()?);
                let c = r.read_i32::<le>()?;
                Ok(Some(ObjDataElemProp::odLINBEG(s, t, c)))
            }
            OD_LINSTEP => {
                let l = r.read_i32::<le>()?;
                Ok(Some(ObjDataElemProp::odLINSTEP(l)))
            }
            OD_SPEECH => {
                let zo = r.read_u16::<le>()?;
                let text = r.read_strz().unwrap();
                Ok(Some(ObjDataElemProp::odSPEECH(zo, text)))
            }
            OD_COMPOSE => {
                let resid = r.read_u16::<le>()?;
                Ok(Some(ObjDataElemProp::odCOMPOSE(resid)))
            }
            OD_DRAWOFF => {
                let off = (r.read_i32::<le>()?, r.read_i32::<le>()?);
                Ok(Some(ObjDataElemProp::odDRAWOFF(off)))
            }
            OD_LUMIN => {
                let off = (r.read_i32::<le>()?, r.read_i32::<le>()?);
                let sz = r.read_u16::<le>()?;
                let str_ = r.read_u8()?;
                Ok(Some(ObjDataElemProp::odLUMIN(off, sz, str_)))
            }
            OD_AVATAR => {
                let mut layers = Vec::new();
                loop {
                    let layer = r.read_u16::<le>()?;
                    if layer == 65535 {
                        break;
                    }
                    layers.push(layer);
                }
                Ok(Some(ObjDataElemProp::odAVATAR(layers)))
            }
            OD_FOLLOW => {
                let oid = r.read_u32::<le>()?;
                if oid == 0xff_ff_ff_ff {
                    Ok(Some(ObjDataElemProp::odFOLLOW(odFOLLOW::Stop)))
                } else {
                    let xfres = r.read_u16::<le>()?;
                    let xfname = r.read_strz().unwrap();
                    Ok(Some(ObjDataElemProp::odFOLLOW(odFOLLOW::To(oid, xfres, xfname))))
                }
            }
            OD_HOMING => {
                let oid = r.read_u32::<le>()?;
                match oid {
                    0xff_ff_ff_ff => Ok(Some(ObjDataElemProp::odHOMING(odHOMING::Delete))),
                    0xff_ff_ff_fe => {
                        let tgtc = (r.read_i32::<le>()?, r.read_i32::<le>()?);
                        let v = r.read_u16::<le>()?;
                        Ok(Some(ObjDataElemProp::odHOMING(odHOMING::Change(tgtc, v))))
                    }
                    _ => {
                        let tgtc = (r.read_i32::<le>()?, r.read_i32::<le>()?);
                        let v = r.read_u16::<le>()?;
                        Ok(Some(ObjDataElemProp::odHOMING(odHOMING::New(tgtc, v))))
                    }
                }
            }
            OD_OVERLAY => {
                let /*olid*/ _ = r.read_i32::<le>()?;
                let resid = r.read_u16::<le>()?;
                if (resid != 0xffff) && ((resid & 0x8000) != 0) {
                    let sdt_len = r.read_u8()? as usize;
                    let /*sdt*/ _ = {
                        let mut tmp = vec![0; sdt_len];
                        r.read_exact(&mut tmp).unwrap();
                        tmp
                    };
                }
                Ok(Some(ObjDataElemProp::odOVERLAY(resid & (!0x8000))))
            }
            OD_AUTH => {
                Ok(Some(ObjDataElemProp::odAUTH)) // Removed
            }
            OD_HEALTH => {
                let hp = r.read_u8()?;
                Ok(Some(ObjDataElemProp::odHEALTH(hp)))
            }
            OD_BUDDY => {
                let name = r.read_strz().unwrap();
                // XXX FIXME C string is not like Rust string, it has \0 at the end,
                //          so this check is incorrect, I SUPPOSE.
                //          MOST PROBABLY we will crash here because 2 more readings.
                if name.is_empty() {
                    Ok(Some(ObjDataElemProp::odBUDDY(odBUDDY::Delete)))
                } else {
                    let group = r.read_u8()?;
                    let btype = r.read_u8()?;
                    Ok(Some(ObjDataElemProp::odBUDDY(odBUDDY::Update(name, group, btype))))
                }
            }
            OD_CMPPOSE => {
                let pfl = r.read_u8()?;
                let seq = r.read_u8()?;
                let ids1 = if (pfl & 2) != 0 {
                    let mut ids = Vec::new();
                    loop {
                        let mut resid = r.read_u16::<le>()?;
                        if resid == 65535 {
                            break;
                        }
                        if (resid & 0x8000) != 0 {
                            resid &= !0x8000;
                            let sdt_len = r.read_u8()? as usize;
                            let /*sdt*/ _ = {
                                let mut tmp = vec![0; sdt_len];
                                r.read_exact(&mut tmp).unwrap();
                                tmp
                            };
                        }
                        ids.push(resid);
                    }
                    if !ids.is_empty() {
                        Some(ids)
                    } else {
                        None
                    }
                } else {
                    None
                };
                let ids2 = if (pfl & 4) != 0 {
                    let mut ids = Vec::new();
                    loop {
                        let mut resid = r.read_u16::<le>()?;
                        if resid == 65535 {
                            break;
                        }
                        if (resid & 0x8000) != 0 {
                            resid &= !0x8000;
                            let sdt_len = r.read_u8()? as usize;
                            let /*sdt*/ _ = {
                                let mut tmp = vec![0; sdt_len];
                                r.read_exact(&mut tmp).unwrap();
                                tmp
                            };
                        }
                        ids.push(resid);
                    }
                    let ttime = r.read_u8()?;
                    if !ids.is_empty() {
                        Some((Some(ids), ttime))
                    } else {
                        Some((None, ttime))
                    }
                } else {
                    None
                };
                Ok(Some(ObjDataElemProp::odCMPPOSE(seq, ids1, ids2)))
            }
            OD_CMPMOD => {
                let mut m = Vec::new();
                loop {
                    let modif = r.read_u16::<le>()?;
                    if modif == 65535 {
                        break;
                    }
                    let mut ids = Vec::new();
                    loop {
                        let resid = r.read_u16::<le>()?;
                        if resid == 65535 {
                            break;
                        }
                        ids.push(resid);
                    }
                    if !ids.is_empty() {
                        m.push(ids);
                    }
                }
                let mods = if !m.is_empty() {
                    Some(m)
                } else {
                    None
                };
                Ok(Some(ObjDataElemProp::odCMPMOD(mods)))
            }
            OD_CMPEQU => {
                let mut e = Vec::new();
                loop {
                    let h = r.read_u8()?;
                    if h == 255 {
                        break;
                    }
                    let at = r.read_strz().unwrap();
                    let resid = r.read_u16::<le>()?;
                    let off = if (h & 0x80) != 0 {
                        let x = r.read_u16::<le>()?;
                        let y = r.read_u16::<le>()?;
                        let z = r.read_u16::<le>()?;
                        Some((x, y, z))
                    } else {
                        None
                    };
                    e.push((h & 0x7f, at, resid, off));
                }
                let equ = if !e.is_empty() {
                    Some(e)
                } else {
                    None
                };
                Ok(Some(ObjDataElemProp::odCMPEQU(equ)))
            }
            OD_ICON => {
                let resid = r.read_u16::<le>()?;
                if resid == 65535 {
                    Ok(Some(ObjDataElemProp::odICON(odICON::Del)))
                } else {
                    let /*ifl*/ _ = r.read_u8()?;
                    Ok(Some(ObjDataElemProp::odICON(odICON::Set(resid))))
                }
            }
            OD_END => Ok(None),
            _ => {
                Ok(None) /*TODO return error*/
            }
        }
    }
}

#[derive(Clone,Copy)]
pub enum MessageDirection {
    FromClient,
    FromServer,
}

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
    // TODO ADD fuzzing tests:
    //        for i in range(0u8, 255) {
    //            let mut v = Vec::new();
    //            v.push(i);
    //            info!("{}", Message::from_buf(v.as_slice()));
    //        }
    // TODO
    // fn from_buf_checked (buf,dir) {
    //     if (this message can be received by this dir) {
    //         return Ok(buf.from_buf)
    //     else
    //         return Err("this king of message can't be received by this side")
    // }
    // TODO return Error with stack trace on Err instead of String
    // TODO get Vec not &[]. return Vec in the case of error
    pub fn from_buf(buf: &[u8], dir: MessageDirection) -> Result<(Message, Option<Vec<u8>>), Error> {
        let mut r = Cursor::new(buf);
        let mtype = r.read_u8()?;
        let res = match mtype {
            SESS => {
                // TODO ??? Ok(Message::sess(err))
                //     impl Message { fn sess (err: u8) -> Message::SESS { ... } }
                match dir {
                    MessageDirection::FromClient => {
                        let /*unknown*/ _ = r.read_u16::<le>()?;
                        let /*proto*/ _ = r.read_strz().unwrap();
                        let /*version*/ _ = r.read_u16::<le>()?;
                        let login = r.read_strz().unwrap();
                        let cookie_len = r.read_u16::<le>()?;
                        let cookie = {
                            let mut tmp = vec![0; cookie_len as usize];
                            r.read_exact(&mut tmp)?;
                            tmp
                        };
                        Ok(Message::C_SESS(cSess {
                            login: login,
                            cookie: cookie,
                        }))
                    }
                    MessageDirection::FromServer => Ok(Message::S_SESS(sSess { err: SessError::new(r.read_u8()?) })),
                }
            }
            REL => {
                let seq = r.read_u16::<le>()?;
                let mut rel_vec = Vec::new();
                loop {
                    let mut rel_type = match r.read_u8() {
                        Ok(b) => b,
                        Err(_) => {
                            break;
                        }
                    };
                    let rel_buf = if (rel_type & 0x80) != 0 {
                        rel_type &= !0x80;
                        let rel_len = r.read_u16::<le>()?;
                        let mut tmp = vec![0; rel_len as usize];
                        r.read_exact(&mut tmp).unwrap();
                        tmp
                    } else {
                        let mut tmp = Vec::new();
                        r.read_to_end(&mut tmp)?;
                        tmp
                    };
                    rel_vec.push(RelElem::from_buf(rel_type, rel_buf.as_slice())?);
                }
                Ok(Message::REL(Rel {
                    seq: seq,
                    rel: rel_vec,
                }))
            }
            ACK => Ok(Message::ACK(Ack { seq: r.read_u16::<le>()? })),
            BEAT => Ok(Message::BEAT),
            MAPREQ => {
                Ok(Message::MAPREQ(MapReq {
                    x: r.read_i32::<le>()?,
                    y: r.read_i32::<le>()?,
                }))
            }
            MAPDATA => {
                let pktid = r.read_i32::<le>()?;
                let off = r.read_u16::<le>()?;
                let len = r.read_u16::<le>()?;
                let mut buf = Vec::new();
                r.read_to_end(&mut buf)?;
                // info!("    pktid={} off={} len={}", pktid, off, len);
                // if (off == 0) {
                //    info!("      coord=({}, {})", r.read_i32::<le>().unwrap(), r.read_i32::<le>().unwrap());
                //    info!("      mmname=\"{}\"", r.read_until(0).unwrap());
                //    loop {
                //        let pidx = r.read_u8().unwrap();
                //        if pidx == 255 break;
                //    }
                // }
                Ok(Message::MAPDATA(MapData {
                    pktid: pktid,
                    off: off,
                    len: len,
                    buf: buf,
                }))
            }
            OBJDATA => {
                let mut obj = Vec::new();
                loop {
                    let fl = match r.read_u8() {
                        Ok(b) => b,
                        Err(_) => {
                            break;
                        }
                    };
                    let id = r.read_u32::<le>()?;
                    let frame = r.read_i32::<le>()?;
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
                w.write_u8(SESS)?;
                w.write_u16::<le>(2)?; // unknown
                w.write("Salem".as_bytes())?; // proto
                w.write_u8(0)?;
                w.write_u16::<le>(36)?; // version
                w.write(sess.login.as_bytes())?; // login
                w.write_u8(0)?;
                w.write_u16::<le>(32)?; // cookie length
                w.write(sess.cookie.as_slice())?; // cookie
                Ok(w)
            }
            Message::S_SESS(/*ref sess*/ _ ) => {
                Err( Error{ source:"sSess.to_buf is not implemented yet", detail:None } )
            }
            Message::ACK(ref ack) => /*ack (seq: u16) -> Vec<u8>*/ {
                let mut w = vec![];
                w.write_u8(ACK)?;
                w.write_u16::<le>(ack.seq)?;
                Ok(w)
            }
            Message::BEAT => /* beat () -> Vec<u8> */ {
                let mut w = vec![];
                w.write_u8(BEAT)?;
                Ok(w)
            }
            Message::REL(ref rel) => /* rel_wdgmsg_play (seq: u16, name: &str) -> Vec<u8> */ {
                let mut w = vec![];
                w.write_u8(REL)?;
                w.write_u16::<le>(rel.seq)?;// sequence
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
                w.write_u8(MAPREQ)?;
                w.write_i32::<le>(mapreq.x)?;
                w.write_i32::<le>(mapreq.y)?;
                Ok(w)
            }
            Message::OBJACK(ref objack) => {
                let mut w = vec![];
                w.write_u8(OBJACK)?;
                for o in &objack.obj {
                    w.write_u32::<le>(o.id)?;
                    w.write_i32::<le>(o.frame)?;
                }
                Ok(w)
            }
            Message::CLOSE => {
                let mut w = vec![];
                w.write_u8(CLOSE)?;
                Ok(w)
            }
            _ => {
                Err( Error{ source:"unknown message type", detail:None } )
            }
        }
    }
}

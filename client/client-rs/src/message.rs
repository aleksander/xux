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

pub fn write_list(list: &[MsgList]) -> Result<Vec<u8>, Error> {
    let mut w = vec![];
    for l in list {
        let tmp = l;
        match *tmp {
            MsgList::tINT(i) => {
                w.write_u8(1)?;
                w.write_i32::<le>(i)?;
            }
            MsgList::tSTR(ref s) => {
                w.write_u8(2)?;
                w.write(s.as_bytes())?;
                w.write_u8(0)?; //'\0'
            }
            MsgList::tCOORD((x, y)) => {
                w.write_u8(3)?;
                w.write_i32::<le>(x)?;
                w.write_i32::<le>(y)?;
            }
            MsgList::tUINT8(u) => {
                w.write_u8(4)?;
                w.write_u8(u)?;
            }
            MsgList::tUINT16(u) => {
                w.write_u8(5)?;
                w.write_u16::<le>(u)?;
            }
            MsgList::tCOLOR((r, g, b, a)) => {
                w.write_u8(6)?;
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
                w.write_u8(9)?;
                w.write_i8(i)?;
            }
            MsgList::tINT16(i) => {
                w.write_u8(10)?;
                w.write_i16::<le>(i)?;
            }
            MsgList::tNIL => {
                w.write_u8(12)?;
            }
            MsgList::tBYTES(_) => {
                return Err(Error {
                    source: "write_list is NOT implemented for tBYTES",
                    detail: None,
                });
            }
            MsgList::tFLOAT32(f) => {
                w.write_u8(15)?;
                w.write_f32::<le>(f)?;
            }
            MsgList::tFLOAT64(f) => {
                w.write_u8(16)?;
                w.write_f64::<le>(f)?;
            }
        }
    }
    w.write_u8(0)?; /* T_END */
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
            // T_END
            0 => {
                if deep > 0 {
                    let tmp = list.remove(deep);
                    deep -= 1;
                    list[deep].push(MsgList::tTTOL(tmp));
                } else {
                    return list.remove(0);
                }
            }
            // T_TTOL
            8 => {
                list.push(Vec::new());
                deep += 1;
            }
            // T_INT
            1 => {
                list[deep].push(MsgList::tINT(r.read_i32::<le>().unwrap()));
            }
            // T_STR
            2 => {
                let tmp = r.read_strz().unwrap();
                list[deep].push(MsgList::tSTR(tmp));
            }
            // T_COORD
            3 => {
                list[deep].push(MsgList::tCOORD((r.read_i32::<le>().unwrap(), r.read_i32::<le>().unwrap())));
            }
            // T_UINT8
            4 => {
                list[deep].push(MsgList::tUINT8(r.read_u8().unwrap()));
            }
            // T_UINT16
            5 => {
                list[deep].push(MsgList::tUINT16(r.read_u16::<le>().unwrap()));
            }
            // T_COLOR
            6 => {
                list[deep].push(MsgList::tCOLOR((r.read_u8().unwrap(),
                                                 r.read_u8().unwrap(),
                                                 r.read_u8().unwrap(),
                                                 r.read_u8().unwrap())));
            }
            // T_INT8
            9 => {
                list[deep].push(MsgList::tINT8(r.read_i8().unwrap()));
            }
            // T_INT16
            10 => {
                list[deep].push(MsgList::tINT16(r.read_i16::<le>().unwrap()));
            }
            // T_NIL
            12 => {
                list[deep].push(MsgList::tNIL);
            }
            // T_BYTES
            14 => {
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
            // T_FLOAT32
            15 => {
                list[deep].push(MsgList::tFLOAT32(r.read_f32::<le>().unwrap()));
            }
            // T_FLOAT64
            16 => {
                list[deep].push(MsgList::tFLOAT64(r.read_f64::<le>().unwrap()));
            }
            // UNKNOWN
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

impl RelElem {
    pub fn from_buf(kind: u8, buf: &[u8]) -> Result<RelElem, Error> {
        let mut r = Cursor::new(buf);
        // XXX RemoteUI.java +53
        match kind {
            0  /*NEWWDG*/  => {
                let id = try!(r.read_u16::<le>());
                let name = r.read_strz().unwrap();
                let parent = try!(r.read_u16::<le>());
                let pargs = read_list(&mut r);
                let cargs = read_list(&mut r);
                Ok( RelElem::NEWWDG( NewWdg{ id:id, name:name, parent:parent, pargs:pargs, cargs:cargs } ) )
            },
            1  /*WDGMSG*/  => {
                let id = try!(r.read_u16::<le>());
                let name = r.read_strz().unwrap();
                let args = read_list(&mut r);
                Ok( RelElem::WDGMSG( WdgMsg{ id:id, name:name, args:args } ) )
            },
            2  /*DSTWDG*/  => {
                let id = try!(r.read_u16::<le>());
                Ok( RelElem::DSTWDG( DstWdg{ id:id } ) )
            },
            3  /*MAPIV*/   => { Ok( RelElem::MAPIV(MapIv) ) },
            4  /*GLOBLOB*/ => {
                let mut globs = Vec::new();
                let inc = r.read_u8().unwrap();
                loop {
                    let t = match r.read_u8() {
                        Ok(b) => b,
                        Err(_) => break //TODO check error type
                    };
                    globs.push( match t {
                        0 /*GMSG_TIME*/ => {
                            Glob::Time {
                                time: r.read_i32::<le>().unwrap(),
                                season: r.read_u8().unwrap(),
                                inc: inc
                            }
                        }
                        /*1 /*GMSG_ASTRO*/ =>*/
                        2 /*GMSG_LIGHT*/ => {
                            Glob::Light { 
                                amb: (r.read_u8().unwrap(), r.read_u8().unwrap(), r.read_u8().unwrap(), r.read_u8().unwrap()),
                                dif: (r.read_u8().unwrap(), r.read_u8().unwrap(), r.read_u8().unwrap(), r.read_u8().unwrap()),
                                spc: (r.read_u8().unwrap(), r.read_u8().unwrap(), r.read_u8().unwrap(), r.read_u8().unwrap()),
                                ang: r.read_i32::<le>().unwrap(),
                                ele: r.read_i32::<le>().unwrap(),
                                inc: inc
                            }
                        }
                        3 /*GMSG_SKY*/ => {
                            use std::u16;
                            let id1 = r.read_u16::<le>().unwrap();
                            Glob::Sky(
                                if id1 == u16::MAX {
                                    None
                                } else {
                                    let id2 = r.read_u16::<le>().unwrap();
                                    if id2 == u16::MAX {
                                        Some((
                                            id1, None
                                        ))
                                    } else {
                                        Some((
                                            id1, Some((
                                                id2, r.read_i32::<le>().unwrap()
                                            ))
                                        ))
                                    }
                                }
                            )
                        }
                        _ => return Err( Error{ source:"unknown GLOBLOB type", detail:None })
                    });
                }
                Ok( RelElem::GLOBLOB( globs ))
            },
            5  /*PAGINAE*/ => { Ok( RelElem::PAGINAE(Paginae) ) },
            6  /*RESID*/   => {
                let id = try!(r.read_u16::<le>());
                let name = r.read_strz().unwrap();
                let ver = try!(r.read_u16::<le>());
                Ok( RelElem::RESID( ResId{ id:id, name:name, ver:ver } ) )
            },
            7  /*PARTY*/   => { Ok( RelElem::PARTY(Party) ) },
            8  /*SFX*/     => { Ok( RelElem::SFX(Sfx) ) },
            9  /*CATTR*/   => { Ok( RelElem::CATTR(Cattr) ) },
            10 /*MUSIC*/   => { Ok( RelElem::MUSIC(Music) ) },
            11 /*TILES*/   => {
                let mut tiles = Vec::new();
                loop {
                    let id = match r.read_u8() {
                        Ok(b) => b,
                        Err(_) => break //TODO check error type
                    };
                    let name = r.read_strz().unwrap();
                    let ver = try!(r.read_u16::<le>());
                    tiles.push(TilesElem{ id:id, name:name, ver:ver });
                }
                Ok( RelElem::TILES(Tiles{ tiles:tiles }) )
            },
            12 /*BUFF*/    => { Ok( RelElem::BUFF(Buff) ) },
            13 /*SESSKEY*/ => { Ok( RelElem::SESSKEY(SessKey) ) },
            _  /*UNKNOWN*/ => { Err( Error{ source:"unknown REL type", detail:None } ) },
        }
    }

    pub fn to_buf(&self, last: bool) -> Result<Vec<u8>, Error> {
        let mut w = vec![];
        match *self {
            RelElem::WDGMSG(ref msg) => {
                let mut tmp = vec![];
                tmp.write_u16::<le>(msg.id)?; // widget ID
                tmp.write(msg.name.as_bytes())?; // message name
                tmp.write_u8(0)?; // \0
                let args_buf = write_list(&msg.args)?;
                tmp.write(&args_buf)?;
                if last {
                    w.write_u8(1)?; // type WDGMSG
                } else {
                    w.write_u8(1 & 0x80)?; // type WDGMSG & more rels attached bit
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

impl ObjDataElemProp {
    pub fn from_buf(r: &mut Cursor<&[u8]>) -> Result<Option<ObjDataElemProp>, Error> {
        let t = r.read_u8()? as usize;
        match t {
            0   /*OD_REM*/ => {
                Ok(Some(ObjDataElemProp::odREM))
            },
            1   /*OD_MOVE*/ => {
                let xy = (try!(r.read_i32::<le>()), try!(r.read_i32::<le>()));
                let ia = try!(r.read_u16::<le>());
                Ok(Some(ObjDataElemProp::odMOVE(xy,ia)))
            },
            2   /*OD_RES*/ => {
                let mut resid = try!(r.read_u16::<le>());
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
            },
            3   /*OD_LINBEG*/ => {
                let s = (try!(r.read_i32::<le>()), try!(r.read_i32::<le>()));
                let t = (try!(r.read_i32::<le>()), try!(r.read_i32::<le>()));
                let c = try!(r.read_i32::<le>());
                Ok(Some(ObjDataElemProp::odLINBEG(s,t,c)))
            },
            4   /*OD_LINSTEP*/ => {
                let l = try!(r.read_i32::<le>());
                Ok(Some(ObjDataElemProp::odLINSTEP(l)))
            },
            5   /*OD_SPEECH*/ => {
                let zo = try!(r.read_u16::<le>());
                let text = r.read_strz().unwrap();
                Ok(Some(ObjDataElemProp::odSPEECH(zo,text)))
            },
            6   /*OD_COMPOSE*/ => {
                let resid = try!(r.read_u16::<le>());
                Ok(Some(ObjDataElemProp::odCOMPOSE(resid)))
            },
            7   /*OD_DRAWOFF*/ => {
                let off = (try!(r.read_i32::<le>()), try!(r.read_i32::<le>()));
                Ok(Some(ObjDataElemProp::odDRAWOFF(off)))
            },
            8   /*OD_LUMIN*/ => {
                let off = (try!(r.read_i32::<le>()), try!(r.read_i32::<le>()));
                let sz = try!(r.read_u16::<le>());
                let str_ = try!(r.read_u8());
                Ok(Some(ObjDataElemProp::odLUMIN(off,sz,str_)))
            },
            9   /*OD_AVATAR*/ => {
                let mut layers = Vec::new();
                loop {
                    let layer = try!(r.read_u16::<le>());
                    if layer == 65535 {
                        break;
                    }
                    layers.push(layer);
                }
                Ok(Some(ObjDataElemProp::odAVATAR(layers)))
            },
            10  /*OD_FOLLOW*/ => {
                let oid = try!(r.read_u32::<le>());
                if oid == 0xff_ff_ff_ff {
                    Ok(Some(ObjDataElemProp::odFOLLOW(odFOLLOW::Stop)))
                } else {
                    let xfres = try!(r.read_u16::<le>());
                    let xfname = r.read_strz().unwrap();
                    Ok(Some(ObjDataElemProp::odFOLLOW(odFOLLOW::To(oid,xfres,xfname))))
                }
            },
            11  /*OD_HOMING*/ => {
                let oid = try!(r.read_u32::<le>());
                match oid {
                    0xff_ff_ff_ff => {
                        Ok(Some(ObjDataElemProp::odHOMING(odHOMING::Delete)))
                    },
                    0xff_ff_ff_fe => {
                        let tgtc = (try!(r.read_i32::<le>()), try!(r.read_i32::<le>()));
                        let v = try!(r.read_u16::<le>());
                        Ok(Some(ObjDataElemProp::odHOMING(odHOMING::Change(tgtc,v))))
                    },
                    _             => {
                        let tgtc = (try!(r.read_i32::<le>()), try!(r.read_i32::<le>()));
                        let v = try!(r.read_u16::<le>());
                        Ok(Some(ObjDataElemProp::odHOMING(odHOMING::New(tgtc,v))))
                    }
                }
            },
            12  /*OD_OVERLAY*/ => {
                let /*olid*/ _ = try!(r.read_i32::<le>());
                let resid = try!(r.read_u16::<le>());
                if (resid != 0xffff) && ((resid & 0x8000) != 0) {
                    let sdt_len = try!(r.read_u8()) as usize;
                    let /*sdt*/ _ = {
                        let mut tmp = vec![0; sdt_len];
                        r.read_exact(&mut tmp).unwrap();
                        tmp
                    };
                }
                Ok(Some(ObjDataElemProp::odOVERLAY( resid&(!0x8000) )))
            },
            13  /*OD_AUTH*/   => {
                Ok(Some(ObjDataElemProp::odAUTH)) // Removed
            },
            14  /*OD_HEALTH*/ => {
                let hp = try!(r.read_u8());
                Ok(Some(ObjDataElemProp::odHEALTH(hp)))
            },
            15  /*OD_BUDDY*/ => {
                let name = r.read_strz().unwrap();
                //XXX FIXME C string is not like Rust string, it has \0 at the end,
                //          so this check is incorrect, I SUPPOSE.
                //          MOST PROBABLY we will crash here because 2 more readings.
                if name.is_empty() {
                    Ok(Some(ObjDataElemProp::odBUDDY(odBUDDY::Delete)))
                } else {
                    let group = try!(r.read_u8());
                    let btype = try!(r.read_u8());
                    Ok(Some(ObjDataElemProp::odBUDDY(odBUDDY::Update(name,group,btype))))
                }
            },
            16  /*OD_CMPPOSE*/ => {
                let pfl = try!(r.read_u8());
                let seq = try!(r.read_u8());
                let ids1 =
                if (pfl & 2) != 0 {
                    let mut ids = Vec::new();
                    loop {
                        let mut resid = try!(r.read_u16::<le>());
                        if resid == 65535 { break; }
                        if (resid & 0x8000) != 0 {
                            resid &= !0x8000;
                            let sdt_len = try!(r.read_u8()) as usize;
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
                let ids2 =
                if (pfl & 4) != 0 {
                    let mut ids = Vec::new();
                    loop {
                        let mut resid = try!(r.read_u16::<le>());
                        if resid == 65535 { break; }
                        if (resid & 0x8000) != 0 {
                            resid &= !0x8000;
                            let sdt_len = try!(r.read_u8()) as usize;
                            let /*sdt*/ _ = {
                                let mut tmp = vec![0; sdt_len];
                                r.read_exact(&mut tmp).unwrap();
                                tmp
                            };
                        }
                        ids.push(resid);
                    }
                    let ttime = try!(r.read_u8());
                    if !ids.is_empty() {
                        Some((Some(ids),ttime))
                    } else {
                        Some((None,ttime))
                    }
                } else {
                    None
                };
                Ok(Some(ObjDataElemProp::odCMPPOSE(seq,ids1,ids2)))
            },
            17  /*OD_CMPMOD*/ => {
                let mut m = Vec::new();
                loop {
                    let modif = try!(r.read_u16::<le>());
                    if modif == 65535 { break; }
                    let mut ids = Vec::new();
                    loop {
                        let resid = try!(r.read_u16::<le>());
                        if resid == 65535 { break; }
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
            },
            18  /*OD_CMPEQU*/ => {
                let mut e = Vec::new();
                loop {
                    let h = try!(r.read_u8());
                    if h == 255 { break; }
                    let at = r.read_strz().unwrap();
                    let resid = try!(r.read_u16::<le>());
                    let off = if (h & 0x80) != 0 {
                        let x = try!(r.read_u16::<le>());
                        let y = try!(r.read_u16::<le>());
                        let z = try!(r.read_u16::<le>());
                        Some((x,y,z))
                    } else {
                        None
                    };
                    e.push((h&0x7f,at,resid,off));
                }
                let equ = if !e.is_empty() {
                    Some(e)
                } else {
                    None
                };
                Ok(Some(ObjDataElemProp::odCMPEQU(equ)))
            },
            19  /*OD_ICON*/ => {
                let resid = try!(r.read_u16::<le>());
                if resid == 65535 {
                    Ok(Some(ObjDataElemProp::odICON(odICON::Del)))
                } else {
                    let /*ifl*/ _ = try!(r.read_u8());
                    Ok(Some(ObjDataElemProp::odICON(odICON::Set(resid))))
                }
            },
            255 /*OD_END*/ => {
                Ok(None)
            },
            _   /*UNKNOWN*/ => {
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
            0 /*SESS*/ => {
                //TODO ??? Ok(Message::sess(err))
                //     impl Message { fn sess (err: u8) -> Message::SESS { ... } }
                match dir {
                    MessageDirection::FromClient => {
                        let /*unknown*/ _ = try!(r.read_u16::<le>());
                        let /*proto*/ _ = r.read_strz().unwrap();
                        let /*version*/ _ = try!(r.read_u16::<le>());
                        let login = r.read_strz().unwrap();
                        let cookie_len = try!(r.read_u16::<le>());
                        let cookie = {
                            let mut tmp = vec![0; cookie_len as usize];
                            try!(r.read_exact(&mut tmp));
                            tmp
                        };
                        Ok( Message::C_SESS( cSess{ login : login, cookie : cookie } ) )
                    }
                    MessageDirection::FromServer => {
                        Ok( Message::S_SESS( sSess{ err : SessError::new(try!(r.read_u8())) } ) )
                    }
                }
            },
            1 /*REL*/ => {
                let seq = try!(r.read_u16::<le>());
                let mut rel_vec = Vec::new();
                loop {
                    let mut rel_type = match r.read_u8() {
                        Ok(b) => {b}
                        Err(_) => {break;}
                    };
                    let rel_buf = if (rel_type & 0x80) != 0 {
                        rel_type &= !0x80;
                        let rel_len = try!(r.read_u16::<le>());
                        let mut tmp = vec![0; rel_len as usize];
                        r.read_exact(&mut tmp).unwrap();
                        tmp
                    } else {
                        let mut tmp = Vec::new();
                        try!(r.read_to_end(&mut tmp));
                        tmp
                    };
                    rel_vec.push(try!(RelElem::from_buf(rel_type, rel_buf.as_slice())));
                }
                Ok( Message::REL( Rel{ seq : seq, rel : rel_vec } ) )
            },
            2 /*ACK*/ => {
                Ok( Message::ACK( Ack{ seq : try!(r.read_u16::<le>()) } ) )
            },
            3 /*BEAT*/ => {
                Ok( Message::BEAT )
            },
            4 /*MAPREQ*/ => {
                Ok( Message::MAPREQ( MapReq {
                    x:try!(r.read_i32::<le>()),
                    y:try!(r.read_i32::<le>()),
                } ) )
            },
            5 /*MAPDATA*/ => {
                let pktid = try!(r.read_i32::<le>());
                let off = try!(r.read_u16::<le>());
                let len = try!(r.read_u16::<le>());
                let mut buf = Vec::new();
                try!(r.read_to_end(&mut buf));
                //info!("    pktid={} off={} len={}", pktid, off, len);
                //if (off == 0) {
                //    info!("      coord=({}, {})", r.read_i32::<le>().unwrap(), r.read_i32::<le>().unwrap());
                //    info!("      mmname=\"{}\"", r.read_until(0).unwrap());
                //    loop {
                //        let pidx = r.read_u8().unwrap();
                //        if pidx == 255 break;
                //    }
                //}
                Ok( Message::MAPDATA( MapData{ pktid:pktid, off:off, len:len, buf:buf } ) )
            },
            6 /*OBJDATA*/ => {
                let mut obj = Vec::new();
                loop {
                    let fl = match r.read_u8() {
                        Ok(b) => {b}
                        Err(_) => {break;}
                    };
                    let id = try!(r.read_u32::<le>());
                    let frame = try!(r.read_i32::<le>());
                    let mut prop = Vec::new();
                    while let Some(p) = try!(ObjDataElemProp::from_buf(&mut r)) { prop.push(p) }
                    obj.push( ObjDataElem{ fl:fl, id:id, frame:frame, prop:prop } );
                }
                Ok( Message::OBJDATA( ObjData{ obj : obj } ) )
            },
            7 /*OBJACK*/ => {
                //TODO FIXME parse ObjAck instead of empty return
                Ok( Message::OBJACK(ObjAck{obj:Vec::new()}) )
            },
            8 /*CLOSE*/ => {
                Ok( Message::CLOSE/*(Close)*/ )
            },
            _ /*UNKNOWN*/ => { Err( Error{ source:"unknown message type", detail:None } ) }
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
                try!(w.write_u8(0)); // SESS
                try!(w.write_u16::<le>(2)); // unknown
                try!(w.write("Salem".as_bytes())); // proto
                try!(w.write_u8(0));
                try!(w.write_u16::<le>(36)); // version
                try!(w.write(sess.login.as_bytes())); // login
                try!(w.write_u8(0));
                try!(w.write_u16::<le>(32)); // cookie length
                try!(w.write(sess.cookie.as_slice())); // cookie
                Ok(w)
            }
            Message::S_SESS(/*ref sess*/ _ ) => {
                Err( Error{ source:"sSess.to_buf is not implemented yet", detail:None } )
            }
            Message::ACK(ref ack) => /*ack (seq: u16) -> Vec<u8>*/ {
                let mut w = vec![];
                try!(w.write_u8(2)); //ACK
                try!(w.write_u16::<le>(ack.seq));
                Ok(w)
            }
            Message::BEAT => /* beat () -> Vec<u8> */ {
                let mut w = vec![];
                try!(w.write_u8(3)); //BEAT
                Ok(w)
            }
            Message::REL(ref rel) => /* rel_wdgmsg_play (seq: u16, name: &str) -> Vec<u8> */ {
                let mut w = vec![];
                try!(w.write_u8(1)); // REL
                try!(w.write_u16::<le>(rel.seq));// sequence
                for i in 0 .. rel.rel.len() {
                    let rel_elem = &rel.rel[i];
                    let last_one = i == (rel.rel.len() - 1);
                    let rel_elem_buf = try!(rel_elem.to_buf(last_one));
                    try!(w.write(&rel_elem_buf));
                }
                Ok(w)
            }
            Message::MAPREQ(ref mapreq) => /* mapreq (x:i32, y:i32) -> Vec<u8> */ {
                let mut w = vec![];
                try!(w.write_u8(4)); // MAPREQ
                try!(w.write_i32::<le>(mapreq.x)); // x
                try!(w.write_i32::<le>(mapreq.y)); // y
                Ok(w)
            }
            Message::OBJACK(ref objack) => {
                let mut w = vec![];
                w.write_u8(7).unwrap(); //OBJACK writer
                for o in &objack.obj {
                    w.write_u32::<le>(o.id).unwrap();
                    w.write_i32::<le>(o.frame).unwrap();
                }
                Ok(w)
            }
            Message::CLOSE => {
                let mut w = vec![];
                try!(w.write_u8(8)); //CLOSE
                Ok(w)
            }
            _ => {
                Err( Error{ source:"unknown message type", detail:None } )
            }
        }
    }
}

//#![feature(int_uint)]
//#![allow(unstable)]
#![feature(rustc_private)]
#![feature(convert)]
#![feature(ip_addr)]
#![feature(collections)]
#![feature(lookup_host)]

extern crate openssl;
extern crate rustc_serialize;
extern crate mio;
extern crate byteorder;

#[macro_use]
extern crate log;

//use std::net::tcp;
use std::net::TcpStream;
use std::net::UdpSocket;
use std::net::SocketAddr;
use std::collections::hash_map::HashMap;
use std::collections::hash_set::HashSet;
use std::collections::LinkedList;
use std::str;
use rustc_serialize::hex::ToHex;
use openssl::crypto::hash::Type;
use openssl::crypto::hash::hash;
use openssl::ssl::{SslMethod, SslContext, SslStream};
use std::vec::Vec;
use std::fmt::Debug;
use std::fmt::Formatter;
use std::io::Cursor;
use byteorder::{LittleEndian, BigEndian, ReadBytesExt, WriteBytesExt};
use std::io::Read;
use std::io::BufRead;
use std::io::Write;
//use std::io::ErrorKind;

#[derive(Debug)]
struct Error {
    source: &'static str,
    detail: Option<String>,
}

impl From<byteorder::Error> for Error {
    fn from (_:byteorder::Error) -> Error { Error {source:"TODO: some error", detail:None} }
}

impl From<std::io::Error> for Error {
    fn from (_:std::io::Error) -> Error { Error {source:"TODO: some error", detail:None} }
}

struct Obj {
    resid : u16,
    xy : (i32,i32),
}

#[derive(Debug)]
struct NewWdg {
    id : u16,
    kind : String,
    parent : u16,
    pargs : Vec<MsgList>,
    cargs : Vec<MsgList>,
}
#[derive(Debug)]
struct WdgMsg {
    id : u16,
    name : String,
    args : Vec<MsgList>,
}
#[derive(Debug)]
struct DstWdg {
    id : u16,
}
#[derive(Debug)]
struct MapIv;
#[derive(Debug)]
struct GlobLob;
#[derive(Debug)]
struct Paginae;
#[derive(Debug)]
struct ResId {
    id : u16,
    name : String,
    ver : u16,
}
#[derive(Debug)]
struct Party;
#[derive(Debug)]
struct Sfx;
#[derive(Debug)]
struct Cattr;
#[derive(Debug)]
struct Music;
struct Tiles {
    tiles : Vec<TilesElem>
}
impl Debug for Tiles {
    fn fmt(&self, f : &mut Formatter) -> std::fmt::Result {
        write!(f, "    TILES")
    }
}
#[derive(Debug)]
struct TilesElem {
    id : u8,
    name : String,
    ver : u16,
}
#[derive(Debug)]
struct Buff;
#[derive(Debug)]
struct SessKey;

#[derive(Debug)]
//TODO replace with plain struct variants
enum RelElem {
    NEWWDG(NewWdg),
    WDGMSG(WdgMsg),
    DSTWDG(DstWdg),
    MAPIV(MapIv),
    GLOBLOB(GlobLob),
    PAGINAE(Paginae),
    RESID(ResId),
    PARTY(Party),
    SFX(Sfx),
    CATTR(Cattr),
    MUSIC(Music),
    TILES(Tiles),
    BUFF(Buff),
    SESSKEY(SessKey)
}

#[allow(non_camel_case_types)]
#[derive(Debug)]
//TODO replace with plain struct variants
enum MsgList {
    tINT    (i32),
    tSTR    (String),
    tCOORD  ((i32,i32)),
    tUINT8  (u8),
    tUINT16 (u16),
    tCOLOR  ((u8,u8,u8,u8)),
    tTTOL   /*TODO (here should be sublist)*/,
    tINT8   (i8),
    tINT16  (i16),
    tNIL    /*(this is null)*/,
    tBYTES  (Vec<u8>),
    tFLOAT32(f32),
    tFLOAT64(f64),
}

#[allow(non_camel_case_types)]
type le = LittleEndian;
#[allow(non_camel_case_types)]
type be = BigEndian;

fn read_sublist (r : &mut std::io::Cursor<&[u8]> /*buf : &[u8]*/) /*TODO return Result instead*/ {
    //let r = Cursor::new(buf);
    let mut deep = 0;
    loop {
        //if r.len() == 0 { return; }
        let t = match r.read_u8() {
            Ok(b) => {b}
            Err(_) => {return;}
        };
        match t {
            /*T_END    */  0  => { if deep == 0 { return; } else { deep -= 1; } },
            /*T_INT    */  1  => { r.read_i32::<le>().unwrap(); },
            /*T_STR    */  2  => { let mut tmp = Vec::new(); r.read_until(0, &mut tmp).unwrap(); },
            /*T_COORD  */  3  => { r.read_i32::<le>().unwrap(); r.read_i32::<le>().unwrap(); },
            /*T_UINT8  */  4  => { r.read_u8().unwrap(); },
            /*T_UINT16 */  5  => { r.read_u16::<le>().unwrap(); },
            /*T_COLOR  */  6  => { r.read_u8().unwrap(); r.read_u8().unwrap(); r.read_u8().unwrap(); r.read_u8().unwrap(); },
            /*T_TTOL   */  8  => { deep += 1; },
            /*T_INT8   */  9  => { r.read_i8().unwrap(); },
            /*T_INT16  */  10 => { r.read_i16::<le>().unwrap(); },
            /*T_NIL    */  12 => { },
            /*T_BYTES  */  14 => {
                let len = r.read_u8().unwrap();
                if (len & 128) != 0 {
                    let len = r.read_i32::<le>().unwrap(); /* WHY NOT u32 ??? */
                    let mut bytes = vec![0; len as usize];
                    let b = r.read(&mut bytes).unwrap();
                    assert_eq!(b, len as usize);
                } else {
                    let mut bytes = vec![0; len as usize];
                    let b = r.read(&mut bytes).unwrap();
                    assert_eq!(b, len as usize);
                }
            },
            /*T_FLOAT32*/  15 => { r.read_f32::<le>().unwrap(); },
            /*T_FLOAT64*/  16 => { r.read_f64::<le>().unwrap(); },
                           _  => { return; /*TODO return Error instead*/ },
        }
    }
}

fn read_list (r : &mut std::io::Cursor<&[u8]>) -> Vec<MsgList> /*TODO return Result instead*/ {
    //let r = Cursor::new(buf);
    let mut list = Vec::new();
    loop {
        //if r.eof() { return list; }
        //let t = r.read_u8().unwrap();
        let t = match r.read_u8() {
            Ok(b) => {b}
            Err(_) => {return list;}
        };
        match t {
            /*T_END    */  0  => { return list; },
            /*T_INT    */  1  => {
                list.push(MsgList::tINT( r.read_i32::<le>().unwrap() ));
            },
            /*T_STR    */  2  => {
                let mut tmp = Vec::new();
                r.read_until(0, &mut tmp).unwrap();
                list.push(MsgList::tSTR( String::from_utf8(tmp).unwrap() ));
            },
            /*T_COORD  */  3  => {
                list.push(MsgList::tCOORD( (r.read_i32::<le>().unwrap(),r.read_i32::<le>().unwrap()) ));
            },
            /*T_UINT8  */  4  => {
                list.push(MsgList::tUINT8( r.read_u8().unwrap() ));
            },
            /*T_UINT16 */  5  => {
                list.push(MsgList::tUINT16( r.read_u16::<le>().unwrap() ));
            },
            /*T_COLOR  */  6  => {
                list.push(MsgList::tCOLOR( (r.read_u8().unwrap(),
                                            r.read_u8().unwrap(),
                                            r.read_u8().unwrap(),
                                            r.read_u8().unwrap()) ));
            },
            /*T_TTOL   */  8  => {
                read_sublist(r); list.push(MsgList::tTTOL);
            },
            /*T_INT8   */  9  => {
                list.push(MsgList::tINT8( r.read_i8().unwrap() ));
            },
            /*T_INT16  */  10 => {
                list.push(MsgList::tINT16( r.read_i16::<le>().unwrap() ));
            },
            /*T_NIL    */  12 => {
                list.push(MsgList::tNIL);
            },
            /*T_BYTES  */  14 => {
                let len = r.read_u8().unwrap();
                if (len & 128) != 0 {
                    let len = r.read_i32::<le>().unwrap(); /* WHY NOT u32 ??? */
                    let mut bytes = vec![0; len as usize];
                    let b = r.read(&mut bytes).unwrap();
                    assert_eq!(b, len as usize);
                    list.push(MsgList::tBYTES( bytes ));
                } else {
                    let mut bytes = vec![0; len as usize];
                    let b = r.read(&mut bytes).unwrap();
                    assert_eq!(b, len as usize);
                    list.push(MsgList::tBYTES( bytes ));
                }
            },
            /*T_FLOAT32*/  15 => {
                list.push(MsgList::tFLOAT32( r.read_f32::<le>().unwrap() ));
            },
            /*T_FLOAT64*/  16 => {
                list.push(MsgList::tFLOAT64( r.read_f64::<le>().unwrap() ));
            },
            /*UNKNOWN*/    _  => {
                println!("    !!! UNKNOWN LIST ELEMENT !!!");
                return list; /*TODO return Error instead*/
            },
        }
    }
}

impl RelElem {
    // TODO in the case of Err return Error with backtrace instead of String
    fn from_buf (kind:u8, buf:&[u8]) -> Result<RelElem,Error> {
        let mut r = Cursor::new(buf);
        //XXX RemoteUI.java +53
        match kind {
            0  /*NEWWDG*/  => {
                let id = try!(r.read_u16::<le>());
                let kind = {
                    let mut tmp = Vec::new();
                    r.read_until(0, &mut tmp).unwrap();
                    String::from_utf8(tmp).unwrap()
                };
                let parent = try!(r.read_u16::<le>());
                let pargs = read_list(&mut r);
                let cargs = read_list(&mut r);
                Ok( RelElem::NEWWDG( NewWdg{ id:id, kind:kind, parent:parent, pargs:pargs, cargs:cargs } ) )
            },
            1  /*WDGMSG*/  => {
                let id = try!(r.read_u16::<le>());
                let name = {
                    let mut tmp = Vec::new();
                    r.read_until(0, &mut tmp).unwrap();
                    String::from_utf8(tmp).unwrap()
                };
                let args = read_list(&mut r);
                Ok( RelElem::WDGMSG( WdgMsg{ id:id, name:name, args:args } ) )
            },
            2  /*DSTWDG*/  => {
                let id = try!(r.read_u16::<le>());
                Ok( RelElem::DSTWDG( DstWdg{ id:id } ) )
            },
            3  /*MAPIV*/   => { Ok( RelElem::MAPIV(MapIv) ) },
            4  /*GLOBLOB*/ => { Ok( RelElem::GLOBLOB(GlobLob) ) },
            5  /*PAGINAE*/ => { Ok( RelElem::PAGINAE(Paginae) ) },
            6  /*RESID*/   => {
                let id = try!(r.read_u16::<le>());
                let name = {
                    let mut tmp = Vec::new();
                    r.read_until(0, &mut tmp).unwrap();
                    String::from_utf8(tmp).unwrap()
                };
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
                        Ok(b) => {b}
                        Err(_) => {break;}
                    };
                    let name = {
                        let mut tmp = Vec::new();
                        r.read_until(0, &mut tmp).unwrap();
                        String::from_utf8(tmp).unwrap()
                    };
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
    fn to_buf (&self) -> Result<&[u8],Error> {
        //TODO FIXME to be implemented
        Ok(&[])
    }
    //fn wdgmsg (id, name, args) {
    //    RelElem::WDGMSG( WdgMsg{ id:id, name:name, args:args } )
    //}
}

#[derive(Debug)]
enum SessError {
    OK,
    AUTH,
    BUSY,
    CONN,
    PVER,
    EXPR,
    UNKNOWN(u8)
}
impl SessError {
    fn new(t:u8) -> SessError {
        match t {
            0 => { SessError::OK },
            1 => { SessError::AUTH },
            2 => { SessError::BUSY },
            3 => { SessError::CONN },
            4 => { SessError::PVER },
            5 => { SessError::EXPR },
            _ => { SessError::UNKNOWN(t) },
        }
    }
}
#[allow(non_camel_case_types)]
#[derive(Debug)]
struct sSess {
    err : SessError,
}
#[allow(non_camel_case_types)]
#[derive(Debug)]
struct cSess {
    login : String,
    cookie : Vec<u8>
}
struct Rel {
    seq : u16,
    rel : Vec<RelElem>
}
#[allow(dead_code)]
impl Rel {
    fn new (seq:u16) -> Rel {
        Rel{ seq:seq, rel:Vec::new() }
    }
    fn append (&mut self, elem:RelElem) {
        self.rel.push(elem);
    }
}
impl Debug for Rel {
    fn fmt(&self, f : &mut Formatter) -> std::fmt::Result {
        write!(f, "REL seq={}", self.seq)
    }
}
#[derive(Debug)]
struct Ack {
    seq : u16,
}
#[allow(dead_code)]
#[derive(Debug)]
struct Beat;
#[derive(Debug)]
struct MapReq {
    x : i32,
    y : i32,
}
struct MapData {
    pktid : i32,
    off   : u16,
    len   : u16,
    buf   : Vec<u8>,
}
impl Debug for MapData {
    fn fmt(&self, f : &mut Formatter) -> std::fmt::Result {
        write!(f, "MAPDATA pktid:{} offset:{} len:{} buf:[..{}]", self.pktid, self.off, self.len, self.buf.len())
    }
}
struct ObjData {
    obj : Vec<ObjDataElem>,
}
impl Debug for ObjData {
    fn fmt(&self, f : &mut Formatter) -> std::fmt::Result {
        write!(f, "OBJDATA")
    }
}
#[derive(Debug)]
struct ObjDataElem {
    fl    : u8,
    id    : u32,
    frame : i32,
    prop  : Vec<ObjProp>,
}
#[derive(Debug)]
struct ObjAck {
    obj : Vec<ObjAckElem>,
}
impl ObjAck {
    fn new (objdata: &ObjData) -> ObjAck {
        let mut objack = ObjAck{ obj : Vec::new() };
        for o in objdata.obj.iter() {
            objack.obj.push(ObjAckElem{ id : o.id, frame : o.frame});
        }
        objack
    }
}
#[derive(Debug)]
struct ObjAckElem {
    id : u32,
    frame : i32,
}
#[derive(Debug)]
struct Close;

#[allow(non_camel_case_types)]
#[derive(Debug)]
//TODO replace with plain struct variants
enum Message {
    C_SESS( cSess ),
    S_SESS( sSess ),
    REL( Rel ),
    ACK( Ack ),
    BEAT,
    MAPREQ( MapReq ),
    MAPDATA( MapData ),
    OBJDATA( ObjData ),
    OBJACK( ObjAck ),
    CLOSE( Close ),
}

#[allow(non_camel_case_types)]
#[derive(Debug)]
//TODO replace with plain struct variants
enum ObjProp {
    odREM,
    odMOVE((i32,i32),u16),
    odRES(u16),
    odLINBEG((i32,i32),(i32,i32),i32),
    odLINSTEP(i32),
    odSPEECH(u16,String),
    odCOMPOSE(u16),
    odDRAWOFF((i32,i32)),
    odLUMIN((i32,i32),u16,u8),
    odAVATAR(Vec<u16>),
    odFOLLOW(odFOLLOW),
    odHOMING(odHOMING),
    odOVERLAY(u16),
    odAUTH,
    odHEALTH(u8),
    odBUDDY(odBUDDY),
    odCMPPOSE,
    odCMPMOD,
    odCMPEQU,
    odICON(odICON),
}
#[allow(non_camel_case_types)]
#[derive(Debug)]
enum odFOLLOW {
    Stop,
    To(u32,u16,String),
}
#[allow(non_camel_case_types)]
#[derive(Debug)]
enum odHOMING {
    New((i32,i32),u16),
    Change((i32,i32),u16),
    Delete,
}
#[allow(non_camel_case_types)]
#[derive(Debug)]
enum odBUDDY {
    Update(String,u8,u8),
    Delete,
}
#[allow(non_camel_case_types)]
#[derive(Debug)]
enum odICON {
    Set(u16),
    Del,
}

impl ObjProp {
    fn from_buf (r : &mut std::io::Cursor<&[u8]>) -> Result<Option<ObjProp>,Error> {
        //let r = Cursor::new(buf);
        let t = try!(r.read_u8()) as usize;
        match t {
            0   /*OD_REM*/ => {
                Ok(Some(ObjProp::odREM))
            },
            1   /*OD_MOVE*/ => {
                let xy = (try!(r.read_i32::<le>()), try!(r.read_i32::<le>()));
                let ia = try!(r.read_u16::<le>());
                Ok(Some(ObjProp::odMOVE(xy,ia)))
            },
            2   /*OD_RES*/ => {
                let mut resid = try!(r.read_u16::<le>());
                if (resid & 0x8000) != 0 {
                    resid &= !0x8000;
                    let sdt_len = r.read_u8().unwrap();
                    let mut sdt = vec![0; sdt_len as usize];
                    let b = r.read(&mut sdt).unwrap();
                    assert_eq!(b, sdt_len as usize);
                    //let _/*sdt*/ = try!(r.read_exact(sdt_len));
                }
                Ok(Some(ObjProp::odRES(resid)))
            },
            3   /*OD_LINBEG*/ => {
                let s = (try!(r.read_i32::<le>()), try!(r.read_i32::<le>()));
                let t = (try!(r.read_i32::<le>()), try!(r.read_i32::<le>()));
                let c = try!(r.read_i32::<le>());
                Ok(Some(ObjProp::odLINBEG(s,t,c)))
            },
            4   /*OD_LINSTEP*/ => {
                let l = try!(r.read_i32::<le>());
                Ok(Some(ObjProp::odLINSTEP(l)))
            },
            5   /*OD_SPEECH*/ => {
                let zo = try!(r.read_u16::<le>());
                let text = {
                    let mut tmp = Vec::new();
                    r.read_until(0, &mut tmp).unwrap();
                    String::from_utf8(tmp).unwrap()
                };
                Ok(Some(ObjProp::odSPEECH(zo,text)))
            },
            6   /*OD_COMPOSE*/ => {
                let resid = try!(r.read_u16::<le>());
                Ok(Some(ObjProp::odCOMPOSE(resid)))
            },
            7   /*OD_DRAWOFF*/ => {
                let off = (try!(r.read_i32::<le>()), try!(r.read_i32::<le>()));
                Ok(Some(ObjProp::odDRAWOFF(off)))
            },
            8   /*OD_LUMIN*/ => {
                let off = (try!(r.read_i32::<le>()), try!(r.read_i32::<le>()));
                let sz = try!(r.read_u16::<le>());
                let str_ = try!(r.read_u8());
                Ok(Some(ObjProp::odLUMIN(off,sz,str_)))
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
                Ok(Some(ObjProp::odAVATAR(layers)))
            },
            10  /*OD_FOLLOW*/ => {
                let oid = try!(r.read_u32::<le>());
                if oid == 0xff_ff_ff_ff {
                    Ok(Some(ObjProp::odFOLLOW(odFOLLOW::Stop)))
                } else {
                    let xfres = try!(r.read_u16::<le>());
                    let xfname = {
                        let mut tmp = Vec::new();
                        r.read_until(0, &mut tmp).unwrap();
                        String::from_utf8(tmp).unwrap()
                    };
                    Ok(Some(ObjProp::odFOLLOW(odFOLLOW::To(oid,xfres,xfname))))
                }
            },
            11  /*OD_HOMING*/ => {
                let oid = try!(r.read_u32::<le>());
                match oid {
                    0xff_ff_ff_ff => {
                        Ok(Some(ObjProp::odHOMING(odHOMING::Delete)))
                    },
                    0xff_ff_ff_fe => {
                        let tgtc = (try!(r.read_i32::<le>()), try!(r.read_i32::<le>()));
                        let v = try!(r.read_u16::<le>());
                        Ok(Some(ObjProp::odHOMING(odHOMING::Change(tgtc,v))))
                    },
                    _             => {
                        let tgtc = (try!(r.read_i32::<le>()), try!(r.read_i32::<le>()));
                        let v = try!(r.read_u16::<le>());
                        Ok(Some(ObjProp::odHOMING(odHOMING::New(tgtc,v))))
                    }
                }
            },
            12  /*OD_OVERLAY*/ => {
                /*let olid =*/ try!(r.read_i32::<le>());
                let resid = try!(r.read_u16::<le>());
                if resid != 65535 {
                    if (resid & 0x8000) != 0 {
                        let sdt_len = try!(r.read_u8()) as usize;
                        let mut sdt = vec![0; sdt_len as usize];
                        let b = r.read(&mut sdt).unwrap();
                        assert_eq!(b, sdt_len as usize);
                        //let sdt = try!(r.read_exact(sdt_len)); //TODO
                    }
                }
                Ok(Some(ObjProp::odOVERLAY( resid&(!0x8000) )))
            },
            13  /*OD_AUTH*/   => {
                Ok(Some(ObjProp::odAUTH)) // Removed
            },
            14  /*OD_HEALTH*/ => {
                let hp = try!(r.read_u8());
                Ok(Some(ObjProp::odHEALTH(hp)))
            },
            15  /*OD_BUDDY*/ => {
                let name = {
                    let mut tmp = Vec::new();
                    r.read_until(0, &mut tmp).unwrap();
                    String::from_utf8(tmp).unwrap()
                };
                //XXX FIXME C string is not like Rust string, it has \0 at the end,
                //          so this check is incorrect, I SUPPOSE.
                //          MOST PROBABLY we will crash here because 2 more readings.
                if name.len() == 0 {
                    Ok(Some(ObjProp::odBUDDY(odBUDDY::Delete)))
                } else {
                    let group = try!(r.read_u8());
                    let btype = try!(r.read_u8());
                    Ok(Some(ObjProp::odBUDDY(odBUDDY::Update(name,group,btype))))
                }
            },
            16  /*OD_CMPPOSE*/ => {
                let pfl = try!(r.read_u8());
                /*let seq =*/ try!(r.read_u8());
                if (pfl & 2) != 0 {
                    loop {
                        let /*mut*/ resid = try!(r.read_u16::<le>());
                        if resid == 65535 { break; }
                        if (resid & 0x8000) != 0 {
                            /*resid &= !0x8000;*/
                            let sdt_len = try!(r.read_u8()) as usize;
                            let mut sdt = vec![0; sdt_len as usize];
                            let b = r.read(&mut sdt).unwrap();
                            assert_eq!(b, sdt_len as usize);
                            //let sdt = try!(r.read_exact(sdt_len));
                        }
                    }
                }
                if (pfl & 4) != 0 {
                    loop {
                        let /*mut*/ resid = try!(r.read_u16::<le>());
                        if resid == 65535 { break; }
                        if (resid & 0x8000) != 0 {
                            /*resid &= !0x8000;*/
                            let sdt_len = try!(r.read_u8()) as usize;
                            let mut sdt = vec![0; sdt_len as usize];
                            let b = r.read(&mut sdt).unwrap();
                            assert_eq!(b, sdt_len as usize);
                            //let sdt = try!(r.read_exact(sdt_len));
                        }
                    }
                    let /*ttime*/ _ = try!(r.read_u8());
                }
                Ok(Some(ObjProp::odCMPPOSE))
            },
            17  /*OD_CMPMOD*/ => {
                loop {
                    let modif = try!(r.read_u16::<le>());
                    if modif == 65535 { break; }
                    loop {
                        let resid = try!(r.read_u16::<le>());
                        if resid == 65535 { break; }
                    }
                }
                Ok(Some(ObjProp::odCMPMOD))
            },
            18  /*OD_CMPEQU*/ => {
                loop {
                    let h = try!(r.read_u8());
                    if h == 255 { break; }
                    let /*at*/ _ = {
                        let mut tmp = Vec::new();
                        r.read_until(0, &mut tmp).unwrap();
                        String::from_utf8(tmp).unwrap()
                    };
                    let /*resid*/ _ = try!(r.read_u16::<le>());
                    if (h & 0x80) != 0 {
                        let /*x*/ _ = try!(r.read_u16::<le>());
                        let /*y*/ _ = try!(r.read_u16::<le>());
                        let /*z*/ _ = try!(r.read_u16::<le>());
                    }
                }
                Ok(Some(ObjProp::odCMPEQU))
            },
            19  /*OD_ICON*/ => {
                let resid = try!(r.read_u16::<le>());
                if resid == 65535 {
                    Ok(Some(ObjProp::odICON(odICON::Del)))
                } else {
                    let /*ifl*/ _ = try!(r.read_u8());
                    Ok(Some(ObjProp::odICON(odICON::Set(resid))))
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

//use std::old_io::IoErrorKind;
enum MessageDirection {
    FromClient,
    FromServer,
}

impl Message {
    //TODO
    // fn from_buf_checked (buf,dir) {
    //     if (this message can be received by this dir) {
    //         return Ok(buf.from_buf)
    //     else
    //         return Err("this king of message can't be received by this side")
    // }
    //TODO return Error with stack trace on Err instead of String
    //TODO get Vec not &[]. return Vec in the case of error
    fn from_buf (buf : &[u8], dir : MessageDirection) -> Result<Message,Error> {
        let mut r = Cursor::new(buf);
        let mtype = try!(r.read_u8());
        let res = match mtype {
            0 /*SESS*/ => {
                //TODO ??? Ok(Message::sess(err))
                //     impl Message { fn sess (err: u8) -> Message::SESS { ... } }
                match dir {
                    MessageDirection::FromClient => {
                        Ok( Message::C_SESS( cSess{ login : "TODO".to_string(), cookie : vec![] } ) )
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
                    //let mut rel_type = try!(r.read_u8());
                    let rel_buf = if (rel_type & 0x80) != 0 {
                        rel_type &= !0x80;
                        let rel_len = try!(r.read_u16::<le>());
                        let mut tmp = vec![0; rel_len as usize];
                        let b = r.read(&mut tmp).unwrap();
                        assert_eq!(b, rel_len as usize);
                        tmp
                        //try!(r.read_exact(rel_len as usize))
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
                Ok( Message::MAPREQ( MapReq { x:0, y:0 } /*FIXME should read x,y from buf*/ ) )
            },
            5 /*MAPDATA*/ => {
                let pktid = try!(r.read_i32::<le>());
                let off = try!(r.read_u16::<le>());
                let len = try!(r.read_u16::<le>());
                let mut buf = Vec::new();
                try!(r.read_to_end(&mut buf));
                //println!("    pktid={} off={} len={}", pktid, off, len);
                //if (off == 0) {
                //    println!("      coord=({}, {})", r.read_i32::<le>().unwrap(), r.read_i32::<le>().unwrap());
                //    println!("      mmname=\"{}\"", r.read_until(0).unwrap());
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
                    loop {
                        match try!(ObjProp::from_buf(&mut r)) {
                            Some(p) => { prop.push(p) },
                            None => { break },
                        }
                    }
                    obj.push( ObjDataElem{ fl:fl, id:id, frame:frame, prop:prop } );
                }
                Ok( Message::OBJDATA( ObjData{ obj : obj } ) )
            },
            7 /*OBJACK*/ => {
                //TODO FIXME parse ObjAck instead of empty return
                Ok( Message::OBJACK(ObjAck{obj:Vec::new()}) )
            },
            8 /*CLOSE*/ => {
                Ok( Message::CLOSE(Close) )
            },
            _ /*UNKNOWN*/ => { Err( Error{ source:"unknown message type", detail:None } ) }
        };

        let mut remains = Vec::new();
        try!(r.read_to_end(&mut remains));
        if !remains.is_empty() {
            println!("                       REMAINS {} bytes", remains.len());
        }

        res
    }

    fn to_buf (self) -> Result<Vec<u8>,Error> {
        //type write_u16::<le> = Vec<u8>::write_u16::<LittleEndian>;
        let mut w = vec![];
        match self {
            // !!! this is client session message, not server !!!
            Message::C_SESS(sess) => /*(name: &str, cookie: &[u8]) -> Vec<u8>*/ {
                try!(w.write_u8(0)); // SESS
                try!(w.write_u16::<LittleEndian>(2)); // unknown
                try!(w.write("Salem".as_bytes())); // proto
                try!(w.write_u8(0));
                try!(w.write_u16::<le>(34)); // version
                try!(w.write(sess.login.as_bytes())); // login
                try!(w.write_u8(0));
                try!(w.write_u16::<le>(32)); // cookie length
                try!(w.write(sess.cookie.as_slice())); // cookie
                Ok(w)
            }
            Message::S_SESS(/*sess*/ _ ) => {
                Err( Error{ source:"sSess.to_buf is not implemented yet", detail:None } )
            }
            Message::ACK(ack) => /*ack (seq: u16) -> Vec<u8>*/ {
                try!(w.write_u8(2)); //ACK
                try!(w.write_u16::<le>(ack.seq));
                Ok(w)
            }
            Message::BEAT => /* beat () -> Vec<u8> */ {
                try!(w.write_u8(3)); //BEAT
                Ok(w)
            }


//            1 /*REL*/ => {
//                let seq = try!(r.read_u16::<le>());
//                let mut rel_vec = Vec::new();
//                while !r.eof() {
//                    let mut rel_type = try!(r.read_u8());
//                    let rel_buf = if (rel_type & 0x80) != 0 {
//                        rel_type &= !0x80;
//  FIXME                 let rel_len = try!(r.read_u16::<le>());
//                        try!(r.read_exact(rel_len as usize))
//                    } else {
//                        try!(r.read_to_end())
//                    };
//                    rel_vec.push(try!(RelElem::from_buf(rel_type, rel_buf.as_slice())));
//                }
//                Ok( Message::REL( Rel{ seq : seq, rel : rel_vec } ) )
//            },



            Message::REL(rel) => /* rel_wdgmsg_play (seq: u16, name: &str) -> Vec<u8> */ {
                try!(w.write_u8(1)); // REL
                try!(w.write_u16::<le>(rel.seq));// sequence
                for rel_elem in rel.rel {
                    let rel_elem_buf = try!(rel_elem.to_buf());
                    try!(w.write(rel_elem_buf));

//                    try!(w.write_u8(1));// rel type WDGMSG
//                    try!(w.write_u16::<le>(3));// widget id
//                    try!(w.write("play".as_bytes()));// message name
//  FIXME             try!(w.write_u8(0));
//                    // args list
//                    try!(w.write_u8(2)); // list element type T_STR
//                    try!(w.write(rel.name.as_bytes())); // element
//                    try!(w.write_u8(0));
                }
                Ok(w)
            }
            Message::MAPREQ(mapreq) => /* mapreq (x:i32, y:i32) -> Vec<u8> */ {
                try!(w.write_u8(4)); // MAPREQ
                try!(w.write_i32::<le>(mapreq.x)); // x
                try!(w.write_i32::<le>(mapreq.y)); // y
                Ok(w)
            }
            Message::OBJACK(objack) => {
                //type write_u32::<le> = Vec::write_u32<LittleEndian>;
                let mut w = vec![];
                w.write_u8(7).unwrap(); //OBJACK writer
                for o in objack.obj.iter() {
                    w.write_u32::<le>(o.id).unwrap();
                    w.write_i32::<le>(o.frame).unwrap();
                }
                Ok(w)
            }
            _ => {
                Err( Error{ source:"unknown message type", detail:None } )
            }
        }
    }
}

struct Client {
    user: &'static str,
    //pass: &'static str,
    cookie: Vec<u8>,
    //host: &'static str,
    //auth_port: u16,
    //port: u16,
    //game_addr: SocketAddr,
    //any_addr: SocketAddr,
    //udp_rx: UdpSocket,
    //udp_tx: UdpSocket,
    //main_to_sender: Sender<Vec<u8>>,     //TODO type OutputBuffer = Vec<u8>
    //sender_from_any: Receiver<Vec<u8>>,  //TODO type OutputBuffer = Vec<u8>
    //receiver_to_sender: Sender<Vec<u8>>, //TODO type OutputBuffer = Vec<u8>
    //beater_to_sender: Sender<Vec<u8>>,
    //receiver_to_main: Sender<()>,
    //main_from_any: Receiver<()>,
    //receiver_to_beater: Sender<()>,
    //beater_from_any: Receiver<()>,
    //receiver_to_viewer: Sender<(u32,Obj)>,
    //viewer_from_any: Receiver<Data>,
    //objects: HashMap<u32,Obj>,
    //resources: HashMap<u16,String>,
    //widgets: HashMap<usize,String>,
    //control_rx: Receiver<Control>,
    //control_tx: Sender<String>,
    widgets : HashMap<u16,String>,
    objects : HashMap<u32,Obj>,
    grids : HashSet<(i32,i32)>,
    charlist : Vec<String>,
    resources : HashMap<u16,String>,
}

/*
#[derive(Debug)]
enum Control {
    Dump,
    Quit,
}
*/

impl Client {
    fn new (/*host: &'static str, auth_port: u16, port: u16*/) -> Client {
        //let host_ip = get_host_addresses(host).unwrap()[0];
        //let any_addr = SocketAddr {ip: Ipv4Addr(0,0,0,0), port: 0};
        //let sock = UdpSocket::bind(any_addr).unwrap();
        let mut widgets = HashMap::new();
        widgets.insert(0, "root".to_string());
        let objects = HashMap::new();
        let grids = HashSet::new();
        let charlist = Vec::new();
        let resources = HashMap::new();

        //let (tx1,rx1) = channel(); // any -> sender   (packet to send)
        //let (tx2,rx2) = channel(); // any -> beater   (wakeup signal)
        //let (tx3,rx3) = channel(); // any -> viewer   (objects data)
        //let (tx4,rx4) = channel(); // any -> main     (exit signal) // TODO remove ???
        //let (tx5,rx5) = channel(); // any -> main     (control messages)
        //let (tx6,rx6) = channel(); // main -> control (strings to output)

        // control socket reader
        //let reader_to_main = tx4.clone();
        //let control_to_main = tx5.clone();
        //let control_from_main = rx6;
        /*
        Thread::spawn(move || {
            let path = Path::new("/tmp/socket");
            if path.exists() {
                fs::unlink(&path);
            }
            let socket = UnixListener::bind(&path);
            let mut listener = socket.listen();
            let mut stream = listener.accept();
            //TODO FIXME after stream accepted:
            // create new channel and send it
            // through another channel(which is constant)
            let mut stream_tx = stream.clone();
            // control stream TX
            Thread::spawn(move || {
                loop {
                    let s:String = control_from_main.recv().unwrap();
                    stream_tx.write_line(s.as_slice()).unwrap();
                    stream_tx.flush().unwrap();
                }
            });
            loop {
                // control stream RX
                match stream.read_byte() {
                    Ok(b) => {
                        println!("reader: read: {}", b);
                        match b {
                            b'e' | b'q' => {
                                println!("reader: exit requested");
                                reader_to_main.send(()); //FIXME remove this channel at all
                                control_to_main.send(Control::Quit);
                                //break 'outer;
                            },
                            b'o' => {
                                println!("reader: objects dump requested");
                                control_to_main.send(Control::Dump);
                            },
                            _ => {},
                        }
                        stream.write_u8(b).unwrap();
                    },
                    Err(e) => {
                        println!("reader: error: {}", e);
                        break;
                    },
                }
            }
        });
        */

        Client {
            user: "",
            cookie: Vec::new(),
            //host: host,
            //auth_port: auth_port,
            //port: port,
            //game_addr: SocketAddr {ip: host_ip, port: port},
            //udp_rx: sock.clone(),
            //udp_tx: sock.clone(),

            //main_to_sender: tx1.clone(),
            //sender_from_any: rx1,
            //receiver_to_sender: tx1.clone(),
            //beater_to_sender: tx1.clone(),
            //receiver_to_main: tx2.clone(),
            //main_from_any: rx4,
            //receiver_to_beater: tx2.clone(),
            //beater_from_any: rx2,
            //receiver_to_viewer: tx3.clone(),
            //viewer_from_any: rx3,

            //objects: HashMap::new(),
            //resources: resources,//HashMap::new(),
            //widgets: HashMap::new(),
            //control_rx: rx5,
            //control_tx: tx6,
            widgets: widgets, 
            objects: objects,
            grids: grids,
            charlist: charlist,
            resources:resources,
        }
    }

    fn authorize (&mut self, user: &'static str, pass: &str, ip: std::net::IpAddr, port: u16) -> Result<(), Error> {
        self.user = user;
        //self.pass = pass;
        //let auth_addr: SocketAddr = SocketAddr {ip: ip, port: port};
        let auth_addr = SocketAddr::new(ip, port);
        println!("authorize {} @ {}", user, auth_addr);
        //TODO add method connect(SocketAddr) to TcpStream
        //let stream = tryio!("tcp.connect" TcpStream::connect(self.auth_addr));
        let stream = TcpStream::connect(auth_addr).unwrap();
        let context = SslContext::new(SslMethod::Sslv23).unwrap();
        let mut stream = SslStream::new(&context, stream).unwrap();

        // send 'pw' command
        let user = user.as_bytes();
        let buf_len = (3 + user.len() + 1 + 32) as u16;
        let mut buf: Vec<u8> = Vec::with_capacity((2 + buf_len) as usize);
        buf.write_u16::<be>(buf_len).unwrap();
        buf.push_all("pw".as_bytes());
        buf.push(0);
        buf.push_all(user);
        buf.push(0);
        let pass_hash = hash(Type::SHA256, pass.as_bytes());
        assert!(pass_hash.len() == 32);
        buf.push_all(pass_hash.as_slice());
        stream.write(buf.as_slice()).unwrap();
        stream.flush().unwrap();

        let mut buf = vec![0,0];
        let len = stream.read(buf.as_mut_slice()).ok().expect("read error");
        if len != 2 { return Err(Error{source:"bytes read != 2",detail:None}); }
        //TODO replace byteorder crate with endian crate ???
        let mut rdr = Cursor::new(buf);
        let len = rdr.read_u16::<be>().unwrap();

        let mut msg: Vec<u8> = Vec::with_capacity(len as usize);
        msg.resize(len as usize, 0);
        let len2 = stream.read(msg.as_mut_slice()).ok().expect("read error");
        if len2 != len as usize { return Err(Error{source:"len2 != len",detail:None}); }
        println!("msg='{}'", str::from_utf8(msg.as_slice()).unwrap());
        //println!("msg='{}'", msg.as_slice().to_hex());
        if msg.len() < "ok\0\0".len() {
            return Err(Error{source:"'pw' command unexpected answer", detail:Some(String::from_utf8(msg).unwrap())});
        }

        // send 'cookie' command
        if (msg[0] == ('o' as u8)) && (msg[1] == ('k' as u8)) {
            // TODO tryio!(stream.write(Msg::cookie(params...)));
            let buf_len = ("cookie".as_bytes().len() + 1) as u16;
            let mut buf: Vec<u8> = Vec::with_capacity((2 + buf_len) as usize);
            buf.write_u16::<be>(buf_len).unwrap();
            buf.push_all("cookie".as_bytes());
            buf.push(0);
            stream.write(buf.as_slice()).unwrap();
            stream.flush().unwrap();

            let mut buf = vec![0,0];
            let len = stream.read(buf.as_mut_slice()).ok().expect("read error");
            if len != 2 { return Err(Error{source:"bytes read != 2",detail:None}); }
            //TODO replace byteorder crate with endian crate ???
            let mut rdr = Cursor::new(buf);
            let len = rdr.read_u16::<be>().unwrap();

            let mut msg: Vec<u8> = Vec::with_capacity(len as usize);
            msg.resize(len as usize, 0);
            let len2 = stream.read(msg.as_mut_slice()).ok().expect("read error");
            if len2 != len as usize { return Err(Error{source:"len2 != len",detail:None}); }
            //println!("msg='{}'", str::from_utf8(msg.as_slice()).unwrap());
            println!("msg='{}'", msg.as_slice().to_hex());
            //TODO check cookie length
            self.cookie = msg[3..].to_vec();
            return Ok(());
        }
        return Err(Error{source:"'cookie' command unexpected answer", detail:Some(String::from_utf8(msg).unwrap())});
    }

    fn start_send_beats () {
        /*TODO*/
    }

    fn enqueue_to_send (&self, msg: Message, tx_buf:&mut LinkedList<Vec<u8>>) {
        match msg.to_buf() {
            Ok(buf) => { tx_buf.push_front(buf); },
            Err(e) => { println!("enqueue error: {:?}", e); },
        }
    }

    /*
    fn save_object () {
        /*TODO*/
    }
    */

    fn shutdown_and_exit () {
        /*TODO*/
    }

    fn dispatch_message (&mut self, buf:&[u8], tx_buf:&mut LinkedList<Vec<u8>>) {
        let msg = match Message::from_buf(buf,MessageDirection::FromServer) {
            Ok(msg) => { msg },
            Err(err) => { println!("message parse error: {:?}", err); return; },
        };
        println!("RX: {:?}", msg);
        match msg {
            Message::S_SESS(sess) => {
                match sess.err {
                    SessError::OK => {},
                    _ => {
                        //TODO event_loop.shutdown(); exit();
                        //XXX ??? should we send CLOSE too ???
                        //FIXME
                        //  receiver: SESS(Sess { err: BUSY })
                        //  task '<unnamed>' panicked at 'receiving on a closed channel', ...
                        //  task '<main>' panicked at 'receiving on a closed channel', ...
                        //  task '<unnamed>' panicked at 'receiving on a closed channel',
                    }
                }
                Client::start_send_beats();
            },
            Message::C_SESS( /*sess*/ _ ) => {/*TODO*/},
            Message::REL( rel ) => {
                //TODO do not process duplicates, but ACK only
                //XXX are we handle seq right in the case of overflow ???
                self.enqueue_to_send(Message::ACK(Ack{seq : rel.seq + ((rel.rel.len() as u16) - 1)}), tx_buf);
                for r in rel.rel.iter() {
                    println!("    {:?}", r);
                    match *r {
                        RelElem::NEWWDG(ref wdg) => {
                            self.widgets.insert(wdg.id, wdg.kind.clone()/*FIXME String -> &str*/);
                        },
                        RelElem::WDGMSG(ref msg) => {
                            //TODO match against widget.type and message.type
                            match self.widgets.get(&(msg.id)) {
                                None => {},
                                Some(c) => {
                                    if (c == "charlist\0") && (msg.name == "add\0") {
                                        match msg.args[0] {
                                            MsgList::tSTR(ref char_name) => {
                                                println!("    add char '{}'", char_name);
                                                self.charlist.push(char_name.clone()/*FIXME rewrite without cloning*/);
                                            },
                                            _ => {}
                                        }
                                    }
                                }
                            }
                        },
                        RelElem::DSTWDG(_) => { /*TODO widgets.delete(wdg.id)*/ },
                        RelElem::MAPIV(_) => {},
                        RelElem::GLOBLOB(_) => {},
                        RelElem::PAGINAE(_) => {},
                        RelElem::RESID(ref res) => {
                            self.resources.insert(res.id, res.name.clone()/*FIXME String -> &str*/);
                        },
                        RelElem::PARTY(_) => {},
                        RelElem::SFX(_) => {},
                        RelElem::CATTR(_) => {},
                        RelElem::MUSIC(_) => {},
                        RelElem::TILES(ref tiles) => {
                            for tile in tiles.tiles.iter() {
                                println!("      {:?}", tile);
                            }
                        },
                        RelElem::BUFF(_) => {},
                        RelElem::SESSKEY(_) => {},
                    }
                }
            },
            Message::ACK(_)     => {},
            Message::BEAT    => { println!("     !!! client must not receive BEAT !!!"); },
            Message::MAPREQ(_)  => { println!("     !!! client must not receive MAPREQ !!!"); },
            Message::MAPDATA(_) => {},
            Message::OBJDATA( objdata ) => {
                //TODO receiver_to_sender.send(objdata.to_buf());
                self.enqueue_to_send(Message::OBJACK(ObjAck::new(&objdata)), tx_buf); // send OBJACKs
                for o in objdata.obj.iter() {
                    println!("    {:?}", o);
                }
                //TODO parse objdata in network thread and send here more normalized objects
                for o in objdata.obj.iter() {
                    if !self.objects.contains_key(&o.id) {
                        self.objects.insert(o.id, Obj{resid:0, xy:(0,0)});
                    }
                    match self.objects.get_mut(&o.id) {
                        Some(obj) => {
                            //TODO check for o.frame vs obj.frame
                            for prop in o.prop.iter() {
                                match *prop {
                                    ObjProp::odREM => { /*FIXME objects.remove(&o.id); break;*/ },
                                    ObjProp::odMOVE(xy,_) => { obj.xy = xy; },
                                    ObjProp::odRES(resid) => { obj.resid = resid; },
                                    ObjProp::odCOMPOSE(resid) => { obj.resid = resid; },
                                    _ => {},
                                }
                            }
                        },
                        None => unreachable!(),
                    };
                }
                for o in self.objects.values() {
                    let (x,y) = o.xy;
                    let gx:i32 = x / 1100;
                    let gy:i32 = y / 1100;
                    if !self.grids.contains(&(gx,gy)) {
                        self.mapreq(gx, gy, tx_buf);
                        self.grids.insert((gx,gy));
                    }
                }
            },
            Message::OBJACK(_)  => {},
            Message::CLOSE(_)   => {
                Client::shutdown_and_exit();
            },
        }

        //TODO send REL until reply
        if self.charlist.len() > 0 {
            println!("send play '{}'", self.charlist[0]);
            let char_name = self.charlist[0].clone();
            //FIXME sequence is ALWAYS ZERO!! get sequence from client
            let mut rel = Rel{seq:0, rel:Vec::new()};
            let id : u16 = 3; //FIXME get widget id by name
            let name : String = "play".to_string();
            let mut args : Vec<MsgList> = Vec::new();
            args.push(MsgList::tSTR(char_name));
            let elem = RelElem::WDGMSG(WdgMsg{ id : id, name : name, args : args });
            rel.rel.push(elem);
            self.enqueue_to_send(Message::REL(rel), tx_buf);
            self.charlist.clear();
        }
    }

    fn connect (&self, tx_buf:&mut LinkedList<Vec<u8>>) {
        //TODO send SESS until reply
        //TODO get username from server responce, not from auth username
        self.enqueue_to_send(Message::C_SESS(cSess{login:self.user.to_string(), cookie:self.cookie.clone()}), tx_buf);
    }

    fn mapreq (&self, x:i32, y:i32, tx_buf:&mut LinkedList<Vec<u8>>) {
        //TODO send until reply
        //TODO replace with client.send(Message::MapReq::new(x,y).to_buf())
        //     or client.send(Message::mapreq(x,y).to_buf())
        self.enqueue_to_send(Message::MAPREQ(MapReq{x:x,y:y}), tx_buf);
    }

}


fn main() {
    //TODO handle keyboard interrupt
    //TODO replace all unwraps with normal error handling
    //TODO ADD tests:
    //        for i in range(0u8, 255) {
    //            let mut v = Vec::new();
    //            v.push(i);
    //            println!("{}", Message::from_buf(v.as_slice()));
    //        }

    use mio::Socket;
    //use mio::IoReader;
    //use mio::IoWriter;
    //use mio::event::{READABLE,WRITABLE,LEVEL};

    struct UdpHandler<'a> {
        sock: mio::NonBlock<mio::udp::UdpSocket>,
        addr: std::net::SocketAddr,
        tx_buf: LinkedList<Vec<u8>>,
        client: &'a mut Client,
        start: bool,
    }
    impl<'a> UdpHandler<'a> {
        fn new(sock: mio::NonBlock<mio::udp::UdpSocket>, client:&'a mut Client, addr: std::net::SocketAddr) -> UdpHandler<'a> {
            UdpHandler {
                sock: sock,
                addr: addr,
                tx_buf: LinkedList::new(),
                client: client,
                start: true,
            }
        }
    }
    const CLIENT: mio::Token = mio::Token(0);
    impl<'a> mio::Handler for UdpHandler<'a> {
        type Timeout = usize;
        type Message = ();
        fn readable(&mut self, _/*event_loop*/: &mut mio::EventLoop<UdpHandler>, token: mio::Token, _: mio::ReadHint) {
            //use mio::buf::Buf;
            match token {
                CLIENT => {
                    let mut rx_buf = mio::buf::RingBuf::new(65535);
                    self.sock.recv_from(&mut rx_buf).ok().expect("sock.recv");
                    let mut client: &mut Client = self.client;
                    let buf: &[u8] = mio::buf::Buf::bytes(&rx_buf);
                    //XXX inspect why borrow checker error if I call self.client.dispatch_message
                    //TODO let out:Vec<Buf> = client.dispatch(); send_all(out);
                    client.dispatch_message(buf, &mut self.tx_buf);
                    //assert!(str::from_utf8(self.rx_buf.reader().bytes()) == self.msg);
                    //event_loop.shutdown();
                },
                _ => ()
            }
        }
        fn writable(&mut self, eloop: &mut mio::EventLoop<UdpHandler>, token: mio::Token) {
            //use mio::buf::Buf;
            match token {
                CLIENT => {
                    //info!("WRITABLE");
                    //TODO Option buf = client.get_any_data_to_send();
                    //Some => self.sock.send(buf),
                    //None => ()
                    match self.tx_buf.pop_back() {
                        Some(data) => {
                            //TODO parse and print message
                            println!("TX: {:?}", Message::from_buf(data.as_slice(),MessageDirection::FromClient));
                            let mut buf = mio::buf::SliceBuf::wrap(data.as_slice());
                            if let Err(e) = self.sock.send_to(&mut buf, &self.addr) {
                                println!("send_to error: {}", e);
                                eloop.shutdown();
                            }
                            self.start = false;
                        },
                        None => {}
                    }
                },
                _ => ()
            }
        }
    }
    //TODO every 5 seconds >>> Client::enqueue_to_send(beat());
    //let server_ip = get_host_addresses("game.salemthegame.com")[0];
    let hostname = "game.salemthegame.com";
    let host = {
        let mut ips = std::net::lookup_host(hostname).ok().expect("lookup_host");
        ips.next().expect("ip.next").ok().expect("ip.next.ok")
    };
    //FIXME let addr = mio::net::SockAddr::InetAddr(host.ip(), 1870);
    let any = str::FromStr::from_str("0.0.0.0:0").ok().expect("any.from_str");
    let sock = mio::udp::bind(&any).ok().expect("bind");
    //type ClientEventLoop = mio::EventLoop<usize>;
    println!("connect to {}", host.ip());
    //FIXME sock.connect(&addr);
    sock.set_reuseaddr(true).ok().expect("set_reuseaddr");

    //TODO return Result and match
    let mut client = Client::new(/*"game.salemthegame.com", 1871, 1870*/);
    //TODO FIXME get login/password from command line instead of storing them here
    match client.authorize("salvian", "простойпароль", host.ip(), 1871) {
        Ok(()) => {
            println!("success. cookie = [{}]", client.cookie.as_slice().to_hex());
        },
        Err(e) => {
            println!("authorize error: {:?}", e);
            return;
        }
    };

    
    let mut event_loop = mio::EventLoop::new().ok().expect("mio.loop.new");
    event_loop.register_opt(&sock, CLIENT, mio::Interest::readable() |
                                           mio::Interest::writable(),
                                           mio::PollOpt::level()).ok().expect("loop.register_opt");
    let mut handler = UdpHandler::new(sock, &mut client, std::net::SocketAddr::new(host.ip(),1870));
    handler.client.connect(&mut handler.tx_buf); //TODO return Result and match

    info!("run event loop");
    event_loop.run(&mut handler).ok().expect("Failed to run the event loop");

    //client.wait_for_end();

/* TODO
    Ok(Control::Dump) => {
        for o in objects.values() {
            let (x,y) = o.xy;
            let resid = o.resid;
            let resname = match resources.get(&o.resid) {
                Some(res) => { res.as_slice() },
                None      => { "null" },
            };
            client.control_tx.send(format!("({:7},{:7}) {:7} {}", x, y, resid, resname));
        }
    },
*/
}




















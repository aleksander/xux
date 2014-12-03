#![feature(macro_rules)]

extern crate openssl;
extern crate serialize;

use std::io::Writer;
use std::io::MemWriter;
use std::io::net::tcp::TcpStream;
use std::io::net::udp::UdpSocket;
use std::io::net::ip::Ipv4Addr;
use std::io::net::ip::SocketAddr;
use std::io::net::addrinfo::get_host_addresses;
use std::io::MemReader;
use std::io::timer;
use std::collections::hash_map::HashMap;
use std::str;
use std::time::Duration;
use serialize::hex::ToHex;
use openssl::crypto::hash::HashType;
use openssl::crypto::hash::hash;
use openssl::ssl::{SslMethod, SslContext, SslStream};
use std::vec::Vec;
use std::fmt::{Show, Formatter};
use std::io::net::pipe::UnixListener;
use std::io::{Listener, Acceptor};

macro_rules! tryio (
    ($fmt:expr $e:expr) => (
        match $e {
            Ok(e) => e,
            Err(e) => return Err(Error{source:$fmt, detail:e.detail})
        }
    )
)

struct Error {
    source: &'static str,
    detail: Option<String>,
}

fn sess (name: &str, cookie: &[u8]) -> Vec<u8> {
    let mut w = MemWriter::new();
    w.write_u8(0).unwrap(); // SESS
    w.write_le_u16(2).unwrap(); // unknown
    w.write("Salem".as_bytes()).unwrap(); // proto
    w.write_u8(0).unwrap();
    w.write_le_u16(34).unwrap(); // version
    w.write(name.as_bytes()).unwrap(); // login
    w.write_u8(0).unwrap();
    w.write_le_u16(32).unwrap(); // cookie length
    w.write(cookie).unwrap(); // cookie
    w.into_inner()
}

fn ack (seq: u16) -> Vec<u8> {
    let mut w = MemWriter::new();
    w.write_u8(2).unwrap(); //ACK
    w.write_le_u16(seq).unwrap();
    w.into_inner()
}

fn beat () -> Vec<u8> {
    let mut w = MemWriter::new();
    w.write_u8(3).unwrap(); //BEAT
    w.into_inner()
}

fn rel_wdgmsg_play (seq: u16, name: &str) -> Vec<u8> {
    let mut w = MemWriter::new();
    w.write_u8(1).unwrap(); // REL
    w.write_le_u16(seq).unwrap();// sequence
    w.write_u8(1).unwrap();// rel type WDGMSG
    w.write_le_u16(3).unwrap();// widget id
    w.write("play".as_bytes()).unwrap();// message name
    w.write_u8(0).unwrap();
    // args list
    w.write_u8(2).unwrap(); // list element type T_STR
    w.write(name.as_bytes()).unwrap(); // element
    w.write_u8(0).unwrap();
    w.into_inner()
}


struct Obj {
    resid : u16,
    xy : (i32,i32),
}

#[deriving(Show)]
struct NewWdg {
    id : u16,
    kind : String,
    parent : u16,
    pargs : Vec<MsgList>,
    cargs : Vec<MsgList>,
}
#[deriving(Show)]
struct WdgMsg {
    id : u16,
    name : String,
    args : Vec<MsgList>,
}
#[deriving(Show)]
struct DstWdg {
    id : u16,
}
#[deriving(Show)]
struct MapIv;
#[deriving(Show)]
struct GlobLob;
#[deriving(Show)]
struct Paginae;
#[deriving(Show)]
struct ResId {
    id : u16,
    name : String,
    ver : u16,
}
#[deriving(Show)]
struct Party;
#[deriving(Show)]
struct Sfx;
#[deriving(Show)]
struct Cattr;
#[deriving(Show)]
struct Music;
#[deriving(Show)]
struct Tiles;
#[deriving(Show)]
struct Buff;
#[deriving(Show)]
struct SessKey;

#[deriving(Show)]
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
    SESSKEY(SessKey),
    UNKNOWN( u8 ), //TODO remove this and use Oprion<Msg> or Result<Msg>
}

#[allow(non_camel_case_types)]
#[deriving(Show)]
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

fn read_sublist (r:&mut MemReader) /*TODO return Result instead*/ {
    let mut deep = 0u;
    loop {
        if r.eof() { return; }
        let t = r.read_u8().unwrap();
        match t {
        /*T_END    */  0  => { if deep == 0 { return; } else { deep -= 1; } },
        /*T_INT    */  1  => { r.read_le_i32().unwrap(); },
        /*T_STR    */  2  => { r.read_until(0).unwrap(); },
        /*T_COORD  */  3  => { r.read_le_i32().unwrap(); r.read_le_i32().unwrap(); },
        /*T_UINT8  */  4  => { r.read_u8().unwrap(); },
        /*T_UINT16 */  5  => { r.read_le_u16().unwrap(); },
        /*T_COLOR  */  6  => { r.read_u8().unwrap(); r.read_u8().unwrap(); r.read_u8().unwrap(); r.read_u8().unwrap(); },
        /*T_TTOL   */  8  => { deep += 1; },
        /*T_INT8   */  9  => { r.read_i8().unwrap(); },
        /*T_INT16  */  10 => { r.read_le_i16().unwrap(); },
        /*T_NIL    */  12 => { },
        /*T_BYTES  */  14 => {
            let len = r.read_u8().unwrap();
            if (len & 128) != 0 {
                let len = r.read_le_i32().unwrap(); /* WHY NOT u32 ??? */
                r.read_exact(len as uint).unwrap();
            } else {
                r.read_exact(len as uint).unwrap();
            }
        },
        /*T_FLOAT32*/  15 => { r.read_le_f32().unwrap(); },
        /*T_FLOAT64*/  16 => { r.read_le_f64().unwrap(); },
                       _  => { return; /*TODO return Error instead*/ },
        }
    }
}

fn read_list (r:&mut MemReader) -> Vec<MsgList> /*TODO return Result instead*/ {
    let mut list = Vec::new();
    loop {
        if r.eof() { return list; }
        let t = r.read_u8().unwrap();
        match t {
            /*T_END    */  0  => { return list; },
            /*T_INT    */  1  => {
                list.push(MsgList::tINT( r.read_le_i32().unwrap() ));
            },
            /*T_STR    */  2  => {
                list.push(MsgList::tSTR( String::from_utf8(r.read_until(0).unwrap()).unwrap() ));
            },
            /*T_COORD  */  3  => {
                list.push(MsgList::tCOORD( (r.read_le_i32().unwrap(),r.read_le_i32().unwrap()) ));
            },
            /*T_UINT8  */  4  => {
                list.push(MsgList::tUINT8( r.read_u8().unwrap() ));
            },
            /*T_UINT16 */  5  => {
                list.push(MsgList::tUINT16( r.read_le_u16().unwrap() ));
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
                list.push(MsgList::tINT16( r.read_le_i16().unwrap() ));
            },
            /*T_NIL    */  12 => {
                list.push(MsgList::tNIL);
            },
            /*T_BYTES  */  14 => {
                let len = r.read_u8().unwrap();
                if (len & 128) != 0 {
                    let len = r.read_le_i32().unwrap(); /* WHY NOT u32 ??? */
                    list.push(MsgList::tBYTES( r.read_exact(len as uint).unwrap() ));
                } else {
                    list.push(MsgList::tBYTES( r.read_exact(len as uint).unwrap() ));
                }
            },
            /*T_FLOAT32*/  15 => {
                list.push(MsgList::tFLOAT32( r.read_le_f32().unwrap() ));
            },
            /*T_FLOAT64*/  16 => {
                list.push(MsgList::tFLOAT64( r.read_le_f64().unwrap() ));
            },
            /*UNKNOWN*/    _  => {
                println!("    !!! UNKNOWN LIST ELEMENT !!!");
                return list; /*TODO return Error instead*/
            },
        }
    }
}

impl RelElem {
    fn from_buf (kind:u8, buf:&[u8]) -> RelElem {
        let mut r = MemReader::new(buf.to_vec());
        //XXX RemoteUI.java +53
        match kind {
            0  /*NEWWDG*/ => {
                let id = r.read_le_u16().unwrap();
                let kind = String::from_utf8(r.read_until(0).unwrap()).unwrap();
                let parent = r.read_le_u16().unwrap();
                let pargs = read_list(&mut r);
                let cargs = read_list(&mut r);
                RelElem::NEWWDG( NewWdg{ id:id, kind:kind, parent:parent, pargs:pargs, cargs:cargs } )
            },
            1  /*WDGMSG*/ => {
                let id = r.read_le_u16().unwrap();
                let name = String::from_utf8(r.read_until(0).unwrap()).unwrap();
                let args = read_list(&mut r);
                RelElem::WDGMSG( WdgMsg{ id:id, name:name, args:args } )
            },
            2  /*DSTWDG*/ => {
                let id = r.read_le_u16().unwrap();
                RelElem::DSTWDG( DstWdg{ id:id } )
            },
            3  /*MAPIV*/ => { RelElem::MAPIV(MapIv) },
            4  /*GLOBLOB*/ => { RelElem::GLOBLOB(GlobLob) },
            5  /*PAGINAE*/ => { RelElem::PAGINAE(Paginae) },
            6  /*RESID*/ => {
                let id = r.read_le_u16().unwrap();
                let name = String::from_utf8(r.read_until(0).unwrap()).unwrap();
                let ver = r.read_le_u16().unwrap();
                RelElem::RESID( ResId{ id:id, name:name, ver:ver } )
            },
            7  /*PARTY*/ => { RelElem::PARTY(Party) },
            8  /*SFX*/ => { RelElem::SFX(Sfx) },
            9  /*CATTR*/ => { RelElem::CATTR(Cattr) },
            10 /*MUSIC*/ => { RelElem::MUSIC(Music) },
            11 /*TILES*/ => { RelElem::TILES(Tiles) },
            12 /*BUFF*/ => { RelElem::BUFF(Buff) },
            13 /*SESSKEY*/ => { RelElem::SESSKEY(SessKey) },
            _ => {
                //println!("\x1b[31m  UNKNOWN {}\x1b[39;49m", rel_type);
                RelElem::UNKNOWN( kind )
            },
        }
    }
}

#[deriving(Show)]
enum SessError {
    OK,
    AUTH,
    BUSY,
    CONN,
    PVER,
    EXPR,
    UNKNOWN(u8), //TODO remove this and use Oprion<Msg> or Result<Msg>
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
#[deriving(Show)]
struct Sess {
    err : SessError,
}
struct Rel {
    seq : u16,
    rel : Vec<RelElem>
}
impl Show for Rel {
    fn fmt(&self, f : &mut Formatter) -> std::fmt::Result {
        write!(f, "REL seq={}", self.seq)
    }
}
#[deriving(Show)]
struct Ack {
    seq : u16,
}
#[deriving(Show)]
struct Beat;
#[deriving(Show)]
struct MapReq;
#[deriving(Show)]
struct MapData;
struct ObjData {
    obj : Vec<ObjDataElem>,
}
impl Show for ObjData {
    fn fmt(&self, f : &mut Formatter) -> std::fmt::Result {
        write!(f, "OBJDATA")
    }
}
#[deriving(Show)]
struct ObjDataElem {
    fl    : u8,
    id    : u32,
    frame : i32,
    prop  : Vec<ObjProp>,
}
#[deriving(Show)]
struct ObjAck;
#[deriving(Show)]
struct Close;

#[deriving(Show)]
//TODO replace with plain struct variants
enum Msg {
    SESS( Sess ),
    REL( Rel ),
    ACK( Ack ),
    BEAT( Beat ),
    MAPREQ( MapReq ),
    MAPDATA( MapData ),
    OBJDATA( ObjData ),
    OBJACK( ObjAck ),
    CLOSE( Close ),
    UNKNOWN( u8 ), //TODO remove this and use Oprion<Msg> or Result<Msg>
}

#[allow(non_camel_case_types)]
#[deriving(Show)]
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
    odFOLLOW(u32),
    odHOMING,
    odOVERLAY,
    odAUTH,
    odHEALTH,
    odBUDDY,
    odCMPPOSE,
    odCMPMOD,
    odCMPEQU,
    odICON,
}
impl ObjProp {
    fn from_buf (r:&mut MemReader) -> Option<ObjProp> /*TODO return Result<Option<ObjProp>>*/ {
        let t = r.read_u8().unwrap() as uint;
        match t {
            0   /*OD_REM*/ => {
                Some(ObjProp::odREM)
            },
            1   /*OD_MOVE*/ => {
                let xy = (r.read_le_i32().unwrap(), r.read_le_i32().unwrap());
                let ia = r.read_le_u16().unwrap();
                Some(ObjProp::odMOVE(xy,ia))
            },
            2   /*OD_RES*/ => {
                let mut resid = r.read_le_u16().unwrap();
                if (resid & 0x8000) != 0 {
                    resid &= !0x8000;
                    let sdt_len = r.read_u8().unwrap() as uint;
                    let _/*sdt*/ = r.read_exact(sdt_len).unwrap();
                }
                Some(ObjProp::odRES(resid))
            },
            3   /*OD_LINBEG*/ => {
                let s = (r.read_le_i32().unwrap(), r.read_le_i32().unwrap());
                let t = (r.read_le_i32().unwrap(), r.read_le_i32().unwrap());
                let c = r.read_le_i32().unwrap();
                Some(ObjProp::odLINBEG(s,t,c))
            },
            4   /*OD_LINSTEP*/ => {
                let l = r.read_le_i32().unwrap();
                Some(ObjProp::odLINSTEP(l))
            },
            5   /*OD_SPEECH*/ => {
                let zo = r.read_le_u16().unwrap();
                let text = String::from_utf8(r.read_until(0).unwrap()).unwrap();
                Some(ObjProp::odSPEECH(zo,text))
            },
            6   /*OD_COMPOSE*/ => {
                let resid = r.read_le_u16().unwrap();
                Some(ObjProp::odCOMPOSE(resid))
            },
            7   /*OD_DRAWOFF*/ => {
                let off = (r.read_le_i32().unwrap(), r.read_le_i32().unwrap());
                Some(ObjProp::odDRAWOFF(off))
            },
            8   /*OD_LUMIN*/ => {
                let off = (r.read_le_i32().unwrap(), r.read_le_i32().unwrap());
                let sz = r.read_le_u16().unwrap();
                let str_ = r.read_u8().unwrap();
                Some(ObjProp::odLUMIN(off,sz,str_))
            },
            9   /*OD_AVATAR*/ => {
                let mut layers = Vec::new();
                loop {
                    let layer = r.read_le_u16().unwrap();
                    if layer == 65535 {
                        break;
                    }
                    layers.push(layer);
                }
                Some(ObjProp::odAVATAR(layers))
            },
            10  /*OD_FOLLOW*/ => {
                let oid = r.read_le_u32().unwrap();
                if oid == 0xff_ff_ff_ff {
                    /*let xfres =*/ r.read_le_u16().unwrap();
                    /*let xfname =*/ String::from_utf8(r.read_until(0).unwrap()).unwrap();
                }
                Some(ObjProp::odFOLLOW(oid))
            },
            11  /*OD_HOMING*/ => {
                let oid = r.read_le_u32().unwrap();
                match oid {
                    0xff_ff_ff_ff => {},
                    0xff_ff_ff_fe => {
                        /*let tgtc =*/ (r.read_le_i32().unwrap(), r.read_le_i32().unwrap());
                        /*let v =*/ r.read_le_u16().unwrap();
                    },
                    _             => {
                        /*let tgtc =*/ (r.read_le_i32().unwrap(), r.read_le_i32().unwrap());
                        /*let v =*/ r.read_le_u16().unwrap();
                    }
                }
                Some(ObjProp::odHOMING)
            },
            12  /*OD_OVERLAY*/ => {
                /*let olid =*/ r.read_le_i32().unwrap();
                let resid = r.read_le_u16().unwrap();
                if (resid & 0x8000) != 0 {
                    let sdt_len = r.read_u8().unwrap() as uint;
                    /*let sdt =*/ r.read_exact(sdt_len).unwrap();
                }
                Some(ObjProp::odOVERLAY)
            },
            13  /*OD_AUTH*/   => {
                /* Removed */
                Some(ObjProp::odAUTH)
            },
            14  /*OD_HEALTH*/ => {
                /*let hp =*/ r.read_u8().unwrap();
                Some(ObjProp::odHEALTH)
            },
            15  /*OD_BUDDY*/ => {
                let name = String::from_utf8(r.read_until(0).unwrap()).unwrap();
                if name.len() > 0 {
                    /*let group =*/ r.read_u8().unwrap();
                    /*let btype =*/ r.read_u8().unwrap();
                }
                Some(ObjProp::odBUDDY)
            },
            16  /*OD_CMPPOSE*/ => {
                let pfl = r.read_u8().unwrap();
                /*let seq =*/ r.read_u8().unwrap();
                if (pfl & 2) != 0 {
                    loop {
                        let /*mut*/ resid = r.read_le_u16().unwrap();
                        if resid == 65535 { break; }
                        if (resid & 0x8000) != 0 {
                            /*resid &= !0x8000;*/
                            let sdt_len = r.read_u8().unwrap() as uint;
                            /*let sdt =*/ r.read_exact(sdt_len).unwrap();
                        }
                    }
                }
                if (pfl & 4) != 0 {
                    loop {
                        let /*mut*/ resid = r.read_le_u16().unwrap();
                        if resid == 65535 { break; }
                        if (resid & 0x8000) != 0 {
                            /*resid &= !0x8000;*/
                            let sdt_len = r.read_u8().unwrap() as uint;
                            /*let sdt =*/ r.read_exact(sdt_len).unwrap();
                        }
                    }
                    /*let ttime =*/ r.read_u8().unwrap();
                }
                Some(ObjProp::odCMPPOSE)
            },
            17  /*OD_CMPMOD*/ => {
                loop {
                    let modif = r.read_le_u16().unwrap();
                    if modif == 65535 { break; }
                    loop {
                        let resid = r.read_le_u16().unwrap();
                        if resid == 65535 { break; }
                    }
                }
                Some(ObjProp::odCMPMOD)
            },
            18  /*OD_CMPEQU*/ => {
                loop {
                    let h = r.read_u8().unwrap();
                    if h == 255 { break; }
                    /*let at =*/ String::from_utf8(r.read_until(0).unwrap()).unwrap();
                    /*let resid =*/ r.read_le_u16().unwrap();
                    if (h & 0x80) != 0 {
                        /*let x =*/ r.read_le_u16().unwrap();
                        /*let y =*/ r.read_le_u16().unwrap();
                        /*let z =*/ r.read_le_u16().unwrap();
                    }
                }
                Some(ObjProp::odCMPEQU)
            },
            19  /*OD_ICON*/ => {
                let resid = r.read_le_u16().unwrap();
                if resid != 65535 {
                    /*let ifl =*/ r.read_u8().unwrap();
                }
                Some(ObjProp::odICON)
            },
            255 /*OD_END*/ => {
                None
            },
            _   /*UNKNOWN*/ => {
                None /*TODO return error*/
            }
        }
    }
}

impl Msg {
    fn from_buf (buf:&[u8]) -> Msg {
        let mut r = MemReader::new(buf.to_vec());
        let mtype = r.read_u8().unwrap();
        let res = match mtype {
            0 /*SESS*/ => {
                Msg::SESS( Sess{ err : SessError::new(r.read_u8().unwrap()) } )
            },
            1 /*REL*/ => {
                let seq = r.read_le_u16().unwrap();
                let mut rel_vec = Vec::new();
                while !r.eof() {
                    let rel_buf;
                    let mut rel_type = r.read_u8().unwrap();
                    if (rel_type & 0x80) != 0 {
                        rel_type &= !0x80;
                        let rel_len = r.read_le_u16().unwrap();
                        rel_buf = r.read_exact(rel_len as uint).unwrap();
                    } else {
                        rel_buf = r.read_to_end().unwrap();
                    }
                    rel_vec.push(RelElem::from_buf(rel_type, rel_buf.as_slice()));
                }
                Msg::REL( Rel{ seq : seq, rel : rel_vec } )
            },
            2 /*ACK*/ => {
                Msg::ACK( Ack{ seq : r.read_le_u16().unwrap() } )
            },
            3 /*BEAT*/ => {
                Msg::BEAT(Beat)
            },
            4 /*MAPREQ*/ => {
                Msg::MAPREQ(MapReq)
            },
            5 /*MAPDATA*/ => {
                Msg::MAPDATA(MapData)
            },
            6 /*OBJDATA*/ => {
                let mut obj = Vec::new();
                while !r.eof() {
                    let fl = r.read_u8().unwrap();
                    let id = r.read_le_u32().unwrap();
                    let frame = r.read_le_i32().unwrap();
                    let mut prop = Vec::new();
                    loop {
                        match ObjProp::from_buf(&mut r) {
                            Some(p) => { prop.push(p) },
                            None => { break },
                        }
                    }
                    obj.push( ObjDataElem{ fl:fl, id:id, frame:frame, prop:prop } );
                }
                Msg::OBJDATA( ObjData{ obj : obj } )
            },
            7 /*OBJACK*/ => {
                Msg::OBJACK(ObjAck)
            },
            8 /*CLOSE*/ => {
                Msg::CLOSE(Close)
            },
            _ /*UNKNOWN*/ => {
                Msg::UNKNOWN(mtype)
            }
        };

        if !r.eof() {
            let remains = r.read_to_end().unwrap();
            println!("                       REMAINS {} bytes", remains.len());
        }

        res
    }
}

struct Client {
    user: &'static str,
    //pass: &'static str,
    cookie: Vec<u8>,
    //host: &'static str,
    //auth_port: u16,
    //port: u16,
    auth_addr: SocketAddr,
    //game_addr: SocketAddr,
    //any_addr: SocketAddr,
    //udp_rx: UdpSocket,
    //udp_tx: UdpSocket,
    main_to_sender: Sender<Vec<u8>>,     //TODO type OutputBuffer = Vec<u8>
    //sender_from_any: Receiver<Vec<u8>>,  //TODO type OutputBuffer = Vec<u8>
    //receiver_to_sender: Sender<Vec<u8>>, //TODO type OutputBuffer = Vec<u8>
    //beater_to_sender: Sender<Vec<u8>>,
    //receiver_to_main: Sender<()>,
    main_from_any: Receiver<()>,
    //receiver_to_beater: Sender<()>,
    //beater_from_any: Receiver<()>,
    //receiver_to_viewer: Sender<(u32,Obj)>,
    //viewer_from_any: Receiver<(u32,Obj)>,
    //objects: HashMap<u32,Obj>,
    //resources: HashMap<u16,String>,
    //widgets: HashMap<uint,String>,
}

impl Client {
    fn new (host: &'static str, auth_port: u16, port: u16) -> Client {
        let host_ip = get_host_addresses(host).unwrap()[0];
        let any_addr = SocketAddr {ip: Ipv4Addr(0,0,0,0), port: 0};
        let sock = UdpSocket::bind(any_addr).unwrap();

        let (tx1,rx1) = channel(); // any -> sender (packet to send)
        let (tx2,rx2) = channel(); // any -> beater (wakeup signal)
        let (tx3,rx3) = channel(); // any -> viewer (objects data)
        let (tx4,rx4) = channel(); // any -> main   (exit signal)

        // control socket reader
        spawn(proc() {
            let path = Path::new("/tmp/socket");
            let stream = UnixListener::bind(&path);
            for mut client in stream.listen().incoming() {
                loop {
                    match client.read_byte() {
                        Ok(b) => { println!("reader: read: {}", b); },
                        Err(e) => { println!("reader: error: {}", e); break; },
                    }
                }
            }
        });

        // sender
        let sender_from_any = rx1;
        let mut udp_tx = sock.clone();
        let host_addr = SocketAddr {ip: host_ip, port: port};
        spawn(proc() {
            loop {
                let buf: Vec<u8> = sender_from_any.recv();
                println!("sender: send {} bytes", buf.len());
                udp_tx.send_to(buf.as_slice(), host_addr).unwrap();
            }
        });

        // beater
        let beater_from_any = rx2;
        let beater_to_sender = tx1.clone();
        spawn(proc() {
            beater_from_any.recv();
            //send BEAT every 5 sec
            loop {
                beater_to_sender.send(beat());
                //FIXME wait on beater_from_any for 5 sec then exit or send(beat)
                timer::sleep(Duration::seconds(5));
            }
        });

        // viewer
        let viewer_from_any = rx3;
        let mut objects = HashMap::new();
        spawn(proc() {
            loop {
                //TODO while(try_recv)
                let objdata:ObjData = viewer_from_any.recv();
                for o in objdata.obj.iter() {
                    if !objects.contains_key(&o.id) {
                        objects.insert(o.id, Obj{resid:0, xy:(0,0)});
                    }
                    match objects.get_mut(&o.id) {
                        Some(obj) => {
                            //TODO check for o.frame vs obj.frame
                            for prop in o.prop.iter() {
                                match *prop {
                                    ObjProp::odREM => { /*FIXME objects.remove(&o.id); break;*/ },
                                    ObjProp::odMOVE(xy,_) => { obj.xy = xy; },
                                    ObjProp::odRES(resid) => { obj.resid = resid; },
                                    _ => {},
                                }
                            }
                        },
                        None => { /*thats cant be*/ },
                    };
                }
            }
        });

        // receiver
        let mut udp_rx = sock.clone();
        let receiver_to_main = tx4.clone();
        let receiver_to_beater = tx2.clone();
        let receiver_to_sender = tx1.clone();
        let receiver_to_viewer = tx3.clone();
        spawn(proc() {
            let mut buf = [0u8, ..65535];
            let mut charlist = Vec::new();
            let mut widgets = HashMap::new();
            let mut resources = HashMap::new();
            widgets.insert(0, "root".to_string());
            loop {
                let (len,addr) = udp_rx.recv_from(buf.as_mut_slice()).ok().expect("failed to recv_from");
                //FIXME connect the socket
                if addr != host_addr {
                    println!("wrong host: {}", addr);
                    continue;
                }
                let msg = Msg::from_buf(buf.slice_to(len));
                println!("receiver: {}", msg);
                match msg {
                    Msg::SESS(sess) => {
                        match sess.err {
                            SessError::OK => {},
                            _ => {
                                receiver_to_main.send(());
                                // ??? should we send CLOSE too ???
                                break;
                            }
                        }
                        receiver_to_beater.send(());
                    },
                    Msg::REL( rel ) => {
                        //TODO do not process duplicates, but ACK only
                        //XXX are we handle seq right in the case of overflow ???
                        receiver_to_sender.send(ack(rel.seq + ((rel.rel.len() as u16) - 1)));
                        for r in rel.rel.iter() {
                            println!("    {}", r);
                            match *r {
                                RelElem::NEWWDG(ref wdg) => {
                                    widgets.insert(wdg.id as uint, wdg.kind.clone()/*FIXME String -> &str*/);
                                },
                                RelElem::WDGMSG(ref msg) => {
                                    //TODO match against widget.type and message.type
                                    match widgets.get(&(msg.id as uint)) {
                                        None => {},
                                        Some(c) => {
                                            if (c.as_slice() == "charlist\0") && (msg.name.as_slice() == "add\0") {
                                                match msg.args[0] {
                                                    MsgList::tSTR(ref char_name) => {
                                                        println!("    add char '{}'", char_name);
                                                        charlist.push(char_name.clone()/*FIXME rewrite without cloning*/);
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
                                    resources.insert(res.id, res.name.clone()/*FIXME String -> &str*/);
                                },
                                RelElem::PARTY(_) => {},
                                RelElem::SFX(_) => {},
                                RelElem::CATTR(_) => {},
                                RelElem::MUSIC(_) => {},
                                RelElem::TILES(_) => {},
                                RelElem::BUFF(_) => {},
                                RelElem::SESSKEY(_) => {},
                                RelElem::UNKNOWN(_) => {},
                            }
                        }
                    },
                    Msg::ACK(_) => {},
                    Msg::BEAT(_) => {
                        println!("     !!! client must not receive BEAT !!!");
                    },
                    Msg::MAPREQ(_) => {
                        println!("     !!! client must not receive MAPREQ !!!");
                    },
                    Msg::MAPDATA(_) => {},
                    Msg::OBJDATA( objdata ) => {
                        let mut w = MemWriter::new();
                        w.write_u8(7).unwrap(); //OBJACK writer
                        for o in objdata.obj.iter() {
                            w.write_le_u32(o.id).unwrap();
                            w.write_le_i32(o.frame).unwrap();
                        }
                        //TODO receiver_to_sender.send(objdata.to_buf());
                        receiver_to_sender.send(w.into_inner()); // send OBJACKs
                        for o in objdata.obj.iter() {
                            println!("    {}", o);
                        }
                        receiver_to_viewer.send(objdata);
                    },
                    Msg::OBJACK(_) => {},
                    Msg::CLOSE(_) => {
                        receiver_to_main.send(());
                        // ??? should we send CLOSE too ???
                        break;
                    },
                    Msg::UNKNOWN(_) => {
                        println!("     !!! UNKNOWN !!!");
                    },
                }

                //TODO send REL until reply
                if charlist.len() > 0 {
                    println!("send play '{}'", charlist[0]);
                    receiver_to_sender.send(rel_wdgmsg_play(0, charlist[0].as_slice()));
                    charlist.clear();
                }
            }
        });

        Client {
            user: "",
            cookie: Vec::new(),
            //host: host,
            //auth_port: auth_port,
            //port: port,
            auth_addr: SocketAddr {ip: host_ip, port: auth_port},
            //game_addr: SocketAddr {ip: host_ip, port: port},
            //udp_rx: sock.clone(),
            //udp_tx: sock.clone(),

            main_to_sender: tx1.clone(),
            //sender_from_any: rx1,
            //receiver_to_sender: tx1.clone(),
            //beater_to_sender: tx1.clone(),
            //receiver_to_main: tx2.clone(),
            main_from_any: rx4,
            //receiver_to_beater: tx2.clone(),
            //beater_from_any: rx2,
            //receiver_to_viewer: tx3.clone(),
            //viewer_from_any: rx3,

            //objects: HashMap::new(),
            //resources: HashMap::new(),
            //widgets: HashMap::new(),
        }
    }

    fn authorize (&mut self, user: &'static str, pass: &str) -> Result<(), Error> {
        self.user = user;
        //self.pass = pass;
        println!("authorize {} @ {}", user, self.auth_addr);
        //TODO add method connect(SocketAddr) to TcpStream
        let stream = tryio!("tcp.connect" TcpStream::connect(self.auth_addr));
        let mut stream = SslStream::new(&SslContext::new(SslMethod::Sslv23).unwrap(), stream).unwrap();

        // send 'pw' command
        // TODO form buffer and send all with one call
        // TODO tryio!(stream.write(Msg::pw(params...)));
        stream.write_be_u16((3+user.as_bytes().len()+1+32) as u16).unwrap();
        stream.write("pw".as_bytes()).unwrap();
        stream.write_u8(0).unwrap();
        stream.write(user.as_bytes()).unwrap();
        stream.write_u8(0).unwrap();
        let pass_hash = hash(HashType::SHA256, pass.as_bytes());
        assert!(pass_hash.len() == 32);
        stream.write(pass_hash.as_slice()).unwrap();
        stream.flush().unwrap();
        let length = stream.read_be_u16().ok().expect("read error");
        let msg = stream.read_exact(length as uint).ok().expect("read error");
        println!("msg='{}'", str::from_utf8(msg.as_slice()).unwrap());
        //println!("msg='{}'", msg.as_slice().to_hex());
        if msg.len() < "ok\0\0".len() {
            return Err(Error{source:"'pw' command unexpected answer", detail:Some(String::from_utf8(msg).unwrap())});
        }

        // send 'cookie' command
        if (msg[0] == ('o' as u8)) && (msg[1] == ('k' as u8)) {
            // TODO form buffer and send all with one call
            // TODO tryio!(stream.write(Msg::cookie(params...)));
            stream.write_be_u16(("cookie".as_bytes().len()+1) as u16).unwrap();
            stream.write("cookie".as_bytes()).unwrap();
            stream.write_u8(0u8).unwrap();
            stream.flush().unwrap();
            let length = stream.read_be_u16().ok().expect("read error");
            let msg = stream.read_exact(length as uint).ok().expect("read error");
            //println!("msg='{}'", str::from_utf8(msg.as_slice()).unwrap());
            println!("msg='{}'", msg.as_slice().to_hex());
            //TODO check cookie length
            self.cookie = msg.slice_from(3).to_vec();
            return Ok(());
        }
        return Err(Error{source:"'cookie' command unexpected answer", detail:Some(String::from_utf8(msg).unwrap())});
    }

    fn connect (&self) {
        //TODO send SESS until reply
        //TODO get username from server responce, not from auth username
        self.main_to_sender.send(sess(self.user.as_slice(), self.cookie.as_slice()));
    }

    fn wait_for_end (&self) {
        self.main_from_any.recv();
    }
}



fn main() {
    //TODO handle keyboard interrupt

    let mut client = Client::new("game.salemthegame.com", 1871, 1870); //TODO return Result and match

    match client.authorize("salvian", "простойпароль") {
        Ok(()) => { println!("success. cookie = [{}]", client.cookie.as_slice().to_hex()); },
        Err(e) => { println!("error. {}: {}", e.source, e.detail.unwrap()); return; }
    };

    client.connect(); //TODO return Result and match
    client.wait_for_end();
}




















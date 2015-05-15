use std::vec::Vec;
use std::fmt::Debug;
use std::fmt::Formatter;
use std::io::Cursor;
use std::io::Read;
use std::io::BufRead;
use std::io::Write;

extern crate byteorder;
use self::byteorder::{LittleEndian, BigEndian, ReadBytesExt, WriteBytesExt};
#[allow(non_camel_case_types)]
type le = LittleEndian;
#[allow(non_camel_case_types)]
type be = BigEndian;


//TODO move to salem::error mod
#[derive(Debug)]
pub struct Error {
    pub source: &'static str,
    pub detail: Option<String>,
}

impl From<byteorder::Error> for Error {
    fn from (_:byteorder::Error) -> Error { Error {source:"TODO: ByteOrder error", detail:None} }
}

impl From<::std::io::Error> for Error {
    fn from (_: ::std::io::Error) -> Error { Error {source:"TODO: Io error", detail:None} }
}

impl From<::std::string::FromUtf8Error> for Error {
    fn from (_: ::std::string::FromUtf8Error) -> Error { Error {source:"TODO: FromUtf8 error", detail:None} }
}

#[derive(Debug)]
pub struct NewWdg {
    pub id : u16,
    pub kind : String,
    pub parent : u16,
    pub pargs : Vec<MsgList>,
    pub cargs : Vec<MsgList>,
}
#[derive(Debug)]
pub struct WdgMsg {
    pub id : u16,
    pub name : String,
    pub args : Vec<MsgList>,
}
#[derive(Debug)]
pub struct DstWdg {
    pub id : u16,
}
#[derive(Debug)]
pub struct MapIv;
#[derive(Debug)]
pub struct GlobLob;
#[derive(Debug)]
pub struct Paginae;
#[derive(Debug)]
pub struct ResId {
    pub id : u16,
    pub name : String,
    pub ver : u16,
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
    pub tiles : Vec<TilesElem>
}
impl Debug for Tiles {
    fn fmt(&self, f : &mut Formatter) -> ::std::fmt::Result {
        try!(writeln!(f, ""));
        for tile in self.tiles.iter() {
            try!(writeln!(f, "      {:?}", tile));
        }
        Ok(())
    }
}
#[derive(Debug)]
pub struct TilesElem {
    pub id : u8,
    pub name : String,
    pub ver : u16,
}
#[derive(Debug)]
pub struct Buff;
#[derive(Debug)]
pub struct SessKey;

#[derive(Debug)]
//TODO replace with plain struct variants
pub enum RelElem {
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
pub enum MsgList {
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

//TODO FIXME merge with read_list function
pub fn read_sublist (r : &mut ::std::io::Cursor<&[u8]> /*buf : &[u8]*/) {
    let mut deep = 0;
    loop {
        let t = match r.read_u8() {
            Ok(b) => {b}
            Err(_) => {return;}
        };
        match t {
            /*T_END    */  0  => { if deep == 0 { return; } else { deep -= 1; } },
            /*T_INT    */  1  => { r.read_i32::<le>().unwrap(); },
            /*T_STR    */  2  => { let mut tmp = Vec::new(); r.read_until(0, &mut tmp).unwrap(); tmp.pop(); },
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
                    let len = r.read_i32::<le>().unwrap();
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
                           _  => { return; },
        }
    }
}

pub fn write_list (list:&[MsgList]) -> Result<Vec<u8>,Error> {
    let mut w = vec![];
    for l in list {
        let tmp = l;
        match *tmp {
            MsgList::tINT(i) => {
                try!(w.write_u8(1));
                try!(w.write_i32::<le>(i));
            },
            MsgList::tSTR(ref s) => {
                try!(w.write_u8(2));
                try!(w.write(s.as_bytes()));
            },
            MsgList::tCOORD((x,y)) => {
                try!(w.write_u8(3));
                try!(w.write_i32::<le>(x));
                try!(w.write_i32::<le>(y));
            },
            MsgList::tUINT8(u) => {
                try!(w.write_u8(4));
                try!(w.write_u8(u));
            },
            MsgList::tUINT16(u) => {
                try!(w.write_u8(5));
                try!(w.write_u16::<le>(u));
            },
            MsgList::tCOLOR((r,g,b,a)) => {
                try!(w.write_u8(6));
                try!(w.write_u8(r));
                try!(w.write_u8(g));
                try!(w.write_u8(b));
                try!(w.write_u8(a));
            },
            MsgList::tTTOL => {
                return Err(Error{source:"write_list is NOT implemented for tTTOL",detail:None});
            },
            MsgList::tINT8(i) => {
                try!(w.write_u8(9));
                try!(w.write_i8(i));
            },
            MsgList::tINT16(i) => {
                try!(w.write_u8(10));
                try!(w.write_i16::<le>(i));
            },
            MsgList::tNIL => {
                try!(w.write_u8(12));
            },
            MsgList::tBYTES(_) => {
                return Err(Error{source:"write_list is NOT implemented for tBYTES",detail:None});
            },
            MsgList::tFLOAT32(f) => {
                try!(w.write_u8(15));
                try!(w.write_f32::<le>(f));
            },
            MsgList::tFLOAT64(f) => {
                try!(w.write_u8(16));
                try!(w.write_f64::<le>(f));
            },
        }
    }
    try!(w.write_u8(0)); /* T_END */
    Ok(w)
}

pub fn read_list (r : &mut Cursor<&[u8]>) -> Vec<MsgList> /*TODO return Result instead*/ {
    let mut list = Vec::new();
    loop {
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
                tmp.pop();
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
                    let len = r.read_i32::<le>().unwrap();
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
    pub fn from_buf (kind:u8, buf:&[u8]) -> Result<RelElem,Error> {
        let mut r = Cursor::new(buf);
        //XXX RemoteUI.java +53
        match kind {
            0  /*NEWWDG*/  => {
                let id = try!(r.read_u16::<le>());
                let kind = {
                    let mut tmp = Vec::new();
                    r.read_until(0, &mut tmp).unwrap();
                    tmp.pop();
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
                    tmp.pop();
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
                    tmp.pop();
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
                        tmp.pop();
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

    pub fn to_buf (&self, last:bool) -> Result<Vec<u8>,Error> {
        let mut w = vec![];
        match *self {
            RelElem::WDGMSG(ref msg) => {
                let mut tmp = vec![];
                try!(tmp.write_u16::<le>(msg.id)); // widget ID
                try!(tmp.write(msg.name.as_bytes())); // message name
                try!(tmp.write_u8(0)); // \0
                let args_buf = try!(write_list(&msg.args));
                try!(tmp.write(&args_buf));
                if last {
                    try!(w.write_u8(1)); // type WDGMSG
                } else {
                    try!(w.write_u8(1 & 0x80)); // type WDGMSG & more rels attached bit
                    try!(w.write_u16::<le>(tmp.len() as u16)); // rel length
                }
                try!(w.write(&tmp));

                Ok(w)
            }
            _ => {Err(Error{source:"RelElem.to_buf is not implemented for that elem type",detail:None})}
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
    UNKNOWN(u8)
}
impl SessError {
    pub fn new(t:u8) -> SessError {
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
pub struct sSess {
    pub err : SessError,
}
#[allow(non_camel_case_types)]
#[derive(Debug)]
pub struct cSess {
    pub login : String,
    pub cookie : Vec<u8>
}
pub struct Rel {
    pub seq : u16,
    pub rel : Vec<RelElem>
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
    fn fmt(&self, f : &mut Formatter) -> ::std::fmt::Result {
        try!(writeln!(f, "REL seq={}", self.seq));
        for r in self.rel.iter() {
            try!(writeln!(f, "      {:?}", r));
        }
        Ok(())
    }
}
#[derive(Debug)]
pub struct Ack {
    pub seq : u16,
}
#[allow(dead_code)]
#[derive(Debug)]
pub struct Beat;
#[derive(Debug)]
pub struct MapReq {
    pub x : i32,
    pub y : i32,
}
pub struct MapData {
    pub pktid : i32,
    pub off   : u16,
    pub len   : u16,
    pub buf   : Vec<u8>,
}
impl Debug for MapData {
    fn fmt(&self, f : &mut Formatter) -> ::std::fmt::Result {
        write!(f, "MAPDATA pktid:{} offset:{} len:{} buf:[..{}]", self.pktid, self.off, self.len, self.buf.len())
    }
}
pub struct ObjData {
    pub obj : Vec<ObjDataElem>,
}
impl Debug for ObjData {
    fn fmt(&self, f : &mut Formatter) -> ::std::fmt::Result {
        try!(writeln!(f, "OBJDATA"));
        for o in self.obj.iter() {
            try!(writeln!(f, "      {:?}", o));
        }
        Ok(())
    }
}
#[derive(Debug)]
pub struct ObjDataElem {
    pub fl    : u8,
    pub id    : u32,
    pub frame : i32,
    pub prop  : Vec<ObjProp>,
}
#[derive(Debug)]
pub struct ObjAck {
    pub obj : Vec<ObjAckElem>,
}
impl ObjAck {
    pub fn new (objdata: &ObjData) -> ObjAck {
        let mut objack = ObjAck{ obj : Vec::new() };
        for o in objdata.obj.iter() {
            objack.obj.push(ObjAckElem{ id : o.id, frame : o.frame});
        }
        objack
    }
}
#[derive(Debug)]
pub struct ObjAckElem {
    pub id : u32,
    pub frame : i32,
}
#[derive(Debug)]
pub struct Close;

#[allow(non_camel_case_types)]
#[derive(Debug)]
//TODO replace with plain struct variants
pub enum Message {
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
pub enum ObjProp {
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
pub enum odFOLLOW {
    Stop,
    To(u32,u16,String),
}
#[allow(non_camel_case_types)]
#[derive(Debug)]
pub enum odHOMING {
    New((i32,i32),u16),
    Change((i32,i32),u16),
    Delete,
}
#[allow(non_camel_case_types)]
#[derive(Debug)]
pub enum odBUDDY {
    Update(String,u8,u8),
    Delete,
}
#[allow(non_camel_case_types)]
#[derive(Debug)]
pub enum odICON {
    Set(u16),
    Del,
}

impl ObjProp {
    pub fn from_buf (r : &mut Cursor<&[u8]>) -> Result<Option<ObjProp>,Error> {
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
                    let /*sdt*/ _ = {
                        let mut tmp = vec![0; sdt_len as usize];
                        let len = r.read(&mut tmp).unwrap();
                        assert_eq!(len, sdt_len as usize);
                        tmp
                    };
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
                    tmp.pop();
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
                        tmp.pop();
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
                let /*olid*/ _ = try!(r.read_i32::<le>());
                let resid = try!(r.read_u16::<le>());
                if resid != 65535 {
                    if (resid & 0x8000) != 0 {
                        let sdt_len = try!(r.read_u8()) as usize;
                        let /*sdt*/ _ = {
                            let mut tmp = vec![0; sdt_len as usize];
                            let len = r.read(&mut tmp).unwrap();
                            assert_eq!(len, sdt_len as usize);
                            tmp
                        };
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
                    tmp.pop();
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
                let /*seq*/ _ = try!(r.read_u8());
                if (pfl & 2) != 0 {
                    loop {
                        let resid = try!(r.read_u16::<le>());
                        if resid == 65535 { break; }
                        if (resid & 0x8000) != 0 {
                            //resid &= !0x8000;
                            let sdt_len = try!(r.read_u8()) as usize;
                            let /*sdt*/ _ = {
                                let mut tmp = vec![0; sdt_len as usize];
                                let len = r.read(&mut tmp).unwrap();
                                assert_eq!(len, sdt_len as usize);
                                tmp
                            };
                        }
                    }
                }
                if (pfl & 4) != 0 {
                    loop {
                        let resid = try!(r.read_u16::<le>());
                        if resid == 65535 { break; }
                        if (resid & 0x8000) != 0 {
                            //resid &= !0x8000;
                            let sdt_len = try!(r.read_u8()) as usize;
                            let /*sdt*/ _ = {
                                let mut tmp = vec![0; sdt_len as usize];
                                let len = r.read(&mut tmp).unwrap();
                                assert_eq!(len, sdt_len as usize);
                                tmp
                            };
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
                        tmp.pop();
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

pub enum MessageDirection {
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
    pub fn from_buf (buf : &[u8], dir : MessageDirection) -> Result<(Message,Option<Vec<u8>>),Error> {
        let mut r = Cursor::new(buf);
        let mtype = try!(r.read_u8());
        let res = match mtype {
            0 /*SESS*/ => {
                //TODO ??? Ok(Message::sess(err))
                //     impl Message { fn sess (err: u8) -> Message::SESS { ... } }
                match dir {
                    MessageDirection::FromClient => {
                        let /*unknown*/ _ = try!(r.read_u16::<le>());
                        let /*proto*/ _ = {
                            let mut tmp = Vec::new();
                            try!(r.read_until(0, &mut tmp));
                            tmp.pop();
                            tmp
                        };
                        let /*version*/ _ = try!(r.read_u16::<le>());
                        let login = {
                            let mut tmp = Vec::new();
                            try!(r.read_until(0, &mut tmp));
                            tmp.pop();
                            tmp
                        };
                        let cookie_len = try!(r.read_u16::<le>());
                        let cookie = {
                            let mut tmp = vec![0; cookie_len as usize];
                            let len = try!(r.read(&mut tmp));
                            assert_eq!(len, cookie_len as usize);
                            tmp
                        };
                        Ok( Message::C_SESS( cSess{ login : try!(String::from_utf8(login)), cookie : cookie } ) )
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
                        let b = r.read(&mut tmp).unwrap();
                        assert_eq!(b, rel_len as usize);
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

        let mut tmp = Vec::new();
        try!(r.read_to_end(&mut tmp));
        let remains = if tmp.is_empty() { None } else { Some(tmp) };

        match res {
            Ok(msg) => {Ok((msg,remains))}
            Err(e) => {Err(e)}
        }
    }

    pub fn to_buf (self) -> Result<Vec<u8>,Error> {
        let mut w = vec![];
        match self {
            // !!! this is client session message, not server !!!
            Message::C_SESS(sess) => /*(name: &str, cookie: &[u8]) -> Vec<u8>*/ {
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
            Message::REL(rel) => /* rel_wdgmsg_play (seq: u16, name: &str) -> Vec<u8> */ {
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
            Message::MAPREQ(mapreq) => /* mapreq (x:i32, y:i32) -> Vec<u8> */ {
                try!(w.write_u8(4)); // MAPREQ
                try!(w.write_i32::<le>(mapreq.x)); // x
                try!(w.write_i32::<le>(mapreq.y)); // y
                Ok(w)
            }
            Message::OBJACK(objack) => {
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


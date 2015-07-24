//use std::net::IpAddr;
//use std::net::Ipv4Addr;
use std::collections::hash_map::HashMap;
//use std::collections::hash_set::HashSet;
use std::collections::LinkedList;
//use std::net::TcpStream;
//use std::net::SocketAddr;

//extern crate openssl;
//use self::openssl::crypto::hash::Type;
//use self::openssl::crypto::hash::hash;
//use self::openssl::ssl::{SslMethod, SslContext, SslStream};

use std::vec::Vec;
use std::io::Cursor;
use std::io::Read;
use std::io::BufRead;
//use std::io::Write;
//use std::str;
use std::u16;

extern crate byteorder;
use self::byteorder::LittleEndian;
use self::byteorder::BigEndian;
use self::byteorder::ReadBytesExt;
//use self::byteorder::WriteBytesExt;
#[allow(non_camel_case_types)]
type le = LittleEndian;
#[allow(non_camel_case_types)]
type be = BigEndian;

use message::MapData;
use message::Rel;
use message::Message;
use message::MessageDirection;
use message::Error;
use message::SessError;
use message::ObjProp;
use message::MsgList;
use message::Ack;
use message::ObjAck;
use message::RelElem;
use message::NewWdg;
use message::WdgMsg;
use message::cSess;
use message::MapReq;

//extern crate rustc_serialize;
//use self::rustc_serialize::hex::ToHex;

extern crate flate2;
//use std::io::prelude::*;
use self::flate2::read::ZlibDecoder;

pub struct Obj {
    pub id : u32,
    pub resid : u16,
    pub frame : i32,
    pub x : i32, //TODO unify coords
    pub y : i32, //TODO unify coords
    pub movement : Option<Movement>,
}

#[derive(Clone,Copy)]
pub struct Movement {
    pub from: (i32,i32), //TODO unify coords
    pub to: (i32,i32), //TODO unify coords
    pub steps: i32,
    pub step: i32,
}

#[derive(Clone)]
pub struct Timeout {
    pub ms  : u64,
    pub seq : usize,
}

#[derive(Clone)]
pub struct EnqueuedBuffer {
    pub buf : Vec<u8>,
    pub timeout : Option<Timeout>,
}

pub struct Widget {
    pub id : u16,
    pub typ : String,
    pub parent : u16,
    pub name : Option<String>,
}

pub struct Hero {
    pub name: Option<String>,
    pub obj: Option<u32>,
    pub weight: Option<u16>,
    pub tmexp: Option<i32>,
    pub hearthfire: Option<(i32,i32)>, //TODO unify coords
    pub inventory: HashMap<(i32,i32),u16>, //TODO unify coords
    pub start_xy: Option<(i32,i32)>,
}

pub struct MapPieces {
    total_len: u16,
    pieces: HashMap<u16,Vec<u8>>,
}

pub struct Surface {
    pub x: i32,
    pub y: i32,
    pub name: String,
    pub id: i64,
    pub tiles: Vec<u8>,
    pub z: Vec<i16>,
    //pub ol: Vec<u8>,
}

pub type PacketId = i32;

pub struct Map {
    pub partial: HashMap<PacketId,MapPieces>, //TODO somehow clean up from old pieces (periodically or whatever)
    pub grids: HashMap<(i32,i32),Surface>, //TODO unify coords
}

impl Map {
    fn append (&mut self, mapdata: MapData) {
        let map = self.partial.entry(mapdata.pktid).or_insert(MapPieces{total_len:mapdata.len,pieces:HashMap::new()});
        map.pieces.insert(mapdata.off, mapdata.buf);
    }

    fn complete (&self, pktid: i32) -> bool {
        let map = match self.partial.get(&pktid) {
            Some(m) => { m }
            None => { return false; }
        };
        let mut len = 0u16;
        loop {
            match map.pieces.get(&len) {
                Some(buf) => {
                    len += buf.len() as u16;
                    if len == map.total_len { return true; }
                }
                None => { return false; }
            }
        }
    }

    fn assemble (&mut self, pktid: i32) -> Vec<u8> /*TODO return Result*/ {
        let map = match self.partial.remove(&pktid) {
            Some(map) => { map }
            None => { return Vec::new(); }
        };
        let mut len = 0;
        let mut buf: Vec<u8> = Vec::new();
        loop {
            match map.pieces.get(&len) {
                Some(b) => {
                    buf.extend(b);
                    len += b.len() as u16;
                    if len == map.total_len {
                        break;
                    }
                }
                None => { break; }
            }
        }
        if buf.len() as u16 != map.total_len {
            println!("ERROR: buf.len() as u16 != map.total_len");
            //return Err(Error{source:"buf.len() as u16 != map.total_len",detail:None});
        }
        buf
    }

    //XXX ??? move to message ?
    fn from_buf (&self, buf: Vec<u8>) -> Surface {
        let mut r = Cursor::new(buf);
        let x = r.read_i32::<le>().unwrap();
        let y = r.read_i32::<le>().unwrap();
        let mmname = {
            let mut tmp = Vec::new();
            r.read_until(0, &mut tmp).unwrap();
            tmp.pop();
            String::from_utf8(tmp).unwrap()
        };
        //let mut pfl = vec![0; 256];
        loop {
            let pidx = r.read_u8().unwrap();
            if pidx == 255 { break; }
            /*pfl[pidx as usize]*/let _ = r.read_u8().unwrap();
        }
        let mut decoder = ZlibDecoder::new(r);
        let mut unzipped = Vec::new();
        let /*unzipped_len*/ _ = decoder.read_to_end(&mut unzipped).unwrap();
        //TODO check unzipped_len
        let mut r = Cursor::new(unzipped);
        let id = r.read_i64::<le>().unwrap();
        let mut tiles = Vec::with_capacity(100*100);
        for _ in 0..100*100 {
            tiles.push(r.read_u8().unwrap());
        }
        let mut z = Vec::with_capacity(100*100);
        for _ in 0..100*100 {
            z.push(r.read_i16::<le>().unwrap());
        }
        /*
        let mut ol = vec![0; 100*100];
        loop {
            let pidx = r.read_u8().unwrap();
            if pidx == 255 { break; }
            let fl = pfl[pidx as usize];
            let typ = r.read_u8().unwrap();
            let (x1,y1) = (r.read_u8().unwrap() as usize, r.read_u8().unwrap() as usize);
            let (x2,y2) = (r.read_u8().unwrap() as usize, r.read_u8().unwrap() as usize);
            println!("#### {} ({},{}) - ({},{})", typ, x1, y1, x2, y2);
            let oli = match typ {
                0 => if (fl & 1) == 1 { 2 } else { 1 },
                1 => if (fl & 1) == 1 { 8 } else { 4 },
                2 => 16,
                _ => { println!("ERROR: unknown plot type {}", typ); break; }
            };
            for y in y1..y2+1 {
                for x in x1..x2+1 {
                    ol[x+y*100] |= oli;
                }
            }
        }
        */
        Surface{x:x,y:y,name:mmname,id:id,tiles:tiles,z:z/*,ol:ol*/}
    }

}

pub enum Event {
    Grid(i32,i32,Vec<u8>,Vec<i16>),
}

pub struct State {
    //TODO do all fileds PRIVATE and use callback interface
    pub widgets     : HashMap<u16,Widget>,
    pub objects     : HashMap<u32,Obj>,
    pub charlist    : Vec<String>,
    pub resources   : HashMap<u16,String>,
    pub seq         : u16,
    pub rx_rel_seq  : u16,
    pub que         : LinkedList<EnqueuedBuffer>,
    pub tx_buf      : LinkedList<EnqueuedBuffer>,
    pub enqueue_seq : usize,
    pub rel_cache   : HashMap<u16,Rel>,
    pub hero        : Hero,
    pub map         : Map,
        events      : LinkedList<Event>,
}

impl State {
    pub fn new () -> State {
        let mut widgets = HashMap::new();
        widgets.insert(0, Widget{ id:0, typ:"root".to_string(), parent:0, name:None });

        State {
            widgets: widgets,
            objects: HashMap::new(),
            charlist: Vec::new(),
            resources: HashMap::new(),
            seq: 0,
            rx_rel_seq: 0,
            que: LinkedList::new(),
            tx_buf: LinkedList::new(),
            enqueue_seq: 0,
            rel_cache: HashMap::new(),
            hero: Hero {
                name: None,
                obj: None,
                weight: None,
                tmexp: None,
                hearthfire: None,
                inventory: HashMap::new(),
                start_xy: None,
            },
            map: Map{ partial: HashMap::new(), grids: HashMap::new() },
            events: LinkedList::new(),
        }
    }

    pub fn start_send_beats () {
        /*TODO*/
    }

    pub fn enqueue_to_send (&mut self, msg: Message) -> Result<(),Error> {
        match msg.to_buf() {
            Ok(buf) => {
                match msg {
                    Message::C_SESS(_) |
                    Message::REL(_) |
                    Message::MAPREQ(_) |
                    Message::CLOSE => {
                        //TODO maybe we should increase timeout in the case of MAPREQ?
                        let ebuf = EnqueuedBuffer{buf : buf, timeout : Some(Timeout{ms : 100, seq : self.enqueue_seq})};
                        if self.que.is_empty() {
                            self.tx_buf.push_front(ebuf.clone());
                        }
                        self.que.push_front(ebuf);
                        self.enqueue_seq += 1;
                    }
                    Message::ACK(_) |
                    Message::BEAT |
                    Message::OBJACK(_) => {
                        let ebuf = EnqueuedBuffer{buf : buf, timeout : None};
                        self.tx_buf.push_front(ebuf);
                    }
                    Message::S_SESS(_) |
                    Message::MAPDATA(_) |
                    Message::OBJDATA(_) => {
                        return Err(Error{source:"client must NOT send this kind of message",detail:None});
                    }
                }
                Ok(())
            },
            Err(e) => { println!("enqueue error: {:?}", e); Err(e) },
        }
    }

    pub fn dispatch_message (&mut self, buf:&[u8]/*, tx_buf:&mut LinkedList<Vec<u8>>*/) -> Result<(),Error> {
        let (msg,remains) = match Message::from_buf(buf,MessageDirection::FromServer) {
            Ok((msg,remains)) => { (msg,remains) },
            Err(err) => { println!("message parse error: {:?}", err); return Err(err); },
        };

        if let Some(remains) = remains {
            println!("                 REMAINS {} bytes", remains.len());
        }

        match msg {
            Message::S_SESS(sess) => {
                println!("RX: S_SESS {:?}", sess.err);
                match sess.err {
                    SessError::OK => {},
                    _ => {
                        //TODO return Error::from(SessError)
                        return Err(Error{source:"session error",detail:None});
                        //XXX ??? should we send CLOSE too ???
                        //??? or can we re-send our SESS requests in case of BUSY err ?
                    }
                }
                self.remove_sess_from_que();
                Self::start_send_beats();
            },
            Message::C_SESS(_) => { println!("     !!! client must not receive C_SESS !!!"); },
            Message::REL( rel ) => {
                //println!("RX: REL {}", rel.seq);
                if rel.seq == self.rx_rel_seq {
                    try!(self.dispatch_rel_cache(&rel));
                } else {
                    let cur = self.rx_rel_seq;
                    let new = rel.seq;
                    let future = ((new > cur) && ((new - cur) < (u16::MAX / 2))) ||
                                 ((new < cur) && ((cur - new) > (u16::MAX / 2)));
                    if future {
                        // future REL
                        self.cache_rel(rel);
                    } else {
                        // past REL
                        println!("past");
                        //TODO self.ack(seq);
                        let last_acked_seq = self.rx_rel_seq - 1;
                        try!(self.enqueue_to_send(Message::ACK(Ack{seq : last_acked_seq})));
                    }
                }
            },
            Message::ACK(ack)   => {
                //println!("RX: ACK {}", ack.seq);
                if ack.seq == self.seq {
                    //println!("our rel {} acked", self.seq);
                    self.remove_rel_from_que();
                    //FIXME self.seq += last_rel.rels.len()
                    self.seq += 1;
                }
            },
            Message::BEAT       => { println!("     !!! client must not receive BEAT !!!"); },
            Message::MAPREQ(_)  => { println!("     !!! client must not receive MAPREQ !!!"); },
            Message::MAPDATA(mapdata) => {
                //println!("RX: MAPDATA {:?}", mapdata);
                let pktid = mapdata.pktid;
                self.map.append(mapdata);
                //TODO if self.mapdata.complete() { ... }
                if self.map.complete(pktid) {
                    //TODO let map = self.mapdata.assemble(pktid).to_map();
                    let map_buf = self.map.assemble(pktid);
                    let map = self.map.from_buf(map_buf);
                    println!("MAP COMPLETE ({},{}) name='{}' id={} tiles=[..{}] z=[..{}]", map.x, map.y, map.name, map.id, map.tiles.len(), map.z.len());
                    self.events.push_front(Event::Grid(map.x,map.y,map.tiles.clone(),map.z.clone()));
                    self.map.grids.insert((map.x,map.y),map);
                    //TODO complete map only if (x,y) == requested (x,y)
                    self.remove_mapreq_from_que();
                }
            },
            Message::OBJDATA(objdata) => {
                //println!("RX: OBJDATA {:?}", objdata);
                try!(self.enqueue_to_send(Message::OBJACK(ObjAck::new(&objdata)))); // send OBJACKs
                for o in objdata.obj.iter() {
                    //FIXME ??? do NOT add hero object
                    //TODO  if o.id == self.hero.id {
                    //          ... do something with hero, not in objects ...
                    //          if odMOVE {
                    //              if hero.grid.is_changed() {
                    //                  self.request_grids_around();
                    //              }
                    //          }
                    //      }
                    {
                        let mut to_remove = false;
                        for prop in o.prop.iter() {
                            if let ObjProp::odREM = *prop {
                                 to_remove = true;
                                 break;
                            }
                        }
                        if to_remove {
                            self.objects.remove(&o.id);
                            continue;
                        }
                    }
                    let obj = self.objects.entry(o.id).or_insert(Obj{id:o.id, frame:0, resid:0, x:0, y:0, movement:None});
                    //FIXME consider o.frame overflow !!!
                    if o.frame <= obj.frame {
                        continue;
                    }
                    for prop in o.prop.iter() {
                        match *prop {
                            ObjProp::odMOVE(xy,_) => { let (x,y) = xy; obj.x = x; obj.y = y; }
                            ObjProp::odRES(resid) => { obj.resid = resid; }
                            ObjProp::odCOMPOSE(resid) => { obj.resid = resid; }
                            ObjProp::odLINBEG((x1,y1),(x2,y2),steps) => {
                                obj.movement = Some(Movement{
                                    from: (x1,y1),
                                    to: (x2,y2),
                                    steps: steps,
                                    step: 0
                                })
                            }
                            ObjProp::odLINSTEP(step) => {
                                let movement = match obj.movement {
                                    Some(ref m) => {
                                        if (step > 0) && (step < m.steps) {
                                            if step <= m.step {
                                                println!("WARNING: odLINSTEP step <= m.step");
                                            }
                                            Some(Movement{
                                                from: m.from,
                                                to: m.to,
                                                steps: m.steps,
                                                step: step })
                                        } else {
                                            None
                                        }
                                    }
                                    None => {
                                        println!("WARNING: odLINSTEP while movement == None");
                                        None
                                    }
                                };
                                obj.movement = movement;
                            }
                            _ => {}
                        }
                    }
                }
            },
            Message::OBJACK(_)  => {},
            Message::CLOSE => {
                println!("RX: CLOSE");
                //TODO return Status::EndOfSession instead of Error
                return Err(Error{source:"session closed",detail:None});
            },
        }

        //TODO return Status::Continue/AllOk instead of ()
        Ok(())
    }

    fn cache_rel (&mut self, rel: Rel) {
        println!("cache REL {}-{}", rel.seq, rel.seq + ((rel.rel.len() as u16) - 1));
        self.rel_cache.insert(rel.seq, rel);
    }

    fn dispatch_rel_cache (&mut self, rel: &Rel) -> Result<(),Error> {
        //XXX are we handle seq right in the case of overflow ???
        let mut next_rel_seq = rel.seq + ((rel.rel.len() as u16) - 1);
        self.dispatch_rel(rel);
        loop {
            let next_rel = self.rel_cache.remove(&(next_rel_seq + 1));
            match next_rel {
                Some(rel) => {
                    next_rel_seq = rel.seq + ((rel.rel.len() as u16) - 1);
                    self.dispatch_rel(&rel);
                }
                None => {
                    break;
                }
            }
        }
        try!(self.enqueue_to_send(Message::ACK(Ack{seq : next_rel_seq})));
        self.rx_rel_seq = next_rel_seq + 1;
        Ok(())
    }

    fn dispatch_rel (&mut self, rel: &Rel) {
        println!("dispatch REL {}-{}", rel.seq, rel.seq + ((rel.rel.len() as u16) - 1));
        //println!("RX: {:?}", rel);
        for r in rel.rel.iter() {
            match *r {
                RelElem::NEWWDG(ref wdg) => {
                    //println!("      {:?}", wdg);
                    self.dispatch_newwdg(wdg);
                },
                RelElem::WDGMSG(ref msg) => {
                    //println!("      {:?}", msg);
                    self.dispatch_wdgmsg(msg);
                },
                RelElem::DSTWDG(ref wdg) => {
                    //println!("      {:?}", wdg);
                    self.widgets.remove(&wdg.id);
                },
                RelElem::MAPIV(_) => {},
                RelElem::GLOBLOB(_) => {},
                RelElem::PAGINAE(_) => {},
                RelElem::RESID(ref res) => {
                    //println!("      {:?}", res);
                    self.resources.insert(res.id, res.name.clone()/*FIXME String -> &str*/);
                },
                RelElem::PARTY(_) => {},
                RelElem::SFX(_) => {},
                RelElem::CATTR(_) => {},
                RelElem::MUSIC(_) => {},
                RelElem::TILES(_) => {},
                RelElem::BUFF(_) => {},
                RelElem::SESSKEY(_) => {},
            }
        }
    }

    fn dispatch_newwdg (&mut self, wdg: &NewWdg) {
        self.widgets.insert(wdg.id, Widget{id:wdg.id, typ:wdg.name.clone(), parent:wdg.parent, name:None});
        match wdg.name.as_str() {
            "gameui" => {
                if let Some(&MsgList::tSTR(ref name)) = wdg.cargs.get(0) {
                    self.hero.name = Some(name.clone());
                    println!("HERO: name = '{:?}'", self.hero.name);
                }
                if let Some(&MsgList::tINT(obj)) = wdg.cargs.get(1) {
                    //FIXME BUG: object ID is uint32 but here it is int32 WHY??? XXX
                    self.hero.obj = Some(obj as u32);
                    println!("HERO: obj = '{:?}'", self.hero.obj);

                    match self.hero_xy() {
                        Some(xy) => { self.hero.start_xy = Some(xy); }
                        None => { panic!("we have received hero object ID, but hero XY is None"); }
                    }

                    self.update_grids_around();
                }
            }
            "item" => {
                if let Some(parent) = self.widgets.get(&(wdg.parent)) {
                    if parent.typ == "inv" {
                        //8 "item", pargs: [tCOORD((2, 1))], cargs: [tUINT16(2529)] }
                        if let Some(&MsgList::tCOORD((x,y))) = wdg.pargs.get(0) {
                            if let Some(&MsgList::tUINT16(id)) = wdg.cargs.get(0) {
                                self.hero.inventory.insert((x,y), id);
                                println!("HERO: inventory: {:?}", self.hero.inventory);
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }

    fn dispatch_wdgmsg (&mut self, msg: &WdgMsg) {
        if let Some(w) = self.widgets.get(&(msg.id)) {
            match w.typ.as_str() {
                "charlist" => {
                    if msg.name == "add" {
                        if let Some(&MsgList::tSTR(ref name)) = msg.args.get(0) {
                            println!("    add char '{}'", name);
                            /*FIXME rewrite without cloning*/
                            self.charlist.push(name.clone());
                        }
                    }
                }
                "gameui" => {
                    if msg.name == "weight" {
                        if let Some(&MsgList::tUINT16(w)) = msg.args.get(0) {
                            self.hero.weight = Some(w);
                            println!("HERO: weight = '{:?}'", self.hero.weight);
                        }
                    }
                }
                "chr" => {
                    if msg.name == "tmexp" {
                        if let Some(&MsgList::tINT(i)) = msg.args.get(0) {
                            self.hero.tmexp = Some(i);
                            println!("HERO: tmexp = '{:?}'", self.hero.tmexp);
                        }
                    }
                }
                "ui/hrtptr:11" => {
                    if msg.name == "upd" {
                        if let Some(&MsgList::tCOORD((x,y))) = msg.args.get(0) {
                            //self.objects.insert(0xffffffff, Obj{resid:0xffff, x:x, y:y});
                            self.hero.hearthfire = Some((x,y));
                            println!("HERO: heathfire = '{:?}'", self.hero.hearthfire);
                        }
                    }
                }
                _ => {}
            }
        }
    }

    fn update_grids_around (&mut self) {
        //TODO move to fn client.update_grids_around(...) { ... }
        //     if client.hero.current_grid_is_changed() { client.update_grids_around(); }
        //TODO if grids.not_contains(xy) and requests.not_contains(xy) then add_map_request(xy)
        if let Some(xy) = self.hero_grid_xy() {
            let (x,y) = xy;
            self.mapreq(x,y).unwrap();
            self.mapreq(x-1,y-1).unwrap();
            self.mapreq(x,y-1).unwrap();
            self.mapreq(x+1,y-1).unwrap();
            self.mapreq(x-1,y).unwrap();
            self.mapreq(x+1,y).unwrap();
            self.mapreq(x-1,y+1).unwrap();
            self.mapreq(x,y+1).unwrap();
            self.mapreq(x+1,y+1).unwrap();
        } else {
            panic!("update_grids_around when hero_grid_xy is None");
        }
    }

    fn remove_sess_from_que (&mut self) {
        let mut should_be_removed = false;
        if let Some(ref buf) = self.que.back() {
            if let Ok(msg) = Message::from_buf(&buf.buf, MessageDirection::FromClient) {
                if let (Message::C_SESS(_),_) = msg {
                    should_be_removed = true;
                }
            }
        }
        if should_be_removed {
            self.que.pop_back();
            match self.que.back() {
                Some(buf) => {
                    //println!("enqueue next packet");
                    self.tx_buf.push_front(buf.clone());
                }
                None => {
                    //println!("remove_sess: empty que");
                }
            }
        }
    }

    //TODO do something with this ugly duplication of previous fn
    fn remove_rel_from_que (&mut self) {
        let mut should_be_removed = false;
        if let Some(ref buf) = self.que.back() {
            if let Ok(msg) = Message::from_buf(&buf.buf, MessageDirection::FromClient) {
                //FIXME TODO check that this is exactly same REL we are waiting for
                if let (Message::REL(_),_) = msg {
                    should_be_removed = true;
                }
            }
        }
        if should_be_removed {
            self.que.pop_back();
            match self.que.back() {
                Some(buf) => {
                    //println!("enqueue next packet");
                    self.tx_buf.push_front(buf.clone());
                }
                None => {
                    //println!("remove_rel: empty que");
                }
            }
        }
    }

    //TODO do something with this ugly duplication of previous fn
    fn remove_mapreq_from_que (&mut self) {
        let mut should_be_removed = false;
        if let Some(ref buf) = self.que.back() {
            if let Ok(msg) = Message::from_buf(&buf.buf, MessageDirection::FromClient) {
                //FIXME TODO check that this is exactly MAPDATA we are waiting for
                if let (Message::MAPREQ(_),_) = msg {
                    should_be_removed = true;
                }
            }
        }
        if should_be_removed {
            self.que.pop_back();
            match self.que.back() {
                Some(buf) => {
                    //println!("enqueue next packet");
                    self.tx_buf.push_front(buf.clone());
                }
                None => {
                    //println!("remove_mapreq: empty que");
                }
            }
        }
    }

    pub fn widget_id (&self, typ: &str, name: Option<String>) -> Option<u16> {
        for (id,w) in self.widgets.iter() {
            if (w.typ == typ) && (w.name == name) {
                return Some(*id)
            }
        }
        None
    }

    pub fn widget_exists (&self, typ: &str, name: Option<String>) -> bool {
        match self.widget_id(typ, name) {
            Some(_) => true,
            None => false
        }
    }

    pub fn connect (&mut self, login: &str, cookie: &[u8]) -> Result<(),Error> {
        //TODO send SESS until reply
        //TODO get username from server responce, not from auth username
        //let cookie = self.cookie.clone();
        //let user = self.user.clone();
        try!(self.enqueue_to_send(Message::C_SESS(cSess{ login: login.to_string(), cookie: cookie.to_vec() })));
        Ok(())
    }

    pub fn send_play (&mut self, i: usize) -> Result<(),Error> {
        //TODO let mut rel = Rel::new(seq,id,name);
        let mut rel = Rel{seq:self.seq, rel:Vec::new()};
        let id = self.widget_id("charlist", None).expect("charlist widget is not found");
        let name = "play".to_string();
        let charname = self.charlist[i].clone();
        println!("send play '{}'", charname);
        let mut args : Vec<MsgList> = Vec::new();
        args.push(MsgList::tSTR(charname));
        //TODO rel.append(RelElem::new())
        let elem = RelElem::WDGMSG(WdgMsg{ id : id, name : name, args : args });
        rel.rel.push(elem);
        self.enqueue_to_send(Message::REL(rel))
    }

    pub fn mapreq (&mut self, x:i32, y:i32) -> Result<(),Error> {
        //TODO replace with client.send(Message::MapReq::new(x,y).to_buf())
        //     or client.send(Message::mapreq(x,y).to_buf())
        //TODO add "force" flag to update this grid forcelly
        if !self.map.grids.contains_key(&(x,y)) {
            try!(self.enqueue_to_send(Message::MAPREQ(MapReq{x:x,y:y})));
        }
        Ok(())
    }

    pub fn rx (&mut self, buf:&[u8]) -> Result<(),Error> {
        self.dispatch_message(buf)
    }

    pub fn timeout (&mut self, seq: usize) {
        match self.que.back() {
            Some(ref mut buf) => {
                match buf.timeout {
                    Some(ref timeout) => {
                        if timeout.seq == seq {
                            //println!("timeout {}: re-enqueue", seq);
                            self.tx_buf.push_front(buf.clone());
                        } else {
                            //println!("timeout {}: packet dropped", seq);
                        }
                    }
                    None => {
                        println!("ERROR: enqueued packet without timeout");
                    }
                }
            }
            None => {
                //println!("timeout {}: empty que", seq);
            }
        }
    }

    pub fn tx (&mut self) -> Option<EnqueuedBuffer> {
        let buf = self.tx_buf.pop_back();
        if let Some(ref buf) = buf {
            match Message::from_buf(buf.buf.as_slice(), MessageDirection::FromClient) {
                Ok((msg,_)) => /*println!("TX: {:?}", msg)*/(),
                Err(e) => panic!("ERROR: malformed TX message: {:?}", e),
            }
        }
        buf
    }

    pub fn close (&mut self) -> Result<(),Error> {
        try!(self.enqueue_to_send(Message::CLOSE));
        Ok(())
    }

    /*
    pub fn ready_to_go (&self) -> bool {
        let mut ret = false;
        for name in self.widgets.values() {
            if name == "mapview" {
                ret = true;
                break;
            }
        }
        return ret;
    }
    */

    pub fn go (&mut self, x: i32, y: i32) -> Result<(),Error> /*TODO Option<Error>*/ {
        println!("GO");
        //TODO let mut rel = Rel::new(seq,id,name);
        let mut rel = Rel{seq:self.seq, rel:Vec::new()};
        let id = self.widget_id("mapview", None).expect("mapview widget is not found");
        let name : String = "click".to_string();
        let mut args : Vec<MsgList> = Vec::new();
        args.push(MsgList::tCOORD((907, 755))); //TODO set some random coords in the center of screen
        args.push(MsgList::tCOORD((x, y)));
        args.push(MsgList::tINT(1));
        args.push(MsgList::tINT(0));
        let elem = RelElem::WDGMSG(WdgMsg{ id : id, name : name, args : args });
        rel.rel.push(elem);
        try!(self.enqueue_to_send(Message::REL(rel)));
        Ok(())
    }

    /*
    pub fn pick (&mut self, obj_id: u32) -> Result<(),Error> {
        println!("PICK");
        //TODO let mut rel = Rel::new(seq,id,name);
        let mut rel = Rel{seq:self.seq, rel:Vec::new()};
        let id = self.widget_id("mapview", None).expect("mapview widget is not found");
        let name = "click".to_string();
        let mut args = Vec::new();
        let (obj_x,obj_y) = {
            let obj = self.objects.get(&obj_id).unwrap();
            (obj.x, obj.y)
        };
        args.push(MsgList::tCOORD((863, 832))); //TODO set some random coords in the center of screen
        args.push(MsgList::tCOORD((obj_x-1, obj_y+1)));
        args.push(MsgList::tINT(3));
        args.push(MsgList::tINT(0));
        args.push(MsgList::tINT(0));
        args.push(MsgList::tINT(obj_id as i32));
        args.push(MsgList::tCOORD((obj_x, obj_y)));
        args.push(MsgList::tINT(0));
        args.push(MsgList::tINT(-1));
        let elem = RelElem::WDGMSG(WdgMsg{ id : id, name : name, args : args });
        rel.rel.push(elem);
        try!(self.enqueue_to_send(Message::REL(rel)));
        Ok(())
    }

    pub fn choose_pick (&mut self, wdg_id: u16) -> Result<(),Error> {
        println!("GO");
        //TODO let mut rel = Rel::new(seq,id,name);
        let mut rel = Rel{seq:self.seq, rel:Vec::new()};
        let name = "cl".to_string();
        let mut args = Vec::new();
        args.push(MsgList::tINT(0));
        args.push(MsgList::tINT(0));
        let elem = RelElem::WDGMSG(WdgMsg{ id : wdg_id, name : name, args : args });
        rel.rel.push(elem);
        try!(self.enqueue_to_send(Message::REL(rel)));
        Ok(())
    }
    */

    //TODO fn grid(Coord) {...}, fn xy(Grid) {...}
    //     and then we can do: hero.grid().xy();

    pub fn hero_obj (&self) -> Option<&Obj>{
        match self.hero.obj {
            Some(id) => self.objects.get(&id),
            None => None
        }
    }

    pub fn hero_xy (&self) -> Option<(i32,i32)> {
        match self.hero_obj() {
            Some(hero) => Some((hero.x,hero.y)),
            None => None
        }
    }

    pub fn hero_grid_xy (&self) -> Option<(i32,i32)> {
        match self.hero_xy() {
            Some(xy) => Some(grid(xy)),
            None => None
        }
    }

    pub fn hero_grid (&self) -> Option<&Surface> {
        match self.hero_grid_xy() {
            Some(xy) => self.map.grids.get(&xy),
            None => None
        }
    }

    pub fn hero_exists (&self) -> bool {
        match self.hero_obj() {
            Some(_) => true,
            None => false
        }
    }

    pub fn hero_grid_exists (&self) -> bool {
        match self.hero_grid() {
            Some(_) => true,
            None => false
        }
    }

    pub fn hero_movement (&self) -> Option<Movement> {
        match self.hero_obj() {
            Some(hero) => hero.movement,
            None => None
        }
    }

    pub fn hero_is_moving (&self) -> bool {
        match self.hero_movement() {
            Some(_) => true,
            None => false
        }
    }

    pub fn start_point (&self) -> Option<(i32,i32)> {
        self.hero.start_xy
    }

    pub fn next_event (&mut self) -> Option<Event> {
        self.events.pop_back()
    }
}

pub fn grid ((x,y): (i32,i32)) -> (i32,i32) {
    let mut gx = x / 1100; if x < 0 { gx -= 1; }
    let mut gy = y / 1100; if y < 0 { gy -= 1; }
    (gx,gy)
}

/*

CLIENT
 REL  seq=4
  WDGMSG len=65
   id=6 name=click
     COORD : [907, 755]        Coord pc
     COORD : [39683, 36377]    Coord mc
     INT : 1                   int clickb
     INT : 0                   ui.modflags()
     INT : 0                   inf.ol != null
     INT : 325183464           (int)inf.gob.id
     COORD : [39737, 36437]    inf.gob.rc
     INT : 0                   inf.ol.id
     INT : -1                  inf.r.id or -1

CLIENT
 REL  seq=5
  WDGMSG len=36
   id=6 name=click
     COORD : [1019, 759]        Coord pc
     COORD : [39709, 36386]     Coord mc
     INT : 1                    int clickb
     INT : 0                    ui.modflags()

private class Click extends Hittest {
    int clickb;

    private Click(Coord c, int b) {
        super(c);
        clickb = b;
    }

    protected void hit(Coord pc, Coord mc, ClickInfo inf) {
        if(inf == null) {
            wdgmsg("click", pc, mc, clickb, ui.modflags());
        } else {
            if(inf.ol == null) {
                wdgmsg("click", pc, mc, clickb, ui.modflags(), 0, (int)inf.gob.id, inf.gob.rc, 0, getid(inf.r));
            } else {
                wdgmsg("click", pc, mc, clickb, ui.modflags(), 1, (int)inf.gob.id, inf.gob.rc, inf.ol.id, getid(inf.r));
            }
        }
    }
}

*/

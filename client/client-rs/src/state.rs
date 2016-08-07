use std::collections::hash_map::HashMap;
use std::collections::LinkedList;

use std::vec::Vec;
use std::io::Cursor;
use std::io::Read;
use std::io::BufRead;
use std::u16;

extern crate byteorder;
use self::byteorder::LittleEndian;
use self::byteorder::BigEndian;
use self::byteorder::ReadBytesExt;
// use self::byteorder::WriteBytesExt;
#[allow(non_camel_case_types)]
type le = LittleEndian;
#[allow(non_camel_case_types)]
type be = BigEndian;

use proto::message_mapdata::MapData;
use proto::message_rel::Rel;
use proto::message::Message;
use proto::message::MessageDirection;
use proto::message_sess::SessError;
use proto::message_objdata::ObjDataElemProp;
use proto::msg_list::MsgList;
use proto::message_ack::Ack;
use proto::message_objack::ObjAck;
use proto::message_rel::RelElem;
use proto::message_rel::NewWdg;
use proto::message_rel::WdgMsg;
use proto::message_sess::cSess;
use proto::message_mapreq::MapReq;

use ::Error;

// extern crate rustc_serialize;
// use self::rustc_serialize::hex::ToHex;

extern crate flate2;
// use std::io::prelude::*;
use self::flate2::read::ZlibDecoder;

pub type Resid = u16;
pub type Coord = (i32, i32);

struct ObjProp {
    xy: Option<Coord>,
    resid: Option<Resid>, // TODO replace with Vec<resid> for composite objects
    line: Option<(Coord, Coord, i32)>, // TODO replace with struct LinearMovement
    step: Option<i32>,
}

impl ObjProp {
    fn new() -> Self {
        ObjProp {
            xy: None,
            resid: None,
            line: None,
            step: None,
        }
    }

    fn from_obj_data_elem_prop(odep: &[ObjDataElemProp]) -> Option<Self> {
        let mut prop = Self::new();
        for p in odep {
            match *p {
                ObjDataElemProp::odREM => {
                    return None;
                }
                ObjDataElemProp::odMOVE(xy, _) => {
                    prop.xy = Some(xy);
                }
                ObjDataElemProp::odRES(resid) => {
                    prop.resid = Some(resid);
                }
                ObjDataElemProp::odCOMPOSE(resid) => {
                    prop.resid = Some(resid);
                }
                ObjDataElemProp::odLINBEG(from, to, steps) => {
                    prop.line = Some((from, to, steps));
                }
                ObjDataElemProp::odLINSTEP(step) => {
                    prop.step = Some(step);
                }
                _ => {}
            }
        }
        Some(prop)
    }
}

#[derive(Debug)]
pub struct Obj {
    pub id: u32, // TODO maybe remove this? because this is also a key field in objects hashmap
    pub frame: Option<i32>,
    pub resid: Option<Resid>,
    pub xy: Option<Coord>,
    pub movement: Option<Movement>,
}

impl Obj {
    fn new(id: u32, frame: Option<i32>, resid: Option<Resid>, xy: Option<Coord>, movement: Option<Movement>) -> Obj {
        Obj {
            id: id,
            frame: frame,
            resid: resid,
            xy: xy,
            movement: movement,
        }
    }

    fn update(&mut self, prop: &ObjProp) {
        if let Some(resid) = prop.resid {
            self.resid = Some(resid);
        }

        if let Some(xy) = prop.xy {
            self.xy = Some(xy);
        }

        if let Some((from, to, steps)) = prop.line {
            self.movement = Some(Movement::new(from, to, steps, 0));
        }

        if let Some(step) = prop.step {
            let movement = match self.movement {
                Some(ref m) => {
                    if (step > 0) && (step < m.steps) {
                        if step <= m.step {
                            warn!("odLINSTEP step <= m.step");
                        }
                        Some(Movement::new(m.from, m.to, m.steps, step))
                    } else {
                        None
                    }
                }
                None => {
                    warn!("odLINSTEP({}) while movement == None", step);
                    None
                }
            };
            self.movement = movement;
        }
    }
}

#[derive(Clone,Copy,Debug)]
pub struct Movement {
    pub from: Coord,
    pub to: Coord,
    pub steps: i32,
    pub step: i32,
}

impl Movement {
    fn new(from: Coord, to: Coord, steps: i32, step: i32) -> Movement {
        Movement {
            from: from,
            to: to,
            steps: steps,
            step: step,
        }
    }
}

#[derive(Clone)]
pub struct Timeout {
    pub ms: u64,
    pub seq: usize,
}

#[allow(non_camel_case_types)]
#[derive(Clone,PartialEq)]
pub enum MessageHint {
    C_SESS,
    REL(u16),
    MAPREQ(i32, i32),
    CLOSE,
    NONE,
}

#[derive(Clone)]
pub struct EnqueuedBuffer {
    pub buf: Vec<u8>,
    pub msg_hint: MessageHint,
    pub timeout: Option<Timeout>,
}

pub struct Widget {
    pub id: u16,
    pub typ: String,
    pub parent: u16,
    pub name: Option<String>,
}

pub struct Hero {
    pub name: Option<String>,
    pub obj: Option<u32>,
    pub weight: Option<u16>,
    pub tmexp: Option<i32>,
    pub hearthfire: Option<Coord>,
    pub inventory: HashMap<Coord, u16>,
    pub equipment: HashMap<u8, u16>,
    pub start_xy: Option<Coord>,
}

pub struct MapPieces {
    total_len: u16,
    pieces: HashMap<u16, Vec<u8>>,
}

pub struct Surface {
    pub x: i32,
    pub y: i32,
    pub name: String,
    pub id: i64,
    pub tiles: Vec<u8>,
    pub z: Vec<i16>, // pub ol: Vec<u8>,
}

pub type PacketId = i32;

pub struct Map {
    pub partial: HashMap<PacketId, MapPieces>, // TODO somehow clean up from old pieces (periodically or whatever)
    pub grids: HashMap<Coord, (String, i64) /* TODO struct GridHint */>,
}

impl Map {
    fn append(&mut self, mapdata: MapData) {
        let map = self.partial.entry(mapdata.pktid).or_insert(MapPieces {
            total_len: mapdata.len,
            pieces: HashMap::new(),
        });
        map.pieces.insert(mapdata.off, mapdata.buf);
    }

    fn complete(&self, pktid: i32) -> bool {
        let map = match self.partial.get(&pktid) {
            Some(m) => m,
            None => {
                return false;
            }
        };
        let mut len = 0u16;
        loop {
            match map.pieces.get(&len) {
                Some(buf) => {
                    len += buf.len() as u16;
                    if len == map.total_len {
                        return true;
                    }
                }
                None => {
                    return false;
                }
            }
        }
    }

    fn assemble(&mut self, pktid: i32) -> Vec<u8> /*TODO return Result*/ {
        let map = match self.partial.remove(&pktid) {
            Some(map) => map,
            None => {
                return Vec::new();
            }
        };
        let mut len = 0;
        let mut buf: Vec<u8> = Vec::new();
        while let Some(b) = map.pieces.get(&len) {
            buf.extend(b);
            len += b.len() as u16;
            if len == map.total_len {
                break;
            }
        }
        if buf.len() as u16 != map.total_len {
            info!("ERROR: buf.len() as u16 != map.total_len");
            // return Err(Error{source:"buf.len() as u16 != map.total_len",detail:None});
        }
        buf
    }

    // XXX ??? move to message ?
    fn from_buf(buf: Vec<u8>) -> Surface {
        let mut r = Cursor::new(buf);
        let x = r.read_i32::<le>().unwrap();
        let y = r.read_i32::<le>().unwrap();
        let mmname = {
            let mut tmp = Vec::new();
            r.read_until(0, &mut tmp).unwrap();
            tmp.pop();
            String::from_utf8(tmp).unwrap()
        };
        // let mut pfl = vec![0; 256];
        loop {
            let pidx = r.read_u8().unwrap();
            if pidx == 255 {
                break;
            }
            // pfl[pidx as usize]
            let _ = r.read_u8().unwrap();
        }
        let mut decoder = ZlibDecoder::new(r);
        let mut unzipped = Vec::new();
        let /*unzipped_len*/ _ = decoder.read_to_end(&mut unzipped).unwrap();
        // TODO check unzipped_len
        let mut r = Cursor::new(unzipped);
        let id = r.read_i64::<le>().unwrap();
        let mut tiles = Vec::with_capacity(100 * 100);
        for _ in 0..100 * 100 {
            tiles.push(r.read_u8().unwrap());
        }
        let mut z = Vec::with_capacity(100 * 100);
        for _ in 0..100 * 100 {
            z.push(r.read_i16::<le>().unwrap());
        }
        // let mut ol = vec![0; 100*100];
        // loop {
        //     let pidx = r.read_u8().unwrap();
        //     if pidx == 255 { break; }
        //     let fl = pfl[pidx as usize];
        //     let typ = r.read_u8().unwrap();
        //     let (x1,y1) = (r.read_u8().unwrap() as usize, r.read_u8().unwrap() as usize);
        //     let (x2,y2) = (r.read_u8().unwrap() as usize, r.read_u8().unwrap() as usize);
        //     info!("#### {} ({},{}) - ({},{})", typ, x1, y1, x2, y2);
        //     let oli = match typ {
        //         0 => if (fl & 1) == 1 { 2 } else { 1 },
        //         1 => if (fl & 1) == 1 { 8 } else { 4 },
        //         2 => 16,
        //         _ => { info!("ERROR: unknown plot type {}", typ); break; }
        //     };
        //     for y in y1..y2+1 {
        //         for x in x1..x2+1 {
        //             ol[x+y*100] |= oli;
        //         }
        //     }
        // }
        Surface {
            x: x,
            y: y,
            name: mmname,
            id: id,
            tiles: tiles,
            z: z, // ,ol:ol
        }
    }
}

pub enum Event {
    Grid(i32, i32, Vec<u8>, Vec<i16>), // TODO struct Grid { x: i32, y: i32, tiles: Vec<u8>, z: Vec<i16> }
    Obj(Coord),
}

pub struct State {
    // TODO do all fileds PRIVATE and use callback interface
    pub widgets: HashMap<u16, Widget>,
    pub objects: HashMap<u32, Obj>,
    pub charlist: Vec<String>,
    pub resources: HashMap<u16, String>,
    pub seq: u16,
    pub rx_rel_seq: u16, //TODO wrap this to struct OverflowableCounter to incapsulate correct handling of all the operations on it
    pub que: LinkedList<EnqueuedBuffer>,
    pub tx_buf: LinkedList<EnqueuedBuffer>,
    pub enqueue_seq: usize,
    pub rel_cache: HashMap<u16, Rel>, //TODO unify with rx_rel_seq to have more consistent entity (struct Rel { ... })
    pub hero: Hero,
    pub map: Map,
    events: LinkedList<Event>,
    origin: Option<Coord>,
}

impl State {
    pub fn new() -> State {
        let mut widgets = HashMap::new();
        widgets.insert(0,
                       Widget {
                           id: 0,
                           typ: "root".to_owned(),
                           parent: 0,
                           name: None,
                       });

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
                equipment: HashMap::new(),
                start_xy: None,
            },
            map: Map {
                partial: HashMap::new(),
                grids: HashMap::new(),
            },
            events: LinkedList::new(),
            origin: None,
        }
    }

    pub fn start_send_beats() {
        // TODO
    }

    pub fn enqueue_to_send(&mut self, mut msg: Message) -> Result<(), Error> {
        if let Message::REL(ref mut rel) = msg {
            assert!(rel.seq == 0);
            rel.seq = self.seq;
            self.seq += rel.rel.len() as u16;
        }
        match msg.to_buf() {
            Ok(buf) => {
                let (msg_hint, timeout) = match msg {
                    Message::C_SESS(_) => {
                        (MessageHint::C_SESS,
                         Some(Timeout {
                            ms: 100,
                            seq: self.enqueue_seq,
                        }))
                    }
                    Message::REL(rel) => {
                        (MessageHint::REL(rel.seq),
                         Some(Timeout {
                            ms: 100,
                            seq: self.enqueue_seq,
                        }))
                    }
                    Message::CLOSE => {
                        (MessageHint::CLOSE,
                         Some(Timeout {
                            ms: 100,
                            seq: self.enqueue_seq,
                        }))
                    }
                    Message::MAPREQ(mapreq) => {
                        (MessageHint::MAPREQ(mapreq.x, mapreq.y),
                         Some(Timeout {
                            ms: 400,
                            seq: self.enqueue_seq,
                        }))
                    }
                    Message::ACK(_) |
                    Message::BEAT |
                    Message::OBJACK(_) => (MessageHint::NONE, None),
                    Message::S_SESS(_) |
                    Message::MAPDATA(_) |
                    Message::OBJDATA(_) => {
                        return Err(Error {
                            source: "client must NOT send this kind of message",
                            detail: None,
                        });
                    }
                };

                let ebuf = EnqueuedBuffer {
                    buf: buf,
                    timeout: timeout,
                    msg_hint: msg_hint,
                };

                match ebuf.timeout {
                    Some(_) => {
                        // FIXME TODO merge que and tx_buf (remove tx_buf and que only)
                        //     + remove EnqueuedBuffer clone deriving
                        if self.que.is_empty() {
                            self.tx_buf.push_front(ebuf.clone());
                        }
                        self.que.push_front(ebuf);
                        self.enqueue_seq += 1;
                    }
                    None => {
                        self.tx_buf.push_front(ebuf);
                    }
                }

                Ok(())
            }
            Err(e) => {
                info!("enqueue error: {:?}", e);
                Err(e)
            }
        }
    }

    pub fn dispatch_message(&mut self, buf: &[u8] /* , tx_buf:&mut LinkedList<Vec<u8>> */) -> Result<(), Error> {
        let (msg, remains) = match Message::from_buf(buf, MessageDirection::FromServer) {
            Ok((msg, remains)) => (msg, remains),
            Err(err) => {
                info!("message parse error: {:?}", err);
                return Err(err);
            }
        };

        debug!("RX: {:?}", msg);

        if let Some(remains) = remains {
            debug!("                 REMAINS {} bytes", remains.len());
        }

        match msg {
            Message::S_SESS(sess) => {
                // info!("RX: S_SESS {:?}", sess.err);
                match sess.err {
                    SessError::OK => {}
                    _ => {
                        // TODO return Error::from(SessError)
                        return Err(Error {
                            source: "session error",
                            detail: None,
                        });
                        // XXX ??? should we send CLOSE too ???
                        // ??? or can we re-send our SESS requests in case of BUSY err ?
                    }
                }
                self.remove_from_que(MessageHint::C_SESS);
                Self::start_send_beats();
            }
            Message::C_SESS(_) => {
                info!("     !!! client must not receive C_SESS !!!");
            }
            Message::REL(rel) => {
                // info!("RX: REL {}", rel.seq);
                if rel.seq == self.rx_rel_seq {
                    self.dispatch_rel_cache(&rel)?;
                    //TODO shuold we clean up cache here (remove RELs that in the past) ?
                } else if rel.seq.wrapping_sub(self.rx_rel_seq) < u16::MAX/2 {
                    // future REL
                    self.cache_rel(rel);
                } else {
                    // past REL
                    info!("past");
                    // TODO self.ack(seq);
                    let last_acked_seq = self.rx_rel_seq - 1;
                    self.enqueue_to_send(Message::ACK(Ack { seq: last_acked_seq }))?;
                }
            }
            Message::ACK(ack) => {
                // info!("RX: ACK {}", ack.seq);
                // info!("our rel {} acked", self.seq);
                self.remove_from_que(MessageHint::REL(ack.seq));
            }
            Message::BEAT => {
                info!("     !!! client must not receive BEAT !!!");
            }
            Message::MAPREQ(_) => {
                info!("     !!! client must not receive MAPREQ !!!");
            }
            Message::MAPDATA(mapdata) => {
                // info!("RX: MAPDATA {:?}", mapdata);
                let pktid = mapdata.pktid;
                self.map.append(mapdata);
                if self.map.complete(pktid) {
                    // TODO let map = self.mapdata.assemble(pktid).to_map();
                    let map_buf = self.map.assemble(pktid);
                    let map = Map::from_buf(map_buf);
                    assert!(map.tiles.len() == 10_000);
                    assert!(map.z.len() == 10_000);
                    info!("MAP COMPLETE ({},{}) name='{}' id={}", map.x, map.y, map.name, map.id);
                    self.remove_from_que(MessageHint::MAPREQ(map.x, map.y));
                    // FIXME TODO update grid only if new grid id != cached grid id
                    match self.map.grids.get(&(map.x, map.y)) {
                        Some(_) => info!("MAP DUPLICATE"),
                        None => {
                            self.events.push_front(Event::Grid(map.x, map.y, map.tiles, map.z));
                            self.map.grids.insert((map.x, map.y), (map.name, map.id));
                        }
                    }
                }
            }
            Message::OBJDATA(objdata) => {
                // info!("RX: OBJDATA {:?}", objdata);
                self.enqueue_to_send(Message::OBJACK(ObjAck::from_objdata(&objdata)))?; // send OBJACKs
                for o in &objdata.obj {
                    // FIXME ??? do NOT add hero object
                    // TODO  if o.id == self.hero.id {
                    //          ... do something with hero, not in objects ...
                    //          if odMOVE {
                    //              if hero.grid.is_changed() {
                    //                  self.request_grids_around();
                    //              }
                    //          }
                    //      }

                    match ObjProp::from_obj_data_elem_prop(&o.prop) {
                        Some(new_obj_prop) => {
                            // TODO use Entry API:
                            // let obj = match self.objects.entry(o.id) {
                            //   Occupied(obj) { if obj.frame > o.frame { obj.update(o); } }
                            //   Vacant(obj) { obj.insert(Obj::new(o.id, None, None, None, None)); }
                            // }
                            let obj = self.objects
                                .entry(o.id)
                                .or_insert(Obj::new(o.id, None, None, None, None));

                            // FIXME consider o.frame overflow !!!
                            if let Some(frame) = obj.frame {
                                if o.frame <= frame {
                                    continue;
                                }
                            }

                            obj.frame = Some(o.frame);
                            obj.update(&new_obj_prop);

                            if let Some(xy) = obj.xy {
                                self.events.push_front(Event::Obj(xy));

                                if let Some(_) = self.hero.obj {
                                    // TODO request_any_new_grids()
                                }
                            }

                            info!("OBJ: {:?}", obj);
                        }
                        None => {
                            self.objects.remove(&o.id);
                            // TODO send Event::ObjRemove(id)

                            info!("OBJ: {} removed", o.id);
                        }
                    }
                }
            }
            Message::OBJACK(_) => {}
            Message::CLOSE => {
                // info!("RX: CLOSE");
                // TODO return Status::EndOfSession instead of Error
                return Err(Error {
                    source: "session closed",
                    detail: None,
                });
            }
        }

        // TODO return Status::Continue/AllOk instead of ()
        Ok(())
    }

    //TODO add struct Rel { ... } and move this to self.rel.cache(rel)
    fn cache_rel(&mut self, rel: Rel) {
        info!("cache REL {}-{}", rel.seq, rel.seq + ((rel.rel.len() as u16) - 1));
        self.rel_cache.insert(rel.seq, rel);
    }

    //TODO add struct Rel { ... } and move this to self.rel.dispatch_cache(rel)
    fn dispatch_rel_cache(&mut self, rel: &Rel) -> Result<(), Error> {
        // XXX FIXME do we handle seq right in the case of overflow ???
        //           to do refactor this code and replace add with wrapping_add
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
        self.enqueue_to_send(Message::ACK(Ack { seq: next_rel_seq }))?;
        self.rx_rel_seq = next_rel_seq + 1;
        Ok(())
    }

    //TODO add struct Rel { ... } and move this to self.rel.dispatch(rel)
    fn dispatch_rel(&mut self, rel: &Rel) {
        info!("dispatch REL {}-{}", rel.seq, rel.seq + ((rel.rel.len() as u16) - 1));
        // info!("RX: {:?}", rel);
        for r in &rel.rel {
            match *r {
                RelElem::NEWWDG(ref wdg) => {
                    // info!("      {:?}", wdg);
                    self.dispatch_newwdg(wdg);
                }
                RelElem::WDGMSG(ref msg) => {
                    // info!("      {:?}", msg);
                    self.dispatch_wdgmsg(msg);
                }
                RelElem::DSTWDG(ref wdg) => {
                    // info!("      {:?}", wdg);
                    self.widgets.remove(&wdg.id);
                }
                RelElem::MAPIV(_) => {}
                RelElem::GLOBLOB(_) => {}
                RelElem::PAGINAE(_) => {}
                RelElem::RESID(ref res) => {
                    // info!("      {:?}", res);
                    self.resources.insert(res.id, res.name.clone() /* FIXME String -> &str */);
                }
                RelElem::PARTY(_) => {}
                RelElem::SFX(_) => {}
                RelElem::CATTR(_) => {}
                RelElem::MUSIC(_) => {}
                RelElem::TILES(_) => {}
                RelElem::BUFF(_) => {}
                RelElem::SESSKEY(_) => {}
            }
        }
    }

    fn dispatch_newwdg(&mut self, wdg: &NewWdg) {
        self.widgets.insert(wdg.id,
                            Widget {
                                id: wdg.id,
                                typ: wdg.name.clone(),
                                parent: wdg.parent,
                                name: None,
                            });
        match wdg.name.as_str() {
            "gameui" => {
                if let Some(&MsgList::tSTR(ref name)) = wdg.cargs.get(0) {
                    self.hero.name = Some(name.clone());
                    info!("HERO: name = '{:?}'", self.hero.name);
                }
                if let Some(&MsgList::tINT(obj)) = wdg.cargs.get(1) {
                    // FIXME BUG: object ID is uint32 but here it is int32 WHY??? XXX
                    assert!(obj >= 0);
                    self.hero.obj = Some(obj as u32);
                    info!("HERO: obj = '{:?}'", self.hero.obj);

                    self.hero.start_xy = match self.hero_xy() {
                        Some(xy) => Some(xy),
                        None => panic!("we have received hero object ID, but hero XY is None"),
                    };

                    self.update_grids_around();
                }
            }
            "mapview" => {
                if let Some(&MsgList::tCOORD(xy)) = wdg.cargs.get(0) {
                    self.origin = Some(xy);
                    info!("origin = '{:?}'", self.origin);
                }
            }
            "item" => {
                if let Some(parent) = self.widgets.get(&(wdg.parent)) {
                    match &*parent.typ {
                        "inv" => {
                            if let Some(&MsgList::tCOORD((x, y))) = wdg.pargs.get(0) {
                                if let Some(&MsgList::tUINT16(id)) = wdg.cargs.get(0) {
                                    self.hero.inventory.insert((x, y), id);
                                    info!("HERO: inventory: {:?}", self.hero.inventory);
                                }
                            }
                        }
                        "epry" => {
                            if let Some(&MsgList::tUINT8(i)) = wdg.pargs.get(0) {
                                if let Some(&MsgList::tUINT16(id)) = wdg.cargs.get(0) {
                                    self.hero.equipment.insert(i, id);
                                    info!("HERO: equipment: {:?}", self.hero.equipment);
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }

    fn dispatch_wdgmsg(&mut self, msg: &WdgMsg) {
        if let Some(w) = self.widgets.get(&(msg.id)) {
            match w.typ.as_str() {
                "charlist" => {
                    if msg.name == "add" {
                        if let Some(&MsgList::tSTR(ref name)) = msg.args.get(0) {
                            info!("    add char '{}'", name);
                            // FIXME rewrite without cloning
                            self.charlist.push(name.clone());
                        }
                    }
                }
                "gameui" => {
                    if msg.name == "weight" {
                        if let Some(&MsgList::tUINT16(w)) = msg.args.get(0) {
                            self.hero.weight = Some(w);
                            info!("HERO: weight = '{:?}'", self.hero.weight);
                        }
                    }
                }
                "chr" => {
                    if msg.name == "tmexp" {
                        if let Some(&MsgList::tINT(i)) = msg.args.get(0) {
                            self.hero.tmexp = Some(i);
                            info!("HERO: tmexp = '{:?}'", self.hero.tmexp);
                        }
                    }
                }
                "ui/hrtptr:11" => {
                    if msg.name == "upd" {
                        if let Some(&MsgList::tCOORD((x, y))) = msg.args.get(0) {
                            // self.objects.insert(0xffffffff, Obj{resid:0xffff, x:x, y:y});
                            self.hero.hearthfire = Some((x, y));
                            info!("HERO: heathfire = '{:?}'", self.hero.hearthfire);
                        }
                    }
                }
                _ => {}
            }
        }
    }

    fn update_grids_around(&mut self) {
        // TODO move to fn client.update_grids_around(...) { ... }
        //     if client.hero.current_grid_is_changed() { client.update_grids_around(); }
        // TODO if grids.not_contains(xy) and requests.not_contains(xy) then add_map_request(xy)
        match self.hero_grid_xy() {
            Some((x, y)) => {
                self.mapreq(x, y).unwrap();
                self.mapreq(x - 1, y).unwrap();
                self.mapreq(x + 1, y).unwrap();

                self.mapreq(x - 1, y - 1).unwrap();
                self.mapreq(x, y - 1).unwrap();
                self.mapreq(x + 1, y - 1).unwrap();

                self.mapreq(x - 1, y + 1).unwrap();
                self.mapreq(x, y + 1).unwrap();
                self.mapreq(x + 1, y + 1).unwrap();
            }
            None => panic!("update_grids_around when hero_grid_xy is None"),
        }
    }

    fn remove_from_que(&mut self, msg_hint: MessageHint) {
        let mut should_be_removed = false;
        if let Some(ref emsg) = self.que.back() {
            if emsg.msg_hint == msg_hint {
                should_be_removed = true;
            }
        }
        if should_be_removed {
            self.que.pop_back();
            match self.que.back() {
                Some(buf) => {
                    // info!("enqueue next packet");
                    self.tx_buf.push_front(buf.clone());
                }
                None => {
                    // info!("remove_from_que: empty que");
                }
            }
        }
    }

    pub fn widget_id(&self, typ: &str, name: Option<String>) -> Option<u16> {
        for (id, w) in &self.widgets {
            if (w.typ == typ) && (w.name == name) {
                return Some(*id);
            }
        }
        None
    }

    pub fn widget_exists(&self, typ: &str, name: Option<String>) -> bool {
        match self.widget_id(typ, name) {
            Some(_) => true,
            None => false,
        }
    }

    pub fn connect(&mut self, login: &str, cookie: &[u8]) -> Result<(), Error> {
        // TODO send SESS until reply
        // TODO get username from server responce, not from auth username
        // let cookie = self.cookie.clone();
        // let user = self.user.clone();
        self.enqueue_to_send(Message::C_SESS(cSess {
                login: login.to_owned(),
                cookie: cookie.to_vec(),
            }))?;
        Ok(())
    }

    pub fn send_play(&mut self, i: usize) -> Result<(), Error> {
        let id = self.widget_id("charlist", None).expect("charlist widget is not found");
        let name = "play".to_owned();
        let charname = self.charlist[i].clone();
        info!("send play '{}'", charname);
        let mut args: Vec<MsgList> = Vec::new();
        args.push(MsgList::tSTR(charname));
        // TODO rel.append(RelElem::new())
        let elem = RelElem::WDGMSG(WdgMsg {
            id: id,
            name: name,
            args: args,
        });
        let mut rel = Rel::new(0);
        rel.append(elem);
        self.enqueue_to_send(Message::REL(rel))
    }

    pub fn mapreq(&mut self, x: i32, y: i32) -> Result<(), Error> {
        // TODO replace with client.send(Message::MapReq::new(x,y).to_buf())
        //     or client.send(Message::mapreq(x,y).to_buf())
        // TODO add "force" flag to update this grid forcelly
        if !self.map.grids.contains_key(&(x, y)) {
            self.enqueue_to_send(Message::MAPREQ(MapReq { x: x, y: y }))?;
        }
        Ok(())
    }

    pub fn rx(&mut self, buf: &[u8]) -> Result<(), Error> {
        self.dispatch_message(buf)
    }

    pub fn timeout(&mut self, seq: usize) {
        match self.que.back() {
            Some(ref mut buf) => {
                match buf.timeout {
                    Some(ref timeout) => {
                        if timeout.seq == seq {
                            // info!("timeout {}: re-enqueue", seq);
                            self.tx_buf.push_front(buf.clone());
                        } else {
                            // info!("timeout {}: packet dropped", seq);
                        }
                    }
                    None => {
                        info!("ERROR: enqueued packet without timeout");
                    }
                }
            }
            None => {
                // info!("timeout {}: empty que", seq);
            }
        }
    }

    pub fn tx(&mut self) -> Option<EnqueuedBuffer> {
        let buf = self.tx_buf.pop_back();
        if let Some(ref buf) = buf {
            match Message::from_buf(buf.buf.as_slice(), MessageDirection::FromClient) {
                Ok((msg, _)) => info!("TX: {:?}", msg),
                Err(e) => panic!("ERROR: malformed TX message: {:?}", e),
            }
        }
        buf
    }

    pub fn close(&mut self) -> Result<(), Error> {
        self.enqueue_to_send(Message::CLOSE)?;
        Ok(())
    }

    // pub fn ready_to_go (&self) -> bool {
    //     let mut ret = false;
    //     for name in self.widgets.values() {
    //         if name == "mapview" {
    //             ret = true;
    //             break;
    //         }
    //     }
    //     return ret;
    // }

    pub fn go(&mut self, x: i32, y: i32) -> Result<(), Error> /*TODO Option<Error>*/ {
        info!("GO");
        let id = self.widget_id("mapview", None).expect("mapview widget is not found");
        let name: String = "click".to_owned();
        let mut args: Vec<MsgList> = Vec::new();
        args.push(MsgList::tCOORD((907, 755))); //TODO set some random coords in the center of screen
        args.push(MsgList::tCOORD((x, y)));
        args.push(MsgList::tINT(1));
        args.push(MsgList::tINT(0));
        let elem = RelElem::WDGMSG(WdgMsg {
            id: id,
            name: name,
            args: args,
        });
        let mut rel = Rel::new(0);
        rel.append(elem);
        self.enqueue_to_send(Message::REL(rel))?;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn pick(&mut self, obj_id: u32) -> Result<(), Error> {
        info!("PICK");
        let id = self.widget_id("mapview", None).expect("mapview widget is not found");
        let name = "click".to_owned();
        let mut args = Vec::new();
        let (obj_x, obj_y) = {
            match self.objects.get(&obj_id) {
                Some(obj) => {
                    match obj.xy {
                        Some(xy) => xy,
                        None => panic!("pick(): picking object has no XY"),
                    }
                }
                None => panic!("pick(): picking object is not found"),
            }
        };
        args.push(MsgList::tCOORD((863, 832))); //TODO set some random coords in the center of screen
        args.push(MsgList::tCOORD((obj_x - 1, obj_y + 1)));
        args.push(MsgList::tINT(3));
        args.push(MsgList::tINT(0));
        args.push(MsgList::tINT(0));
        args.push(MsgList::tINT(obj_id as i32));
        args.push(MsgList::tCOORD((obj_x, obj_y)));
        args.push(MsgList::tINT(0));
        args.push(MsgList::tINT(-1));
        let elem = RelElem::WDGMSG(WdgMsg {
            id: id,
            name: name,
            args: args,
        });
        let mut rel = Rel::new(0);
        rel.append(elem);
        self.enqueue_to_send(Message::REL(rel))?;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn choose_pick(&mut self, wdg_id: u16) -> Result<(), Error> {
        info!("GO");
        let name = "cl".to_owned();
        let mut args = Vec::new();
        args.push(MsgList::tINT(0));
        args.push(MsgList::tINT(0));
        let elem = RelElem::WDGMSG(WdgMsg {
            id: wdg_id,
            name: name,
            args: args,
        });
        let mut rel = Rel::new(0);
        rel.append(elem);
        self.enqueue_to_send(Message::REL(rel))?;
        Ok(())
    }

    // TODO fn grid(Coord) {...}, fn xy(Grid) {...}
    //     and then we can do: hero.grid().xy();

    pub fn hero_obj(&self) -> Option<&Obj> {
        match self.hero.obj {
            Some(id) => self.objects.get(&id),
            None => None,
        }
    }

    pub fn hero_xy(&self) -> Option<Coord> {
        match self.hero_obj() {
            Some(hero) => hero.xy,
            None => None,
        }
    }

    pub fn hero_grid_xy(&self) -> Option<Coord> {
        match self.hero_xy() {
            Some(xy) => Some(grid(xy)),
            None => None,
        }
    }

    pub fn hero_grid(&self) -> Option<&(String, i64)> {
        match self.hero_grid_xy() {
            Some(xy) => self.map.grids.get(&xy),
            None => None,
        }
    }

    pub fn hero_exists(&self) -> bool {
        match self.hero_obj() {
            Some(_) => true,
            None => false,
        }
    }

    pub fn hero_grid_exists(&self) -> bool {
        match self.hero_grid() {
            Some(_) => true,
            None => false,
        }
    }

    pub fn hero_movement(&self) -> Option<Movement> {
        match self.hero_obj() {
            Some(hero) => hero.movement,
            None => None,
        }
    }

    pub fn hero_is_moving(&self) -> bool {
        match self.hero_movement() {
            Some(_) => true,
            None => false,
        }
    }

    #[allow(dead_code)]
    pub fn start_point(&self) -> Option<Coord> {
        self.hero.start_xy
    }

    pub fn next_event(&mut self) -> Option<Event> {
        self.events.pop_back()
    }
}

pub fn grid((x, y): Coord) -> Coord {
    let mut gx = x / 1100;
    if x < 0 {
        gx -= 1;
    }
    let mut gy = y / 1100;
    if y < 0 {
        gy -= 1;
    }
    (gx, gy)
}

// CLIENT
//  REL  seq=4
//   WDGMSG len=65
//    id=6 name=click
//      COORD : [907, 755]        Coord pc
//      COORD : [39683, 36377]    Coord mc
//      INT : 1                   int clickb
//      INT : 0                   ui.modflags()
//      INT : 0                   inf.ol != null
//      INT : 325183464           (int)inf.gob.id
//      COORD : [39737, 36437]    inf.gob.rc
//      INT : 0                   inf.ol.id
//      INT : -1                  inf.r.id or -1
//
// CLIENT
//  REL  seq=5
//   WDGMSG len=36
//    id=6 name=click
//      COORD : [1019, 759]        Coord pc
//      COORD : [39709, 36386]     Coord mc
//      INT : 1                    int clickb
//      INT : 0                    ui.modflags()
//
// private class Click extends Hittest {
//     int clickb;
//
//     private Click(Coord c, int b) {
//         super(c);
//         clickb = b;
//     }
//
//     protected void hit(Coord pc, Coord mc, ClickInfo inf) {
//         if(inf == null) {
//             wdgmsg("click", pc, mc, clickb, ui.modflags());
//         } else {
//             if(inf.ol == null) {
//                 wdgmsg("click", pc, mc, clickb, ui.modflags(), 0, (int)inf.gob.id, inf.gob.rc, 0, getid(inf.r));
//             } else {
//                 wdgmsg("click", pc, mc, clickb, ui.modflags(), 1, (int)inf.gob.id, inf.gob.rc, inf.ol.id, getid(inf.r));
//             }
//         }
//     }
// }
//

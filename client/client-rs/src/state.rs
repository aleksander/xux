use std::collections::{HashMap, LinkedList, BTreeSet};

use std::vec::Vec;
use std::io::Cursor;
use std::io::Read;
use std::u16;

use proto::*;
use errors::*;

extern crate flate2;
use self::flate2::read::ZlibDecoder;

pub type ObjID = u32;
pub type ResID = u16;
pub type Coord = (i32, i32);

struct ObjProp {
    frame: i32,
    xy: Option<Coord>,
    resid: Option<ResID>, // TODO replace with Vec<resid> for composite objects
    line: Option<(Coord, Coord, i32)>, // TODO replace with struct LinearMovement
    step: Option<i32>,
}

impl ObjProp {
    fn new(frame: i32) -> Self {
        ObjProp {
            frame: frame,
            xy: None,
            resid: None,
            line: None,
            step: None,
        }
    }

    fn from_obj_data_elem(ode: &ObjDataElem) -> Option<Self> {
        let mut prop = Self::new(ode.frame);
        for p in &ode.prop {
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
    pub id: ObjID, // TODO maybe remove this? because this is also a key field in objects hashmap
    pub frame: Option<i32>,
    pub resid: Option<ResID>,
    pub xy: Option<Coord>,
    pub movement: Option<Movement>,
}

impl Obj {
    fn new(id: ObjID, frame: Option<i32>, resid: Option<ResID>, xy: Option<Coord>, movement: Option<Movement>) -> Obj {
        Obj {
            id: id,
            frame: frame,
            resid: resid,
            xy: xy,
            movement: movement,
        }
    }

    fn update(&mut self, prop: &ObjProp) -> bool {

        // FIXME consider o.frame overflow !!!
        if let Some(frame) = self.frame {
            if frame >= prop.frame {
                return false;
            }
        }

        self.frame = Some(prop.frame);

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

        true
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
    SESS,
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
    pub id: Option<ObjID>,
    pub obj: Option<Obj>,
    pub weight: Option<u16>,
    pub tmexp: Option<i32>,
    pub hearthfire: Option<Coord>,
    pub inventory: HashMap<Coord, u16>,
    pub equipment: HashMap<u8, u16>,
    //pub start_xy: Option<Coord>,
}

pub struct MapPieces {
    total_len: u16,
    pieces: HashMap<u16, Vec<u8>>,
}

//TODO rename to Grid
pub struct Surface {
    pub x: i32,
    pub y: i32,
    pub name: String,
    pub id: i64,
    pub tiles: Vec<u8>,
    pub z: Vec<i16>,
    pub ol: Vec<u8>,
}

impl Surface {
    fn from_buf <R:ReadBytesSac> (r: &mut R) -> Result<Surface> {
        //let mut r = Cursor::new(buf);
        let x = r.i32().chain_err(||"x")?;
        let y = r.i32().chain_err(||"y")?;
        let mmname = r.strz()?;
        let mut pfl = vec![0; 256];
        loop {
            let pidx = r.u8().chain_err(||"pidx")?;
            if pidx == 255 {
                break;
            }
            pfl[pidx as usize] = r.u8().chain_err(||"pfl[pidx]")?;
        }
        let mut decoder = ZlibDecoder::new(r);
        let mut unzipped = Vec::new();
        let _unzipped_len = decoder.read_to_end(&mut unzipped).chain_err(||"decoder.read_to_end")?;
        // TODO check unzipped_len
        let mut r = Cursor::new(unzipped);

        Self::surface(&mut r, &pfl, x, y, mmname)
    }

    fn surface <R:ReadBytesSac> (r: &mut R, pfl: &[u8], x: i32, y: i32, name: String) -> Result<Surface> {
        let id = r.i64()?;
        let tiles = (0..100*100).map(|_|r.u8()).collect::<Result<Vec<u8>>>()?;
        let z = (0..100*100).map(|_|r.i16()).collect::<Result<Vec<i16>>>()?;

        let mut ol = vec![0; 100*100];
        loop {
            let pidx = r.u8()?;
            if pidx == 255 { break; }
            let fl = pfl[pidx as usize];
            let typ = r.u8()?;
            let (x1,y1) = (r.u8()? as usize, r.u8()? as usize);
            let (x2,y2) = (r.u8()? as usize, r.u8()? as usize);
            //info!("#### {} ({},{}) - ({},{})", typ, x1, y1, x2, y2);
            let oli = match typ {
                0 => if (fl & 1) == 1 { 2 } else { 1 },
                1 => if (fl & 1) == 1 { 8 } else { 4 },
                2 => 16,
                _ => { return Err(format!("ERROR: unknown plot type {}", typ).into()); }
            };
            for y in y1..y2+1 {
                for x in x1..x2+1 {
                    ol[y*100+x] |= oli;
                }
            }
        }

        Ok(Surface {
            x: x,
            y: y,
            name: name,
            id: id,
            tiles: tiles,
            z: z,
            ol: ol
        })
    }
}

pub type PacketId = i32;

pub struct Map {
    pub partial: HashMap<PacketId, MapPieces>, // TODO somehow clean up from old pieces (periodically maybe)
    pub grids: HashMap<Coord, Surface>,
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
}

pub enum Event {
    Grid(Coord),
    Obj(ObjID, Coord, ResID),
    ObjRemove(ObjID),
    Res(ResID, String),
    Hero,
}

pub struct State {
    // TODO do all fields PRIVATE and use callback interface
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
    requested_grids: BTreeSet<(i32, i32)>
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
            }
        );

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
                id: None,
                obj: None,
                weight: None,
                tmexp: None,
                hearthfire: None,
                inventory: HashMap::new(),
                equipment: HashMap::new(),
                //start_xy: None,
            },
            map: Map {
                partial: HashMap::new(),
                grids: HashMap::new(),
            },
            events: LinkedList::new(),
            origin: None,
            requested_grids: BTreeSet::new(),
        }
    }

    pub fn start_send_beats() {
        // TODO
    }

    pub fn enqueue_to_send(&mut self, mut msg: ClientMessage) -> Result<()> {
        if let ClientMessage::REL(ref mut rel) = msg {
            assert!(rel.seq == 0);
            rel.seq = self.seq;
            self.seq += rel.rel.len() as u16;
        }
        let mut buf = vec!();
        match msg.to_buf(&mut buf) {
            Ok(()) => {
                let (msg_hint, timeout) = match msg {
                    ClientMessage::SESS(_) => {
                        (MessageHint::SESS,
                         Some(Timeout {
                            ms: 100,
                            seq: self.enqueue_seq,
                        }))
                    }
                    ClientMessage::REL(rel) => {
                        (MessageHint::REL(rel.seq),
                         Some(Timeout {
                            ms: 100,
                            seq: self.enqueue_seq,
                        }))
                    }
                    ClientMessage::CLOSE(_) => {
                        (MessageHint::CLOSE,
                         Some(Timeout {
                            ms: 100,
                            seq: self.enqueue_seq,
                        }))
                    }
                    ClientMessage::MAPREQ(mapreq) => {
                        (MessageHint::MAPREQ(mapreq.x, mapreq.y),
                         Some(Timeout {
                            ms: 400,
                            seq: self.enqueue_seq,
                        }))
                    }
                    ClientMessage::ACK(_) |
                    ClientMessage::BEAT(_) |
                    ClientMessage::OBJACK(_) => (MessageHint::NONE, None),
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

    pub fn dispatch_message(&mut self, buf: &[u8]) -> Result<()> {
        let mut r = Cursor::new(buf);
        let (msg, remains) = match ServerMessage::from_buf(&mut r) {
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
            ServerMessage::SESS(sess) => {
                // info!("RX: S_SESS {:?}", sess.err);
                match sess.err {
                    SessError::OK => {}
                    _ => {
                        return Err("session error".into());
                        // XXX ??? should we send CLOSE too ???
                        // ??? or can we re-send our SESS requests in case of BUSY err ?
                    }
                }
                self.remove_from_que(MessageHint::SESS);
                Self::start_send_beats();
            }
            ServerMessage::REL(rel) => {
                // info!("RX: REL {}", rel.seq);
                if rel.seq == self.rx_rel_seq {
                    self.dispatch_rel_cache(&rel)?;
                    //TODO should we clean up cache here (remove RELs that in the past) ?
                } else if rel.seq.wrapping_sub(self.rx_rel_seq) < u16::MAX/2 {
                    // future REL
                    self.cache_rel(rel);
                } else {
                    // past REL
                    info!("past");
                    // TODO self.ack(seq);
                    let last_acked_seq = self.rx_rel_seq - 1;
                    self.enqueue_to_send(ClientMessage::ACK(Ack { seq: last_acked_seq }))?;
                }
            }
            ServerMessage::ACK(ack) => {
                // info!("RX: ACK {}", ack.seq);
                // info!("our rel {} acked", self.seq);
                self.remove_from_que(MessageHint::REL(ack.seq));
            }
            ServerMessage::MAPDATA(mapdata) => {
                // info!("RX: MAPDATA {:?}", mapdata);
                let pktid = mapdata.pktid;
                self.map.append(mapdata);
                if self.map.complete(pktid) {
                    // TODO let map = self.mapdata.assemble(pktid).to_map();
                    let map_buf = self.map.assemble(pktid);
                    let map = Surface::from_buf(&mut Cursor::new(map_buf))?;
                    assert!(map.tiles.len() == 10_000);
                    assert!(map.z.len() == 10_000);
                    info!("MAP COMPLETE ({},{}) name='{}' id={}", map.x, map.y, map.name, map.id);
                    self.remove_from_que(MessageHint::MAPREQ(map.x, map.y));
                    // FIXME TODO update grid only if new grid id != cached grid id
                    match self.map.grids.get(&(map.x, map.y)) {
                        Some(_) => info!("MAP DUPLICATE"),
                        None => {
                            self.events.push_front(Event::Grid((map.x, map.y)));
                            self.map.grids.insert((map.x, map.y), map);
                        }
                    }
                }
            }
            ServerMessage::OBJDATA(objdata) => {
                // info!("RX: OBJDATA {:?}", objdata);
                self.enqueue_to_send(ClientMessage::OBJACK(ObjAck::from_objdata(&objdata)))?; // send OBJACKs
                for o in &objdata.obj {

                    match ObjProp::from_obj_data_elem(&o) {
                        Some(ref new_obj_prop) => {

                            if let Some(hero_id) = self.hero.id {
                                if o.id == hero_id {
                                    match self.hero.obj {
                                        Some(ref mut hero_obj) => {
                                            hero_obj.update(new_obj_prop);
                                        }
                                        None => {
                                            if let Some(mut cached_hero_obj) = self.objects.remove(&hero_id) {
                                                cached_hero_obj.update(new_obj_prop);
                                                self.hero.obj = Some(cached_hero_obj);
                                                self.events.push_front(Event::ObjRemove(hero_id));
                                            } else {
                                                let mut hero_obj = Obj::new(o.id, None, None, None, None);
                                                hero_obj.update(new_obj_prop);
                                                self.hero.obj = Some(hero_obj);
                                            }
                                            info!("HERO: obj: {:?}", self.hero.obj);
                                        }
                                    }
                                    self.request_grids_around_hero();
                                    self.events.push_front(Event::Hero);
                                    continue;
                                }
                            }

                            let obj = self.objects
                                .entry(o.id)
                                .or_insert(Obj::new(o.id, None, None, None, None));

                            if obj.update(&new_obj_prop) {
                                if let Some(xy) = obj.xy {
                                    self.events.push_front(Event::Obj(obj.id, xy, obj.resid.unwrap_or(0)));
                                }
                                info!("OBJ: {:?}", obj);
                            }
                        }
                        None => {
                            self.objects.remove(&o.id);
                            self.events.push_front(Event::ObjRemove(o.id));
                            info!("OBJ: {} removed", o.id);
                        }
                    }
                }
            }
            ServerMessage::CLOSE(_) => {
                // info!("RX: CLOSE");
                // TODO return Status::EndOfSession instead of Error
                return Err("session closed".into());
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
    fn dispatch_rel_cache(&mut self, rel: &Rel) -> Result<()> {
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
        self.enqueue_to_send(ClientMessage::ACK(Ack { seq: next_rel_seq }))?;
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
                    self.resources.insert(res.id, res.name.clone());
                    self.events.push_front(Event::Res(res.id, res.name.clone()));
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
                    let id = obj as u32;
                    self.hero.id = Some(id);
                    info!("HERO: id: {:?}", self.hero.id);
                    let cached_hero_obj = self.objects.remove(&id);
                    if cached_hero_obj.is_some() {
                        self.hero.obj = cached_hero_obj;
                        info!("HERO: obj: {:?}", self.hero.obj);
                        self.request_grids_around_hero();
                        self.events.push_front(Event::ObjRemove(id));
                        self.events.push_front(Event::Hero);
                    }

                    //self.hero.start_xy = match self.hero_xy() {
                    //    Some(xy) => Some(xy),
                    //    None => panic!("we have received hero object ID, but hero XY is None"),
                    //};
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
                            self.hero.hearthfire = Some((x, y));
                            info!("HERO: heathfire = '{:?}'", self.hero.hearthfire);
                            //TODO send Event::Hearthfire
                        }
                    }
                }
                _ => {}
            }
        }
    }

    fn request_grids_around_hero (&mut self) {
        // TODO move to fn client.update_grids_around(...) { ... }
        //     if client.hero.current_grid_is_changed() { client.update_grids_around(); }
        match self.hero_grid_xy() {
            Some(xy) => {
                self.request_grids_around(xy);
            }
            None => {}
        }
    }

    fn request_grids_around (&mut self, (x, y): Coord) {
        self.request_grid((x,     y));
        self.request_grid((x - 1, y));
        self.request_grid((x + 1, y));

        self.request_grid((x - 1, y - 1));
        self.request_grid((x,     y - 1));
        self.request_grid((x + 1, y - 1));

        self.request_grid((x - 1, y + 1));
        self.request_grid((x,     y + 1));
        self.request_grid((x + 1, y + 1));
    }

    fn request_grid (&mut self, (x, y): Coord) {
        if ! self.requested_grids.contains(&(x,y)) {
            info!("request grid ({},{})", x, y);
            self.mapreq(x, y).unwrap();
            self.requested_grids.insert((x,y));
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

    pub fn connect(&mut self, login: &str, cookie: &[u8]) -> Result<()> {
        // TODO send SESS until reply
        // TODO get username from server responce, not from auth username
        // let cookie = self.cookie.clone();
        // let user = self.user.clone();
        self.enqueue_to_send(ClientMessage::SESS(cSess {
                login: login.to_owned(),
                cookie: cookie.to_vec(),
            }))?;
        Ok(())
    }

    pub fn send_play(&mut self, i: usize) -> Result<()> {
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
        self.enqueue_to_send(ClientMessage::REL(rel))
    }

    pub fn mapreq(&mut self, x: i32, y: i32) -> Result<()> {
        // TODO replace with client.send(Message::MapReq::new(x,y).to_buf())
        //     or client.send(Message::mapreq(x,y).to_buf())
        // TODO add "force" flag to update this grid forcelly
        if !self.map.grids.contains_key(&(x, y)) {
            self.enqueue_to_send(ClientMessage::MAPREQ(MapReq::new(x, y)))?;
        }
        Ok(())
    }

    pub fn rx(&mut self, buf: &[u8]) -> Result<()> {
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
            let mut r = Cursor::new(buf.buf.as_slice());
            match ClientMessage::from_buf(&mut r/*buf.buf.as_slice()*/) {
                Ok((msg, _)) => info!("TX: {:?}", msg),
                Err(e) => panic!("ERROR: malformed TX message: {:?}", e),
            }
        }
        buf
    }

    pub fn close(&mut self) -> Result<()> {
        self.enqueue_to_send(ClientMessage::CLOSE(Close))
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

    pub fn go(&mut self, x: i32, y: i32) -> Result<()> /*TODO Option<Error>*/ {
        info!("GO {} {}", x, y);
        let id = self.widget_id("mapview", None).ok_or(ErrorKind::Msg("try to go with no mapview widget".into()))?;
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
        self.enqueue_to_send(ClientMessage::REL(rel))?;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn pick(&mut self, obj_id: u32) -> Result<()> {
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
        self.enqueue_to_send(ClientMessage::REL(rel))?;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn choose_pick(&mut self, wdg_id: u16) -> Result<()> {
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
        self.enqueue_to_send(ClientMessage::REL(rel))?;
        Ok(())
    }

    // TODO fn grid(Coord) {...}, fn xy(Grid) {...}
    //     and then we can do: hero.grid().xy();

    pub fn hero_xy(&self) -> Option<Coord> {
        match self.hero.obj {
            Some(ref hero) => hero.xy,
            None => None,
        }
    }

    pub fn hero_grid_xy(&self) -> Option<Coord> {
        match self.hero_xy() {
            Some(xy) => Some(grid(xy)),
            None => None,
        }
    }

    pub fn hero_grid(&self) -> Option<&Surface> {
        match self.hero_grid_xy() {
            Some(xy) => self.map.grids.get(&xy),
            None => None,
        }
    }

    pub fn hero_exists(&self) -> bool {
        match self.hero.obj {
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
        match self.hero.obj {
            Some(ref hero) => hero.movement,
            None => None,
        }
    }

    pub fn hero_is_moving(&self) -> bool {
        match self.hero_movement() {
            Some(_) => true,
            None => false,
        }
    }

    //pub fn start_point(&self) -> Option<Coord> {
    //    self.hero.start_xy
    //}

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

use std::collections::{HashMap, LinkedList, BTreeSet};
use std::vec::Vec;
use std::io::Cursor;
use std::io::Read;
use std::u16;
use proto::*;
use Result;
use failure::err_msg;
use flate2::read::ZlibDecoder;
use std::sync::mpsc;
use driver::Driver;

struct ObjProp {
    frame: i32,
    xy: Option<ObjXY>,
    resid: Option<ResID>, // TODO replace with Vec<resid> for composite objects
    line: Option<Linbeg>,
    step: Option<Linstep>,
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
        for p in ode.prop.iter() {
            match p {
                &ObjDataElemProp::Rem => {
                    return None;
                }
                &ObjDataElemProp::Move(xy, _) => {
                    prop.xy = Some(xy);
                }
                &ObjDataElemProp::Res(resid) => {
                    prop.resid = Some(resid);
                }
                &ObjDataElemProp::Compose(resid) => {
                    prop.resid = Some(resid);
                }
                &ObjDataElemProp::Linbeg(linbeg) => {
                    prop.line = Some(linbeg);
                }
                &ObjDataElemProp::Linstep(linstep) => {
                    prop.step = Some(linstep);
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
    pub xy: Option<ObjXY>,
    pub movement: Option<Movement>,
}

impl Obj {
    fn new(id: ObjID, frame: Option<i32>, resid: Option<ResID>, xy: Option<ObjXY>, movement: Option<Movement>) -> Obj {
        Obj {
            id: id,
            frame: frame,
            resid: resid,
            xy: xy,
            movement: movement,
        }
    }

    #[cfg(feature = "salem")]
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

        if let Some(Linbeg{from, to, steps}) = prop.line {
            self.movement = Some(Movement::new(from, to, steps, 0));
        }

        if let Some(Linstep{ step }) = prop.step {
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

    #[cfg(feature = "hafen")]
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

        //TODO update linbeg
        //TODO update linstep

        true
    }
}

#[derive(Clone,Copy,Debug)]
pub struct Movement {
    pub from: ObjXY,
    pub to: ObjXY,
    pub steps: i32,
    pub step: i32,
}

impl Movement {
    #[cfg(feature = "salem")]
    fn new(from: ObjXY, to: ObjXY, steps: i32, step: i32) -> Movement {
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
    pub hearthfire: Option<(i32,i32)>,
    pub inventory: HashMap<(i32,i32), u16>,
    pub equipment: HashMap<u8, u16>,
    //pub start_xy: Option<ObjXY>,
}

pub struct MapPieces {
    total_len: u16,
    pieces: HashMap<u16, Vec<u8>>,
}

//TODO rename to Grid
//TODO rename to Map
#[derive(Clone)]
pub struct Surface {
    pub x: i32,
    pub y: i32,
    pub name: String,
    pub id: i64,
    #[cfg(feature = "hafen")]
    pub tileres: Vec<Tile>,
    pub tiles: Vec<u8>,
    pub z: Vec<i16>,
    pub ol: Vec<u8>,
}

impl Surface {
    fn from_buf <R:ReadBytesSac> (r: &mut R) -> Result<Surface> {
        //let mut r = Cursor::new(buf);
        let (x,y) = r.coord()?;
        let mmname = r.strz()?;
        let mut pfl = vec![0; 256];
        loop {
            let pidx = r.u8()?;
            if pidx == 255 {
                break;
            }
            pfl[pidx as usize] = r.u8()?;
        }
        let mut decoder = ZlibDecoder::new(r);
        let mut unzipped = Vec::new();
        let _unzipped_len = decoder.read_to_end(&mut unzipped)?;
        // TODO check unzipped_len
        let mut r = Cursor::new(unzipped);

        Self::surface(&mut r, &pfl, x, y, mmname)
    }

    #[cfg(feature = "salem")]
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

    #[cfg(feature = "hafen")]
    fn surface <R:ReadBytesSac> (r: &mut R, pfl: &[u8], x: i32, y: i32, name: String) -> Result<Surface> {
        let id = r.i64()?;

        let mut tileres = Vec::new();
        loop {
            let tileid = r.u8()?;
            if tileid == 255 {
                break;
            }
            let resname = r.strz()?;
            let resver = r.u16()?;
            tileres.push(Tile{id:tileid,name:resname,ver:resver});
        }
        for tile in tileres.iter() {
            debug!("tileres {:5} {} {}", tile.id, tile.name, tile.ver);
        }

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
                2 => if (fl & 1) == 1 { 32 } else { 16 },
                _ => { return Err(format_err!("ERROR: unknown plot type {}", typ)); }
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
            tileres: tileres,
            tiles: tiles,
            z: z,
            ol: ol
        })
    }
}

pub type PacketId = i32;

//TODO rename to PartialMap
pub struct Map {
    pub partial: HashMap<PacketId, MapPieces>, // TODO somehow clean up from old pieces (periodically maybe)
    pub grids: BTreeSet<GridXY>,
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
        }
        buf
    }
}

pub type WdgID = u16;

#[derive(Serialize,Deserialize,Debug,Clone)]
pub enum Wdg {
    New(WdgID,String,WdgID),
    Msg(WdgID,String,Vec<List>),
    Del(WdgID),
}

#[derive(Clone)]
pub enum Event {
    Tiles(Vec<Tile>),
    Grid(Surface),
    Obj(ObjID, ObjXY, ResID),
    ObjRemove(ObjID),
    Res(ResID, String),
    Hero(ObjXY),
    Wdg(Wdg),
    Hearthfire(ObjXY),
}

struct Sender {
    events_tx1: mpsc::Sender<Event>,
    events_tx2: mpsc::Sender<Event>,
}

impl Sender {
    fn send_event (&self, event: Event) -> Result<()> {
        self.events_tx1.send(event.clone())?;
        self.events_tx2.send(event)?;
        Ok(())
    }
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
    pub enqueue_seq: usize,
    pub rel_cache: HashMap<u16, Rels>, //TODO unify with rx_rel_seq to have more consistent entity (struct Rel { ... })
    pub hero: Hero,
    pub map: Map,
    sender: Sender,
    requested_grids: BTreeSet<(i32, i32)>,
    timestamp: String,
    pub login: String,
    driver: Driver,
}

impl State {
    pub fn new(events_tx1: mpsc::Sender<Event>, events_tx2: mpsc::Sender<Event>, driver: Driver) -> State {
        use chrono;

        let mut widgets = HashMap::new();
        widgets.insert(0,
            Widget {
                id: 0,
                typ: "root".into(),
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
            },
            map: Map {
                partial: HashMap::new(),
                grids: BTreeSet::new(),
            },
            sender: Sender {
                events_tx1: events_tx1,
                events_tx2: events_tx2,
            },
            requested_grids: BTreeSet::new(),
            timestamp: chrono::Local::now().format("%Y-%m-%d %H-%M-%S").to_string(),
            login: "".into(),
            driver: driver,
        }
    }

    pub fn start_send_beats() {
        // TODO
    }

    pub fn enqueue_to_send(&mut self, mut msg: ClientMessage) -> Result<()> {
        if let ClientMessage::REL(ref mut rel) = msg {
            assert!(rel.seq == 0);
            rel.seq = self.seq;
            self.seq += rel.rels.len() as u16;
        }
        let mut buf = vec!();
        match msg.to_buf(&mut buf) {
            Ok(()) => {
                let (msg_hint, timeout) = match msg {
                    ClientMessage::SESS(_) => {
                        (MessageHint::SESS,
                         Some(Timeout {
                            ms: 200,
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
                            ms: 200,
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

                if ebuf.timeout.is_some() {
                    if self.que.is_empty() {
                        self.send(ebuf.clone())?;
                    }
                    self.que.push_front(ebuf);
                    self.enqueue_seq += 1;
                } else {
                    self.send(ebuf)?;
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
                        return Err(err_msg("session error"));
                        // XXX ??? should we send CLOSE too ???
                        // ??? or can we re-send our SESS requests in case of BUSY err ?
                    }
                }
                self.remove_from_que(MessageHint::SESS)?;
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
                self.remove_from_que(MessageHint::REL(ack.seq))?;
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
                    self.remove_from_que(MessageHint::MAPREQ(map.x, map.y))?;
                    // FIXME TODO update grid only if new grid id != cached grid id
                    match self.map.grids.get(&(map.x, map.y)) {
                        Some(_) => info!("MAP DUPLICATE"),
                        None => {
                            use util::grid_to_png;
                            self.map.grids.insert((map.x, map.y));
                            let name = if let Some(ref name) = self.hero.name { name } else { "none" };
                            grid_to_png(&self.login, name, &self.timestamp, map.x, map.y, &map.tiles, &map.z)?;
                            #[cfg(feature = "salem")]
                            self.sender.send_event(Event::Grid((map.x, map.y)))?;
                            #[cfg(feature = "hafen")]
                            self.sender.send_event(Event::Tiles(map.tileres.clone()))?;
                            #[cfg(feature = "hafen")]
                            self.sender.send_event(Event::Grid(map))?;
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
                                    if let Some(ref mut hero_obj) = self.hero.obj {
                                        hero_obj.update(new_obj_prop);
                                    } else {
                                        if let Some(mut cached_hero_obj) = self.objects.remove(&hero_id) {
                                            cached_hero_obj.update(new_obj_prop);
                                            self.hero.obj = Some(cached_hero_obj);
                                            self.sender.send_event(Event::ObjRemove(hero_id))?;
                                        } else {
                                            let mut hero_obj = Obj::new(o.id, None, None, None, None);
                                            hero_obj.update(new_obj_prop);
                                            self.hero.obj = Some(hero_obj);
                                        }
                                        info!("HERO: obj: {:?}", self.hero.obj);
                                    }
                                    if let Some(ref obj) = self.hero.obj {
                                        if let Some(xy) = obj.xy {
                                            self.sender.send_event(Event::Hero(xy))?;
                                        } else {
                                            return Err(err_msg("hero without coordinates"));
                                        }
                                    } else {
                                        unreachable!();
                                    }
                                    self.request_grids_around_hero();
                                    continue;
                                }
                            }

                            let obj = self.objects
                                .entry(o.id)
                                .or_insert(Obj::new(o.id, None, None, None, None));

                            if obj.update(&new_obj_prop) {
                                if let Some(xy) = obj.xy {
                                    self.sender.send_event(Event::Obj(obj.id, xy, obj.resid.unwrap_or(0)))?;
                                }
                                info!("OBJ: {:?}", obj);
                            }
                        }
                        None => {
                            self.objects.remove(&o.id);
                            self.sender.send_event(Event::ObjRemove(o.id))?;
                            info!("OBJ: {} removed", o.id);
                        }
                    }
                }
            }
            ServerMessage::CLOSE(_) => {
                // info!("RX: CLOSE");
                // TODO return Status::EndOfSession instead of Error
                return Err(err_msg("session closed"));
            }
        }

        // TODO return Status::Continue/AllOk instead of ()
        Ok(())
    }

    //TODO add struct Rel { ... } and move this to self.rel.cache(rel)
    fn cache_rel(&mut self, rel: Rels) {
        info!("cache REL {}-{}", rel.seq, rel.seq + ((rel.rels.len() as u16) - 1));
        self.rel_cache.insert(rel.seq, rel);
    }

    //TODO add struct Rel { ... } and move this to self.rel.dispatch_cache(rel)
    fn dispatch_rel_cache(&mut self, rel: &Rels) -> Result<()> {
        // XXX FIXME do we handle seq right in the case of overflow ???
        //           to do refactor this code and replace add with wrapping_add
        let mut next_rel_seq = rel.seq + ((rel.rels.len() as u16) - 1);
        self.dispatch_rel(rel)?;
        loop {
            let next_rel = self.rel_cache.remove(&(next_rel_seq + 1));
            match next_rel {
                Some(rel) => {
                    next_rel_seq = rel.seq + ((rel.rels.len() as u16) - 1);
                    self.dispatch_rel(&rel)?;
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
    fn dispatch_rel(&mut self, rel: &Rels) -> Result<()> {
        info!("dispatch REL {}-{}", rel.seq, rel.seq + ((rel.rels.len() as u16) - 1));
        // info!("RX: {:?}", rel);
        for r in &rel.rels {
            match *r {
                Rel::NEWWDG(ref wdg) => {
                    // info!("      {:?}", wdg);
                    self.dispatch_newwdg(wdg)?;
                    self.sender.send_event(Event::Wdg(Wdg::New(wdg.id, wdg.name.clone(), wdg.parent)))?;
                }
                Rel::WDGMSG(ref msg) => {
                    // info!("      {:?}", msg);
                    self.dispatch_wdgmsg(msg)?;
                    self.sender.send_event(Event::Wdg(Wdg::Msg(msg.id, msg.name.clone(), msg.args.clone())))?;
                }
                Rel::DSTWDG(ref wdg) => {
                    // info!("      {:?}", wdg);
                    self.widgets.remove(&wdg.id);
                    self.sender.send_event(Event::Wdg(Wdg::Del(wdg.id)))?;
                }
                Rel::MAPIV(_) => {}
                Rel::GLOBLOB(_) => {}
                Rel::PAGINAE(_) => {}
                Rel::RESID(ref res) => {
                    // info!("      {:?}", res);
                    self.resources.insert(res.id, res.name.clone());
                    self.sender.send_event(Event::Res(res.id, res.name.clone()))?;
                }
                Rel::PARTY(_) => {}
                Rel::SFX(_) => {}
                Rel::CATTR(_) => {}
                Rel::MUSIC(_) => {}
                Rel::TILES(ref tiles) => {
                    self.sender.send_event(Event::Tiles(tiles.tiles.clone()))?;
                }
                Rel::BUFF(_) => {}
                Rel::SESSKEY(_) => {}
                Rel::FRAGMENT(ref fragment) => {
                    info!("      fragment {}", match fragment {
                        &Fragment::Head(_,_) => "head",
                        &Fragment::Middle(_) => "middle",
                        &Fragment::Tail(_) => "head",
                    });
                    panic!("must handle fragments!");
                }
                Rel::ADDWDG(ref wdg) => {
                    info!("      {:?}", wdg);
                }
            }
        }
        Ok(())
    }

    fn dispatch_newwdg(&mut self, wdg: &NewWdg) -> Result<()> {
        self.widgets.insert(wdg.id,
                            Widget {
                                id: wdg.id,
                                typ: wdg.name.clone(),
                                parent: wdg.parent,
                                name: None,
                            });
        match wdg.name.as_str() {
            "gameui" => {
                if let Some(&List::Str(ref name)) = wdg.cargs.get(0) {
                    self.hero.name = Some(name.clone());
                    info!("HERO: name = '{:?}'", self.hero.name);
                }
                if let Some(&List::Int(obj)) = wdg.cargs.get(1) {
                    // FIXME BUG: object ID is uint32 but here it is int32 WHY??? XXX
                    assert!(obj >= 0);
                    let id = obj as u32;
                    self.hero.id = Some(id);
                    info!("HERO: id: {:?}", self.hero.id);
                    let cached_hero_obj = self.objects.remove(&id);
                    match cached_hero_obj {
                        Some(cached_hero_obj) => {
                            self.sender.send_event(Event::ObjRemove(id))?;
                            self.sender.send_event(Event::Hero(cached_hero_obj.xy.unwrap_or(ObjXY::new())))?;
                            self.hero.obj = Some(cached_hero_obj);
                            info!("HERO: obj: {:?}", self.hero.obj);
                            self.request_grids_around_hero();
                        }
                        None => {}
                    }
                }
            }
            "mapview" => {
                if let Some(&List::Coord(xy)) = wdg.cargs.get(0) {
                    info!("origin = '{:?}'", xy);
                }
            }
            "item" => {
                if let Some(parent) = self.widgets.get(&(wdg.parent)) {
                    match &*parent.typ {
                        "inv" => {
                            if let Some(&List::Coord((x, y))) = wdg.pargs.get(0) {
                                if let Some(&List::Uint16(id)) = wdg.cargs.get(0) {
                                    self.hero.inventory.insert((x, y), id);
                                    info!("HERO: inventory: {:?}", self.hero.inventory);
                                }
                            }
                        }
                        "epry" => {
                            if let Some(&List::Uint8(i)) = wdg.pargs.get(0) {
                                if let Some(&List::Uint16(id)) = wdg.cargs.get(0) {
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
        Ok(())
    }

    fn dispatch_wdgmsg(&mut self, msg: &WdgMsg) -> Result<()> {
        if let Some(w) = self.widgets.get(&(msg.id)) {
            match w.typ.as_str() {
                "charlist" => {
                    if msg.name == "add" {
                        if let Some(&List::Str(ref name)) = msg.args.get(0) {
                            info!("    add char '{}'", name);
                            // FIXME rewrite without cloning
                            self.charlist.push(name.clone());
                        }
                    }
                }
                "gameui" => {
                    if msg.name == "weight" {
                        if let Some(&List::Uint16(w)) = msg.args.get(0) {
                            self.hero.weight = Some(w);
                            info!("HERO: weight = '{:?}'", self.hero.weight);
                        }
                    }
                }
                "chr" => {
                    if msg.name == "tmexp" {
                        if let Some(&List::Int(i)) = msg.args.get(0) {
                            self.hero.tmexp = Some(i);
                            info!("HERO: tmexp = '{:?}'", self.hero.tmexp);
                        }
                    }
                }
                "ui/hrtptr:11" => {
                    if msg.name == "upd" {
                        if let Some(&List::Coord(xy)) = msg.args.get(0) {
                            self.hero.hearthfire = Some(xy);
                            info!("HERO: heathfire = '{:?}'", self.hero.hearthfire);
                            self.sender.send_event(Event::Hearthfire(xy.into()))?;
                        }
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }

    fn request_grids_around_hero (&mut self) {
        // TODO move to fn client.update_grids_around(...) { ... }
        //     if client.hero.current_grid_is_changed() { client.update_grids_around(); }
        if let Some(xy) = self.hero_grid_xy() {
            self.request_grids_around(xy);
        }
    }

    fn request_grids_around (&mut self, (x, y): GridXY) {
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

    fn request_grid (&mut self, (x, y): GridXY) {
        if ! self.requested_grids.contains(&(x,y)) {
            info!("request grid ({},{})", x, y);
            self.mapreq(x, y).unwrap();
            self.requested_grids.insert((x,y));
        }
    }

    fn remove_from_que(&mut self, msg_hint: MessageHint) -> Result<()> {
        if let Some(msg) = self.que.pop_back() {
            if msg.msg_hint == msg_hint {
                if let Some(buf) = self.que.pop_back() {
                    // info!("enqueue next packet");
                    self.send(buf.clone())?;
                    self.que.push_back(buf);
                } else {
                    // info!("remove_from_que: empty que");
                }
            } else {
                self.que.push_back(msg);
            }
        }
        Ok(())
    }

    pub fn widget_id(&self, typ: &str, name: Option<String>) -> Option<u16> {
        for (id, w) in &self.widgets {
            if (w.typ == typ) && (w.name == name) {
                return Some(*id);
            }
        }
        None
    }

    pub fn connect(&mut self, login: &str, cookie: &[u8]) -> Result<()> {
        // TODO get username from server responce, not from auth username
        self.enqueue_to_send(ClientMessage::SESS(cSess::new(login.to_owned(), cookie.to_vec())))?;
        Ok(())
    }

    pub fn mapreq(&mut self, x: i32, y: i32) -> Result<()> {
        // TODO replace with client.send(Message::MapReq::new(x,y).to_buf())
        //     or client.send(Message::mapreq(x,y).to_buf())
        // TODO add "force" flag to update this grid forcelly
        if !self.map.grids.contains(&(x, y)) {
            self.enqueue_to_send(ClientMessage::MAPREQ(MapReq::new(x, y)))?;
        }
        Ok(())
    }

    pub fn rx(&mut self, buf: &[u8]) -> Result<()> {
        self.dispatch_message(buf)
    }

    pub fn timeout(&mut self, seq: usize) -> Result<()> {
        if let Some(buf) = self.que.pop_back() {
            if let Some(ref timeout) = buf.timeout {
                if timeout.seq == seq {
                    // info!("timeout {}: re-enqueue", seq);
                    self.send(buf.clone())?;
                } else {
                    // info!("timeout {}: packet dropped", seq);
                }
            }
            self.que.push_back(buf);
        } else {
            // info!("timeout {}: empty que", seq);
        }
        Ok(())
    }

    pub fn close(&mut self) -> Result<()> {
        self.enqueue_to_send(ClientMessage::CLOSE(Close))
    }

    pub fn go(&mut self, xy: ObjXY) -> Result<()> {
        use failure::err_msg;
        info!("GO ({}, {})", xy.0, xy.1);
        let id = self.widget_id("mapview", None).ok_or(err_msg("try to go with no mapview widget"))?;
        let name = "click".to_string();
        let mut args: Vec<List> = Vec::new();
        args.push(List::Coord((907, 755))); //TODO set some random coords in the center of screen
        args.push(List::Coord(xy.into()));
        args.push(List::Int(1));
        args.push(List::Int(0));
        let mut rels = Rels::new(0);
        rels.append(Rel::WDGMSG(WdgMsg::new(id, name, args)));
        self.enqueue_to_send(ClientMessage::REL(rels))?;
        Ok(())
    }

    pub fn hero_xy(&self) -> Option<ObjXY> {
        match self.hero.obj {
            Some(ref hero) => hero.xy,
            None => None,
        }
    }

    pub fn hero_grid_xy(&self) -> Option<GridXY> {
        match self.hero_xy() {
            Some(xy) => Some(xy.grid()),
            None => None,
        }
    }

    fn dispatch_single_event(&mut self) -> Result<()> {
        use driver;
        use proto::ObjXY;

        let event = self.driver.next_event()?;
        match event {
            driver::Event::Rx(buf) => {
                // info!("event::rx: {} bytes", buf.len());
                self.rx(&buf)?;
            }
            driver::Event::Timeout(seq) => {
                // info!("event::timeout: {} seq", seq);
                self.timeout(seq)?;
            }
            #[cfg(feature = "salem")]
            driver::Event::User(u) => {
                use driver::UserInput::*;
                match u {
                    Up    => if let Some(ObjXY(x,y)) = self.hero_xy() {
                        self.go(ObjXY(x,y+100))?;
                    },
                    Down  => if let Some(ObjXY(x,y)) = self.hero_xy() {
                        self.go(ObjXY(x,y-100))?;
                    },
                    Left  => if let Some(ObjXY(x,y)) = self.hero_xy() {
                        self.go(ObjXY(x-100,y))?;
                    },
                    Right => if let Some(ObjXY(x,y)) = self.hero_xy() {
                        self.go(ObjXY(x+100,y))?;
                    },
                    Quit  => self.close()?,
                }
            }
            #[cfg(feature = "hafen")]
            driver::Event::User(u) => {
                use driver::UserInput::*;
                //info!("event: {:?}", u);
                match u {
                    Up    => if let Some(ObjXY(x,y)) = self.hero_xy() {
                        self.go(ObjXY(x,y+100.0))?;
                    },
                    Down  => if let Some(ObjXY(x,y)) = self.hero_xy() {
                        self.go(ObjXY(x,y-100.0))?;
                    },
                    Left  => if let Some(ObjXY(x,y)) = self.hero_xy() {
                        self.go(ObjXY(x-100.0,y))?;
                    },
                    Right => if let Some(ObjXY(x,y)) = self.hero_xy() {
                        self.go(ObjXY(x+100.0,y))?;
                    },
                    Quit  => {
                        self.close()?;
                    }
                    Message(id, name, args) => {
                        let mut rels = Rels::new(0);
                        rels.append(Rel::WDGMSG(WdgMsg::new(id, name, args)));
                        self.enqueue_to_send(ClientMessage::REL(rels))?;
                    }
                }
            }
        }
        Ok(())
    }

    pub fn send (&mut self, buf: EnqueuedBuffer) -> Result<()> {
        let mut r = Cursor::new(buf.buf.as_slice());
        match ClientMessage::from_buf(&mut r) {
            Ok((msg, _)) => info!("TX: {:?}", msg),
            Err(e) => return Err(format_err!("ERROR: malformed TX message: {:?}", e)),
        }
        self.driver.transmit(&buf.buf)?;
        if let Some(timeout) = buf.timeout {
            self.driver.add_timeout(timeout.seq, timeout.ms);
        }
        Ok(())
    }

    pub fn run (&mut self, login: &str, cookie: &[u8]) -> Result<()> {
        info!("connect {} / {}", login, cookie.iter().fold(String::new(), |s,b|format!("{}{:02x}",s,b)));
        self.login = login.into();
        self.connect(login, cookie)?;
        loop {
            self.dispatch_single_event()?;
        }
    }
}

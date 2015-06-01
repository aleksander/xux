use std::net::IpAddr;
use std::net::Ipv4Addr;
use std::collections::hash_map::HashMap;
use std::collections::hash_set::HashSet;
use std::collections::LinkedList;
use std::net::TcpStream;
use std::net::SocketAddr;

extern crate openssl;
use self::openssl::crypto::hash::Type;
use self::openssl::crypto::hash::hash;
use self::openssl::ssl::{SslMethod, SslContext, SslStream};

use std::vec::Vec;
use std::io::Cursor;
use std::io::Read;
use std::io::Write;
use std::str;

extern crate byteorder;
use self::byteorder::{LittleEndian, BigEndian, ReadBytesExt, WriteBytesExt};
#[allow(non_camel_case_types)]
type le = LittleEndian;
#[allow(non_camel_case_types)]
type be = BigEndian;

use salem::message::*;

extern crate rustc_serialize;
use self::rustc_serialize::hex::ToHex;

pub struct Obj {
    pub resid : u16,
    pub x : i32,
    pub y : i32,
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
    pub name : String,
    pub parent : u16,
}
    
pub struct Client {
    //TODO do all fileds PRIVATE and use callback interface
    pub serv_ip    : IpAddr,
    pub user       : String,
    pub pass       : String,
    pub cookie     : Vec<u8>,
    pub widgets    : HashMap<u16,Widget>,
    pub objects    : HashMap<u32,Obj>,
    pub grids      : HashSet<(i32,i32)>,
    pub charlist   : Vec<String>,
    pub resources  : HashMap<u16,String>,
    pub seq        : u16,
    pub rx_rel_seq : u16,
    pub que        : LinkedList<EnqueuedBuffer>,
    pub tx_buf     : LinkedList<EnqueuedBuffer>,
    pub enqueue_seq    : usize,
    pub rel_cache  : HashMap<u16,Rel>,
}

impl Client {
    pub fn new (user: String, pass: String) -> Client {
        let mut widgets = HashMap::new();
        widgets.insert(0, Widget{ id:0, name:"root".to_string(), parent:0 });

        Client {
            serv_ip: IpAddr::V4(Ipv4Addr::new(0,0,0,0)),
            user: user,
            pass: pass,
            cookie: Vec::new(),
            widgets: widgets, 
            objects: HashMap::new(),
            grids: HashSet::new(),
            charlist: Vec::new(),
            resources: HashMap::new(),
            seq: 0,
            rx_rel_seq: 0,
            que: LinkedList::new(),
            tx_buf: LinkedList::new(),
            enqueue_seq: 0,
            rel_cache: HashMap::new(),
        }
    }

    pub fn authorize (&mut self, hostname: &str, port: u16) -> Result<(), Error> {
        let host = {
            let mut ips = ::std::net::lookup_host(hostname).ok().expect("lookup_host");
            ips.next().expect("ip.next").ok().expect("ip.next.ok")
        };
        
        println!("connect to {}", host.ip());

        self.serv_ip = host.ip();
        //self.pass = pass;
        //let auth_addr: SocketAddr = SocketAddr {ip: ip, port: port};
        let auth_addr = SocketAddr::new(self.serv_ip, port);
        println!("authorize {} @ {}", self.user, auth_addr);
        //TODO add method connect(SocketAddr) to TcpStream
        //let stream = tryio!("tcp.connect" TcpStream::connect(self.auth_addr));
        let stream = TcpStream::connect(auth_addr).unwrap();
        let context = SslContext::new(SslMethod::Sslv23).unwrap();
        let mut stream = SslStream::new(&context, stream).unwrap();

        // send 'pw' command
        let user = self.user.as_bytes();
        let buf_len = (3 + user.len() + 1 + 32) as u16;
        let mut buf: Vec<u8> = Vec::with_capacity((2 + buf_len) as usize);
        buf.write_u16::<be>(buf_len).unwrap();
        buf.push_all("pw".as_bytes());
        buf.push(0);
        buf.push_all(user);
        buf.push(0);
        let pass_hash = hash(Type::SHA256, self.pass.as_bytes());
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
                Client::start_send_beats();
            },
            Message::C_SESS(_) => { println!("     !!! client must not receive C_SESS !!!"); },
            Message::REL( rel ) => {
                println!("RX: REL {}", rel.seq);
                if rel.seq == self.rx_rel_seq {
                    try!(self.dispatch_rel_cache(&rel));
                } else if (rel.seq - self.rx_rel_seq) < 32767 {
                    // future REL
                    self.cache_rel(rel);
                } else {
                    // past REL
                    println!("past");
                    //TODO self.ack(seq);
                    let last_acked_seq = self.rx_rel_seq - 1;
                    try!(self.enqueue_to_send(Message::ACK(Ack{seq : last_acked_seq})));
                }
            },
            Message::ACK(ack)   => {
                println!("RX: ACK {}", ack.seq);
                if ack.seq == self.seq {
                    //println!("our rel {} acked", self.seq);
                    self.remove_rel_from_que();
                    //FIXME self.seq += last_rel.rels.len()
                    self.seq += 1;
                }
            },
            Message::BEAT       => { println!("     !!! client must not receive BEAT !!!"); },
            Message::MAPREQ(_)  => { println!("     !!! client must not receive MAPREQ !!!"); },
            Message::MAPDATA(/*mapdata*/_) => {
                println!("RX: MAPDATA");
                //TODO FIXME remove MAPREQ only after all MAPDATA pieces collected
                self.remove_mapreq_from_que();
            },
            Message::OBJDATA(objdata) => {
                //println!("RX: OBJDATA {:?}", objdata);
                try!(self.enqueue_to_send(Message::OBJACK(ObjAck::new(&objdata)))); // send OBJACKs
                for o in objdata.obj.iter() {
                    if !self.objects.contains_key(&o.id) {
                        self.objects.insert(o.id, Obj{resid:0, x:0, y:0});
                    }
                    if let Some(obj) = self.objects.get_mut(&o.id) {
                        //TODO check for o.frame vs obj.frame
                        for prop in o.prop.iter() {
                            match *prop {
                                ObjProp::odREM => { /*FIXME objects.remove(&o.id); break;*/ },
                                ObjProp::odMOVE(xy,_) => { let (x,y) = xy; obj.x = x; obj.y = y; },
                                ObjProp::odRES(resid) => { obj.resid = resid; },
                                ObjProp::odCOMPOSE(resid) => { obj.resid = resid; },
                                _ => {},
                            }
                        }
                    };
                }
                
                let mut tmp = Vec::new();
                for o in self.objects.values() {
                    let gx:i32 = o.x / 1100;
                    let gy:i32 = o.y / 1100;
                    if !self.grids.contains(&(gx,gy)) {
                        tmp.push((gx,gy));
                        self.grids.insert((gx,gy));
                    }
                }
                for (x,y) in tmp {
                     try!(self.mapreq(x, y));
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
                    println!("      {:?}", wdg);
                    self.widgets.insert(wdg.id, Widget{id:wdg.id, name:wdg.kind.clone(), parent:wdg.parent});
                },
                RelElem::WDGMSG(ref msg) => {
                    println!("      {:?}", msg);
                    //TODO match against widget.type and message.type
                    match self.widgets.get(&(msg.id)) {
                        None => {},
                        Some(w) => {
                            if (w.name == "charlist") && (msg.name == "add") {
                                match msg.args[0] {
                                    MsgList::tSTR(ref char_name) => {
                                        println!("    add char '{}'", char_name);
                                        /*FIXME rewrite without cloning*/
                                        self.charlist.push(char_name.clone());
                                    },
                                    _ => {}
                                }
                            }
                        }
                    }
                },
                RelElem::DSTWDG(ref wdg) => {
                    println!("      {:?}", wdg);
                    self.widgets.remove(&wdg.id);
                },
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
                RelElem::TILES(_) => {},
                RelElem::BUFF(_) => {},
                RelElem::SESSKEY(_) => {},
            }
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

    pub fn widget_id_by_name (&self, name:&str) -> Option<u16> {
        for (id,w) in self.widgets.iter() {
            if w.name == name {
                return Some(*id)
            }
        }
        None
    }

    pub fn react (&mut self) -> Result<(),Error> {
        //TODO send REL until reply
        if self.charlist.len() > 0 {
            println!("send play '{}'", self.charlist[0]);
            //TODO let mut rel = Rel::new(seq,id,name);
            let char_name = self.charlist[0].clone();
            let mut rel = Rel{seq:self.seq, rel:Vec::new()};
            let id = self.widget_id_by_name("charlist").expect("charlist widget is not found");
            let name : String = "play".to_string();
            let mut args : Vec<MsgList> = Vec::new();
            args.push(MsgList::tSTR(char_name));
            let elem = RelElem::WDGMSG(WdgMsg{ id : id, name : name, args : args });
            rel.rel.push(elem);
            try!(self.enqueue_to_send(Message::REL(rel)));
            self.charlist.clear();
        }
        Ok(())
    }

    pub fn connect (&mut self) -> Result<(),Error> {
        //TODO send SESS until reply
        //TODO get username from server responce, not from auth username
        let cookie = self.cookie.clone();
        let user = self.user.clone();
        try!(self.enqueue_to_send(Message::C_SESS(cSess{login:user, cookie:cookie})));
        Ok(())
    }

    pub fn mapreq (&mut self, x:i32, y:i32) -> Result<(),Error> {
        //TODO send until reply
        //TODO replace with client.send(Message::MapReq::new(x,y).to_buf())
        //     or client.send(Message::mapreq(x,y).to_buf())
        try!(self.enqueue_to_send(Message::MAPREQ(MapReq{x:x,y:y})));
        Ok(())
    }

    pub fn rx (&mut self, buf:&[u8]) -> Result<(),Error> {
        try!(self.dispatch_message(buf));
        //TODO reactor.react(&client)
        try!(self.react());
        Ok(())
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
        self.tx_buf.pop_back()
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
    
    pub fn go (&mut self, x: i32, y: i32) -> Result<(),Error> {
        println!("let's GO somewhere!");
        //TODO let mut rel = Rel::new(seq,id,name);
        let mut rel = Rel{seq:self.seq, rel:Vec::new()};
        let id = self.widget_id_by_name("mapview").expect("charlist widget is not found");
        let name : String = "click".to_string();
        let mut args : Vec<MsgList> = Vec::new();
        args.push(MsgList::tCOORD((907, 755)));
        args.push(MsgList::tCOORD((x, y)));
        args.push(MsgList::tINT(1));
        args.push(MsgList::tINT(0));
        let elem = RelElem::WDGMSG(WdgMsg{ id : id, name : name, args : args });
        rel.rel.push(elem);
        try!(self.enqueue_to_send(Message::REL(rel)));
        self.charlist.clear();
        Ok(())
    }
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

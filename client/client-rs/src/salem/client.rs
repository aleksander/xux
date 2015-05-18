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
    pub xy : (i32,i32),
}

pub struct Client {
    //TODO do all fileds PRIVATE and use callback interface
    pub serv_ip: IpAddr,
    pub user: &'static str,
    pub cookie: Vec<u8>,
    pub widgets : HashMap<u16,String>,
    pub objects : HashMap<u32,Obj>,
    pub grids : HashSet<(i32,i32)>,
    pub charlist : Vec<String>,
    pub resources : HashMap<u16,String>,
    pub seq : u16,
    pub last_rx_rel_seq : Option<u16>,
    pub que: LinkedList<Vec<u8>>,
    pub tx_buf: LinkedList<Vec<u8>>
}

impl Client {
    pub fn new () -> Client {
        let mut widgets = HashMap::new();
        widgets.insert(0, "root".to_string());
        let objects = HashMap::new();
        let grids = HashSet::new();
        let charlist = Vec::new();
        let resources = HashMap::new();

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
            serv_ip: IpAddr::V4(Ipv4Addr::new(0,0,0,0)),
            user: "",
            cookie: Vec::new(),
            widgets: widgets, 
            objects: objects,
            grids: grids,
            charlist: charlist,
            resources: resources,
            seq: 0,
            last_rx_rel_seq: None,
            que: LinkedList::new(),
            tx_buf: LinkedList::new(),
        }
    }

    pub fn authorize (&mut self, user: &'static str, pass: &str, hostname: &str, port: u16) -> Result<(), Error> {
        let host = {
            let mut ips = ::std::net::lookup_host(hostname).ok().expect("lookup_host");
            ips.next().expect("ip.next").ok().expect("ip.next.ok")
        };
        println!("connect to {}", host.ip());

        self.user = user;
        self.serv_ip = host.ip();
        //self.pass = pass;
        //let auth_addr: SocketAddr = SocketAddr {ip: ip, port: port};
        let auth_addr = SocketAddr::new(self.serv_ip, port);
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

    pub fn start_send_beats () {
        /*TODO*/
    }

    pub fn enqueue_to_send (&mut self, msg: Message) -> Result<(),Error> {
        match msg.to_buf() {
            Ok(buf) => { self.tx_buf.push_front(buf); Ok(()) },
            Err(e) => { println!("enqueue error: {:?}", e); Err(e) },
        }
    }

    pub fn enqueue_to_send_and_repeat (&mut self, msg: Message) -> Result<(),Error> {
        match msg.to_buf() {
            Ok(buf) => {
                self.tx_buf.push_front(buf.clone());
                self.que.push_front(buf);
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

        {
            let mut duplicate = false;
            if let Message::REL(ref rel) = msg {
                match self.last_rx_rel_seq {
                    None => {
                        self.last_rx_rel_seq = Some(rel.seq);
                    }
                    Some(seq) => {
                        if rel.seq == seq {
                            println!("RX: REL {} duplicate", seq);
                            duplicate = true;
                        } else {
                            self.last_rx_rel_seq = Some(rel.seq);
                        }
                    }
                }
            }
            if !duplicate {
                println!("RX: {:?}", msg);
                if let Some(rem) = remains { println!("                 REMAINS {} bytes", rem.len()); }
            }
        }

        match msg {
            Message::S_SESS(sess) => {
                match sess.err {
                    SessError::OK => {},
                    _ => {
                        //TODO return Error::from(SessError)
                        return Err(Error{source:"session error",detail:None});
                        //XXX ??? should we send CLOSE too ???
                        //??? or can we re-send our SESS requests in case of BUSY err ?
                    }
                }
                Client::start_send_beats();
            },
            Message::C_SESS( /*sess*/ _ ) => {/*TODO*/},
            Message::REL( rel ) => {
                //TODO do not process duplicates, but ACK only
                //XXX are we handle seq right in the case of overflow ???
                try!(self.enqueue_to_send(Message::ACK(Ack{seq : rel.seq + ((rel.rel.len() as u16) - 1)})));
                for r in rel.rel.iter() {
                    match *r {
                        RelElem::NEWWDG(ref wdg) => {
                            self.widgets.insert(wdg.id, wdg.kind.clone()/*FIXME String -> &str*/);
                        },
                        RelElem::WDGMSG(ref msg) => {
                            //TODO match against widget.type and message.type
                            match self.widgets.get(&(msg.id)) {
                                None => {},
                                Some(c) => {
                                    if (c == "charlist") && (msg.name == "add") {
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
                        RelElem::TILES(_) => {},
                        RelElem::BUFF(_) => {},
                        RelElem::SESSKEY(_) => {},
                    }
                }
            },
            Message::ACK(ack)   => {
                if ack.seq == self.seq {
                    println!("our rel {} acked", self.seq);
                    //TODO remove pending REL message with this seq
                    //FIXME self.seq += last_rel.rels.len()
                    self.seq += 1;
                }
            },
            Message::BEAT       => { println!("     !!! client must not receive BEAT !!!"); },
            Message::MAPREQ(_)  => { println!("     !!! client must not receive MAPREQ !!!"); },
            Message::MAPDATA(_) => {},
            Message::OBJDATA( objdata ) => {
                try!(self.enqueue_to_send(Message::OBJACK(ObjAck::new(&objdata)))); // send OBJACKs
                for o in objdata.obj.iter() {
                    if !self.objects.contains_key(&o.id) {
                        self.objects.insert(o.id, Obj{resid:0, xy:(0,0)});
                    }
                    if let Some(obj) = self.objects.get_mut(&o.id) {
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
                    };
                }
                
                let mut tmp = Vec::new();
                for o in self.objects.values() {
                    let (x,y) = o.xy;
                    let gx:i32 = x / 1100;
                    let gy:i32 = y / 1100;
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
            Message::CLOSE(_)   => {
                return Err(Error{source:"session closed",detail:None});
            },
        }

        Ok(())
    }

    pub fn widget_id_by_name (&self, name:&str) -> Option<u16> {
        for (id,n) in self.widgets.iter() {
            if n == name {
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
        try!(self.enqueue_to_send_and_repeat(Message::C_SESS(cSess{login:self.user.to_string(), cookie:cookie})));
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
    
    pub fn timeout (&self) {
        println!("TIMEOUT!");
    }
    
    pub fn tx (&self) {
        println!("TXed!");
    }
    
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
    
    pub fn go (&mut self) -> Result<(),Error> {
        println!("let's GO somewhere!");
        //TODO let mut rel = Rel::new(seq,id,name);
        let mut rel = Rel{seq:self.seq, rel:Vec::new()};
        let id = self.widget_id_by_name("mapview").expect("charlist widget is not found");
        let name : String = "click".to_string();
        let mut args : Vec<MsgList> = Vec::new();
        args.push(MsgList::tCOORD((907, 755)));
        args.push(MsgList::tCOORD((39683, 36377)));
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

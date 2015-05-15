#![feature(rustc_private)]
#![feature(convert)]
#![feature(ip_addr)]
#![feature(collections)]
#![feature(lookup_host)]

extern crate openssl;
extern crate rustc_serialize;
extern crate mio;

#[macro_use]
extern crate log;

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
use std::io::Cursor;
use std::io::Read;
use std::io::Write;

mod salem;
use salem::message::*;

extern crate byteorder;
use byteorder::{LittleEndian, BigEndian, ReadBytesExt, WriteBytesExt};
#[allow(non_camel_case_types)]
type le = LittleEndian;
#[allow(non_camel_case_types)]
type be = BigEndian;

struct Obj {
    resid : u16,
    xy : (i32,i32),
}

struct Client {
    serv_ip: std::net::IpAddr,
    user: &'static str,
    cookie: Vec<u8>,
    widgets : HashMap<u16,String>,
    objects : HashMap<u32,Obj>,
    grids : HashSet<(i32,i32)>,
    charlist : Vec<String>,
    resources : HashMap<u16,String>,
    seq : u16,
    last_rx_rel_seq : Option<u16>,
    que: LinkedList<Vec<u8>>,
}

impl Client {
    fn new () -> Client {
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
            serv_ip: std::net::IpAddr::V4(std::net::Ipv4Addr::new(0,0,0,0)),
            user: "",
            cookie: Vec::new(),
            widgets: widgets, 
            objects: objects,
            grids: grids,
            charlist: charlist,
            resources:resources,
            seq : 0,
            last_rx_rel_seq : None,
            que: LinkedList::new(),
        }
    }

    fn authorize (&mut self, user: &'static str, pass: &str, hostname: &str, port: u16) -> Result<(), Error> {
        let host = {
            let mut ips = std::net::lookup_host(hostname).ok().expect("lookup_host");
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

    fn start_send_beats () {
        /*TODO*/
    }

    fn enqueue_to_send (&self, msg: Message, tx_buf:&mut LinkedList<Vec<u8>>) {
        match msg.to_buf() {
            Ok(buf) => { tx_buf.push_front(buf); },
            Err(e) => { println!("enqueue error: {:?}", e); },
        }
    }

    fn enqueue_to_send_and_repeat (&mut self, msg: Message, tx_buf:&mut LinkedList<Vec<u8>>) /*TODO return Result*/ {
        match msg.to_buf() {
            Ok(buf) => {
                tx_buf.push_front(buf.clone());
                self.que.push_front(buf);
            },
            Err(e) => { println!("enqueue error: {:?}", e); },
        }
    }

    fn dispatch_message (&mut self, buf:&[u8], tx_buf:&mut LinkedList<Vec<u8>>) -> Result<(),Error> {
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
                        //TODO event_loop.shutdown(); exit();
                        //XXX ??? should we send CLOSE too ???
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
            Message::BEAT    => { println!("     !!! client must not receive BEAT !!!"); },
            Message::MAPREQ(_)  => { println!("     !!! client must not receive MAPREQ !!!"); },
            Message::MAPDATA(_) => {},
            Message::OBJDATA( objdata ) => {
                self.enqueue_to_send(Message::OBJACK(ObjAck::new(&objdata)), tx_buf); // send OBJACKs
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
                return Err(Error{source:"session closed",detail:None});
            },
        }

        //TODO reactor.react(&client)
        self.react(tx_buf);

        Ok(())
    }

    fn widget_id_by_name (&self, name:&str) -> Option<u16> {
        for (id,n) in self.widgets.iter() {
            if n == name {
                return Some(*id)
            }
        }
        None
    }

    fn react (&mut self, tx_buf:&mut LinkedList<Vec<u8>>) {
        //TODO send REL until reply
        if self.charlist.len() > 0 {
            println!("send play '{}'", self.charlist[0]);
            let char_name = self.charlist[0].clone();
            let mut rel = Rel{seq:self.seq, rel:Vec::new()};
            let id = self.widget_id_by_name("charlist").expect("charlist widget is not found");
            let name : String = "play".to_string();
            let mut args : Vec<MsgList> = Vec::new();
            args.push(MsgList::tSTR(char_name));
            let elem = RelElem::WDGMSG(WdgMsg{ id : id, name : name, args : args });
            rel.rel.push(elem);
            self.enqueue_to_send(Message::REL(rel), tx_buf);
            self.charlist.clear();
        }
    }

    fn connect (&mut self, tx_buf:&mut LinkedList<Vec<u8>>) {
        //TODO send SESS until reply
        //TODO get username from server responce, not from auth username
        let cookie = self.cookie.clone();
        self.enqueue_to_send_and_repeat(Message::C_SESS(cSess{login:self.user.to_string(), cookie:cookie}), tx_buf);
    }

    fn mapreq (&self, x:i32, y:i32, tx_buf:&mut LinkedList<Vec<u8>>) {
        //TODO send until reply
        //TODO replace with client.send(Message::MapReq::new(x,y).to_buf())
        //     or client.send(Message::mapreq(x,y).to_buf())
        self.enqueue_to_send(Message::MAPREQ(MapReq{x:x,y:y}), tx_buf);
    }

}

//TODO
/*
enum MsgType {
    REL,
    C_SESS,
    MAPREQ,
}
*/


fn main() {
    //TODO handle keyboard interrupt
    //TODO replace all unwraps with normal error handling
    //TODO ADD tests:
    //        for i in range(0u8, 255) {
    //            let mut v = Vec::new();
    //            v.push(i);
    //            println!("{}", Message::from_buf(v.as_slice()));
    //        }
    //TODO FIXME add username/password prompt, remove plain text username/password from sources

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

    use mio::Socket;

    struct UdpHandler<'a> {
        sock: mio::NonBlock<mio::udp::UdpSocket>,
        addr: std::net::SocketAddr,
        tx_buf: LinkedList<Vec<u8>>,
        client: &'a mut Client,
        //start: bool,
    }

    impl<'a> UdpHandler<'a> {
        fn new(sock: mio::NonBlock<mio::udp::UdpSocket>, client:&'a mut Client, addr: std::net::SocketAddr) -> UdpHandler<'a> {
            UdpHandler {
                sock: sock,
                addr: addr,
                tx_buf: LinkedList::new(),
                client: client,
                //start: true,
            }
        }
    }

    const CLIENT: mio::Token = mio::Token(0);

    impl<'a> mio::Handler for UdpHandler<'a> {
        type Timeout = usize;
        type Message = ();

        fn readable(&mut self, eloop: &mut mio::EventLoop<UdpHandler>, token: mio::Token, _: mio::ReadHint) {
            match token {
                CLIENT => {
                    let mut rx_buf = mio::buf::RingBuf::new(65535);
                    self.sock.recv_from(&mut rx_buf).ok().expect("sock.recv");
                    let mut client: &mut Client = self.client;
                    let buf: &[u8] = mio::buf::Buf::bytes(&rx_buf);
                    if let Err(e) = client.dispatch_message(buf, &mut self.tx_buf) {
                        println!("error: {:?}", e);
                        eloop.shutdown();
                    }
                },
                _ => ()
            }
        }

        fn writable(&mut self, eloop: &mut mio::EventLoop<UdpHandler>, token: mio::Token) {
            match token {
                CLIENT => {
                    match self.tx_buf.pop_back() {
                        Some(data) => {
                            if let Ok((msg,_)) = Message::from_buf(data.as_slice(),MessageDirection::FromClient) {
                                println!("TX: {:?}", msg);
                            }
                            let mut buf = mio::buf::SliceBuf::wrap(data.as_slice());
                            if let Err(e) = self.sock.send_to(&mut buf, &self.addr) {
                                println!("send_to error: {}", e);
                                eloop.shutdown();
                            }
                            if !self.client.que.is_empty() {
                                //TODO use returned timeout handle to cancel timeout
                                if let Err(e) = eloop.timeout_ms(123, 300) {
                                    println!("eloop.timeout FAILED: {:?}", e);
                                    eloop.shutdown();
                                }
                            }
                            //self.start = false;
                        },
                        None => {}
                    }
                },
                _ => ()
            }
        }

        fn timeout (&mut self, eloop: &mut mio::EventLoop<UdpHandler>, /*timeout*/ _: usize) {
            let client = &self.client;
            match client.que.front() {
                Some(buf) => {
                    println!("re-enqueue to send by timeout");
                    self.tx_buf.push_front(buf.clone());
                    //TODO use returned timeout handle to cancel timeout
                    if let Err(e) = eloop.timeout_ms(123, 300) {
                        println!("eloop.timeout FAILED: {:?}", e);
                        eloop.shutdown();
                    }
                }
                None => {
                    println!("WARNING: timeout on empty que");
                }
            }
        }
    }

    let hostname = "game.salemthegame.com";

    let any = str::FromStr::from_str("0.0.0.0:0").ok().expect("any.from_str");
    let sock = mio::udp::bind(&any).ok().expect("bind");

    //FIXME sock.connect(&addr);
    sock.set_reuseaddr(true).ok().expect("set_reuseaddr");

    //TODO return Result and match
    let mut client = Client::new(/*"game.salemthegame.com", 1871, 1870*/);

    //TODO FIXME get login/password from command line instead of storing them here
    match client.authorize("salvian", "простойпароль", hostname, 1871) {
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
    let ip = client.serv_ip;
    let mut handler = UdpHandler::new(sock, &mut client, std::net::SocketAddr::new(ip, 1870));
    handler.client.connect(&mut handler.tx_buf); //TODO return Result and match

    info!("run event loop");
    event_loop.run(&mut handler).ok().expect("Failed to run the event loop");
}

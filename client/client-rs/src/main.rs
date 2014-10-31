#![feature(macro_rules)]

extern crate openssl;
extern crate serialize;

use std::io::Writer;
use std::io::MemWriter;
use std::io::net::tcp::TcpStream;
use std::io::net::udp::UdpSocket;
use std::io::net::ip::Ipv4Addr;
use std::io::net::ip::IpAddr;
use std::io::net::ip::SocketAddr;
use std::io::net::addrinfo::get_host_addresses;
use std::io::MemReader;
use std::io::timer;
use std::collections::hashmap::HashMap;
use std::str;
use std::time::Duration;
use serialize::hex::ToHex;
use openssl::crypto::hash::{SHA256, hash};
use openssl::ssl::{Sslv23, SslContext, SslStream};

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
    //unknown=2 proto=Salem ver=33 user=soos cookie=[ .//J..%.....R...G......Q x![..e.b]
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
    w.unwrap()
}

fn ack (seq: u16) -> Vec<u8> {
    let mut w = MemWriter::new();
    w.write_u8(2).unwrap(); //ACK
    w.write_le_u16(seq).unwrap();
    w.unwrap()
}

fn beat () -> Vec<u8> {
    let mut w = MemWriter::new();
    w.write_u8(3).unwrap(); //BEAT
    w.unwrap()
}

fn rel_wdgmsg_play (seq: u16, name: &str) -> Vec<u8> {
    let mut w = MemWriter::new();
    //REL  seq=0
    //  WDGMSG
    //  id=0 name=focus
    //    list:
    //    INT : 1

    // REL
    w.write_u8(1).unwrap();
    // sequence
    w.write_le_u16(seq).unwrap();
    // rel type WDGMSG
    w.write_u8(1).unwrap();
    // widget id
    w.write_le_u16(3).unwrap();
    // message name
    w.write("play".as_bytes()).unwrap();
    w.write_u8(0).unwrap();
    // args list
    w.write_u8(2).unwrap(); // list element type T_STR
    w.write(name.as_bytes()).unwrap(); // element
    w.write_u8(0).unwrap();
    w.unwrap()
}


struct Obj {
    x:i32,
    y:i32,
    frame:i32,
    resid:u16,
}

impl Obj {
    fn new() -> Obj {
        Obj{
            x:0,
            y:0,
            frame:0,
            resid:0,
        } 
    }
}

struct Client {
    user: &'static str,
    //pass: &'static str,
    cookie: Vec<u8>,
    host: &'static str,
    auth_port: u16,
    //port: u16,
    auth_addr: SocketAddr,
    host_addr: SocketAddr,
    //any_addr: SocketAddr,
    //udp_rx: UdpSocket,
    //udp_tx: UdpSocket,
    main_to_sender: Sender<Vec<u8>>,     //TODO type OutputBuffer = Vec<u8>
    //sender_from_any: Receiver<Vec<u8>>,  //TODO type OutputBuffer = Vec<u8>
    //receiver_to_sender: Sender<Vec<u8>>, //TODO type OutputBuffer = Vec<u8>
    //beater_to_sender: Sender<Vec<u8>>,
    receiver_to_main: Sender<()>,
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
        let (tx3,rx3) = channel(); // any -> viewer (new object)
        let (tx4,rx4) = channel(); // any -> main   (exit signal)

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
            let (id,obj):(u32,Obj) = viewer_from_any.recv();
            objects.insert(id,obj);
            let mut minx = obj.x;
            let mut miny = obj.y;
            let mut maxx = obj.x;
            let mut maxy = obj.y;
            loop {
                //TODO while(try_recv)
                let (id,obj):(u32,Obj) = viewer_from_any.recv();
                objects.insert(id,obj);
                if obj.x < minx { minx = obj.x; }
                if obj.y < miny { miny = obj.y; }
                if obj.x > maxx { maxx = obj.x; }
                if obj.y > maxy { maxy = obj.y; }
            }
        });

        let msg_types = [
            "SESS",
            "REL",
            "ACK",
            "BEAT",
            "MAPREQ",
            "MAPDATA",
            "OBJDATA",
            "OBJACK",
            "CLOSE" ];

        let objdata_types = [
            "OD_REM",
            "OD_MOVE",
            "OD_RES",
            "OD_LINBEG",
            "OD_LINSTEP",
            "OD_SPEECH",
            "OD_COMPOSE",
            "OD_DRAWOFF",
            "OD_LUMIN",
            "OD_AVATAR",
            "OD_FOLLOW",
            "OD_HOMING",
            "OD_OVERLAY",
            "OD_AUTH",
            "OD_HEALTH",
            "OD_BUDDY",
            "OD_CMPPOSE",
            "OD_CMPMOD",
            "OD_CMPEQU",
            "OD_ICON" ];

    let sess_errors = [
        "OK",
        "AUTH",
        "BUSY",
        "CONN",
        "PVER",
        "EXPR" ];

        let debug = true;

        // receiver
        let mut udp_rx = sock.clone();
        let receiver_to_main = tx4.clone();
        let receiver_to_beater = tx2.clone();
        let receiver_to_sender = tx1.clone();
        let receiver_to_viewer = tx3.clone();
        spawn(proc() {
            //let mut connected = false;
            let mut buf = [0u8, ..65535];
            let mut charlist = Vec::new();
            let mut widgets = HashMap::new();
            let mut resources = HashMap::new();
            widgets.insert(0, "root".to_string());
            loop {
                let (len,addr) = udp_rx.recv_from(buf.as_mut_slice()).unwrap();
                if addr != host_addr {
                    println!("wrong host: {}", addr);
                    continue;
                }
                //println!("seceiver: dgram [{}]", buf.slice_to(len).to_hex());
                let mut r = MemReader::new(buf.slice_to(len).to_vec());
                let mtype = r.read_u8().unwrap() as uint;
                if debug { println!("receiver: {}", msg_types[mtype]); }
                match mtype {
                    0 /*SESS*/ => {
                        let sess_error = r.read_u8().unwrap() as uint;
                        if sess_error != 0 {
                            println!("sess error {}", sess_errors[sess_error]);
                            receiver_to_main.send(());
                            // ??? should we send CLOSE too ???
                            break;
                        }
                        //connected = true;
                        receiver_to_beater.send(());
                    },
                    1 /*REL*/ => {
                        let seq = r.read_le_u16().unwrap();
                        //println!("  seq: {}", seq);
                        let mut rel_count = 0u16;
                        while !r.eof() {
                            let rel;
                            let mut rel_type = r.read_u8().unwrap() as uint;
                            if (rel_type & 0x80) != 0 {
                                rel_type &= !0x80;
                                let rel_len = r.read_le_u16().unwrap() as uint;
                                rel = r.read_exact(rel_len).unwrap();
                            } else {
                                rel = r.read_to_end().unwrap();
                            }
                            rel_count += 1;

                            let mut rr = MemReader::new(rel);
                            match rel_type {
                                0  /*NEWWDG*/ => {
                                    let wdg_id = rr.read_le_u16().unwrap();
                                    let wdg_type = String::from_utf8(rr.read_until(0).unwrap()).unwrap();
                                    let wdg_parent = rr.read_le_u16().unwrap();
                                    //pargs = read_list
                                    //cargs = read_list
                                    if debug { println!("  NEWWDG id:{} type:{} parent:{}", wdg_id, wdg_type, wdg_parent); }
                                    widgets.insert(wdg_id as uint, wdg_type);
                                },
                                1  /*WDGMSG*/ => {
                                    let wdg_id = rr.read_le_u16().unwrap();
                                    let msg_name = String::from_utf8(rr.read_until(0).unwrap()).unwrap();
                                    if debug { println!("  WDGMSG id:{} name:{}", wdg_id, msg_name); }
                                    if widgets.find(&(wdg_id as uint)).unwrap().as_slice() == "charlist\0" && msg_name.as_slice() == "add\0" {
                                        let el_type = rr.read_u8().unwrap();
                                        if el_type != 2 { println!("{} NOT T_STR", el_type); continue; }
                                        let char_name = String::from_utf8(rr.read_until(0).unwrap()).unwrap();
                                        if debug { println!("    add char '{}'", char_name); }
                                        charlist.push(char_name);
                                    }
                                },
                                2  /*DSTWDG*/ => {},
                                3  /*MAPIV*/ => {},
                                4  /*GLOBLOB*/ => {},
                                5  /*PAGINAE*/ => {},
                                6  /*RESID*/ => {
                                    let resid = rr.read_le_u16().unwrap();
                                    let resname = String::from_utf8(rr.read_until(0).unwrap()).unwrap();
                                    let resver = rr.read_le_u16().unwrap();
                                    println!("  RESID id:{} name:{} ver:{}", resid, resname, resver);
                                    resources.insert(resid, resname);
                                },
                                7  /*PARTY*/ => {},
                                8  /*SFX*/ => {},
                                9  /*CATTR*/ => {},
                                10 /*MUSIC*/ => {},
                                11 /*TILES*/ => {},
                                12 /*BUFF*/ => {},
                                13 /*SESSKEY*/ => {},
                                _ => {
                                    println!("\x1b[31m  UNKNOWN {}\x1b[39;49m", rel_type);
                                },
                            }
                        }
                        //XXX are we handle seq right in the case of overflow ???
                        receiver_to_sender.send(ack(seq + (rel_count - 1)));
                    },
                    2 /*ACK*/ => {
                        let seq = r.read_le_u16().unwrap();
                        if debug { println!("  seq: {}", seq); }
                    },
                    3 /*BEAT*/ => {},
                    4 /*MAPREQ*/ => {},
                    5 /*MAPDATA*/ => {},
                    6 /*OBJDATA*/ => {
                        let mut w = MemWriter::new();
                        w.write_u8(7).unwrap(); //OBJACK writer
                        while !r.eof() {
                            /*let fl =*/ r.read_u8().unwrap();
                            let id = r.read_le_u32().unwrap();
                            let frame = r.read_le_i32().unwrap();
                            if debug { println!("  id={} frame={}", id, frame); }
                            w.write_le_u32(id).unwrap();
                            w.write_le_i32(frame).unwrap();
                            let mut obj = Obj::new();
                            obj.frame = frame;
                            loop {
                                let t = r.read_u8().unwrap() as uint;
                                //if debug { if t < objdata_types.len() { println!("    {}", objdata_types[t]); } }
                                match t {
                                    0   /*OD_REM*/ => {},
                                    1   /*OD_MOVE*/ => {
                                        let (x,y) = (r.read_le_i32().unwrap(), r.read_le_i32().unwrap());
                                        /*let ia =*/ r.read_le_u16().unwrap();
                                        if debug { println!("    MOVE ({},{})", x, y); }
                                        obj.x = x;
                                        obj.y = y;
                                    },
                                    2   /*OD_RES*/ => {
                                        let mut resid = r.read_le_u16().unwrap();
                                        if (resid & 0x8000) != 0 {
                                            resid &= !0x8000;
                                            let sdt_len = r.read_u8().unwrap() as uint;
                                            let _/*sdt*/ = r.read_exact(sdt_len).unwrap();
                                        }
                                        if debug { println!("    RES {}", resid); }
                                        obj.resid = resid;
                                    },
                                    3   /*OD_LINBEG*/ => {
                                        /*let s =*/ (r.read_le_i32().unwrap(), r.read_le_i32().unwrap());
                                        /*let t =*/ (r.read_le_i32().unwrap(), r.read_le_i32().unwrap());
                                        let _/*c*/ = r.read_le_i32();
                                    },
                                    4   /*OD_LINSTEP*/ => {
                                        let l = r.read_le_i32().unwrap();
                                        if debug { println!("    LINSTEP l={}", l); }
                                    },
                                    5   /*OD_SPEECH*/ => {
                                        let _/*zo*/ = r.read_le_u16();
                                        /*let text =*/ String::from_utf8(r.read_until(0).unwrap()).unwrap();
                                    },
                                    6   /*OD_COMPOSE*/ => {
                                        /*let resid =*/ r.read_le_u16().unwrap();
                                    },
                                    7   /*OD_DRAWOFF*/ => {
                                        /*let off =*/ (r.read_le_i32().unwrap(), r.read_le_i32().unwrap());
                                    },
                                    8   /*OD_LUMIN*/ => {
                                        /*let off =*/ (r.read_le_i32().unwrap(), r.read_le_i32().unwrap());
                                        /*let sz =*/ r.read_le_u16().unwrap();
                                        /*let str_ =*/ r.read_u8().unwrap();
                                    },
                                    9   /*OD_AVATAR*/ => {
                                        loop {
                                            let layer = r.read_le_u16().unwrap();
                                            if layer == 65535 { break; }
                                        }
                                    },
                                    10  /*OD_FOLLOW*/ => {
                                        let oid = r.read_le_u32().unwrap();
                                        if oid == 0xff_ff_ff_ff {
                                            /*let xfres =*/ r.read_le_u16().unwrap();
                                            /*let xfname =*/ String::from_utf8(r.read_until(0).unwrap()).unwrap();
                                        }
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
                                    },
                                    12  /*OD_OVERLAY*/ => {
                                        /*let olid =*/ r.read_le_i32().unwrap();
                                        let resid = r.read_le_u16().unwrap();
                                        if (resid & 0x8000) != 0 {
                                            let sdt_len = r.read_u8().unwrap() as uint;
                                            /*let sdt =*/ r.read_exact(sdt_len).unwrap();
                                        }
                                    },
                                    13  /*OD_AUTH*/   => { /* Removed */ },
                                    14  /*OD_HEALTH*/ => {
                                        /*let hp =*/ r.read_u8().unwrap();
                                    },
                                    15  /*OD_BUDDY*/ => {
                                        let name = String::from_utf8(r.read_until(0).unwrap()).unwrap();
                                        if name.len() > 0 {
                                            /*let group =*/ r.read_u8().unwrap();
                                            /*let btype =*/ r.read_u8().unwrap();
                                        }
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
                                    },
                                    19  /*OD_ICON*/ => {
                                        let resid = r.read_le_u16().unwrap();
                                        if resid != 65535 {
                                            /*let ifl =*/ r.read_u8().unwrap();
                                        }
                                    },
                                    255 /*OD_END*/ => { break; },
                                    _   /*UNKNOWN*/ => {}
                                }
                            }
                            receiver_to_viewer.send((id,obj));
                        }
                        receiver_to_sender.send(w.unwrap()); // send OBJACKs
                    },
                    7 /*OBJACK*/ => {},
                    8 /*CLOSE*/ => {
                        receiver_to_main.send(());
                        // ??? should we send CLOSE too ???
                        break;
                    },
                    _ /*UNKNOWN*/ => {
                    }
                }

                if !r.eof() {
                    let _/*remains*/ = r.read_to_end().unwrap();
                    //println!("                       REMAINS {} bytes", remains.len());
                }

                //TODO send REL until reply
                if charlist.len() > 0 {
                    //println!("send play '{}'", charlist[0]);
                    receiver_to_sender.send(rel_wdgmsg_play(0, charlist[0].as_slice()));
                    charlist.clear();
                }
            }
        });

        Client {
            user: "",
            cookie: Vec::new(),
            host: host,           //FIXME only need because TcpStream::connect() don't accept SocketAddr
            auth_port: auth_port, //FIXME only need because TcpStream::connect() don't accept SocketAddr
            //port: port,
            auth_addr: SocketAddr {ip: host_ip, port: auth_port},
            host_addr: SocketAddr {ip: host_ip, port: port},
            //udp_rx: sock.clone(),
            //udp_tx: sock.clone(),

            main_to_sender: tx1.clone(),
            //sender_from_any: rx1,
            //receiver_to_sender: tx1.clone(),
            //beater_to_sender: tx1.clone(),
            receiver_to_main: tx2.clone(),
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
        let stream = tryio!("tcp.connect" TcpStream::connect(self.host, self.auth_port));
        let mut stream = SslStream::new(&SslContext::new(Sslv23).unwrap(), stream).unwrap();

        // send 'pw' command
        // TODO form buffer and send all with one call
        // TODO tryio!(stream.write(Msg::pw(params...)));
        stream.write_be_u16((3+user.as_bytes().len()+1+32) as u16).unwrap();
        stream.write("pw".as_bytes()).unwrap();
        stream.write_u8(0).unwrap();
        stream.write(user.as_bytes()).unwrap();
        stream.write_u8(0).unwrap();
        let pass_hash = hash(SHA256, pass.as_bytes());
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

    /*
    fn start_sender (&self) {
        let rx = &self.sender_from_any;
        let tx = &self.udp_tx;
        spawn(proc() {
            loop {
                let buf: Vec<u8> = rx.recv();
                println!("sender: send {} bytes", buf.len());
                tx.send_to(buf.as_slice(), self.host_addr).unwrap();
            }
        });
    }
    */

    /*
    fn start_beater (&self) {
        spawn(proc() {
            self.from_receiver.recv();
            //send BEAT every 5 sec
            loop {
                self.beater_to_sender.send(beat());
                timer::sleep(Duration::seconds(5));
            }
        });
    }

    fn start_viewer (&self) {
        spawn(proc() {
            //let mut objects = HashMap::new();
            /*
            initscr();
            cbreak();
            noecho();
            halfdelay(1);
            */
            let (id,obj):(u32,Obj) = self.viewer_from_any.recv();
            self.objects.insert(id,obj);
            let mut minx = obj.x;
            let mut miny = obj.y;
            let mut maxx = obj.x;
            let mut maxy = obj.y;
            //let mut i = 0u;
            loop {
                //for _ in range(0u,100) {
                    //TODO while(try_recv)
                    let (id,obj):(u32,Obj) = self.viewer_from_any.recv();
                    self.objects.insert(id,obj);
                    if obj.x < minx { minx = obj.x; }
                    if obj.y < miny { miny = obj.y; }
                    if obj.x > maxx { maxx = obj.x; }
                    if obj.y > maxy { maxy = obj.y; }
                    //i += 1;
                //}
                /*
                for &obj in objects.values() {
                    mvaddch((obj.y-miny)/10, (obj.x-minx)/10, 'x' as u32);
                }
                mvaddstr(0, 0, format!("objects: {}", objects.len()).as_slice());
                mvaddstr(1, 0, format!("x: ({},{}) dx: {}    ", minx, maxx, maxx-minx).as_slice());
                mvaddstr(2, 0, format!("y: ({},{}) dy: {}    ", miny, maxy, maxy-miny).as_slice());
                mvaddstr(3, 0, format!("i: {}     ", i).as_slice());
                //println!("objects: {}", objects.len());
                refresh();
                let ch = getch();
                if ch == ('s' as i32) {
                    println!("TODO: dump to file");
                }
                */
                //endwin();
            }
        });
    }

    fn start_receiver (&self) {
        let msg_types = [
            "SESS",
            "REL",
            "ACK",
            "BEAT",
            "MAPREQ",
            "MAPDATA",
            "OBJDATA",
            "OBJACK",
            "CLOSE" ];

        let objdata_types = [
            "OD_REM",
            "OD_MOVE",
            "OD_RES",
            "OD_LINBEG",
            "OD_LINSTEP",
            "OD_SPEECH",
            "OD_COMPOSE",
            "OD_DRAWOFF",
            "OD_LUMIN",
            "OD_AVATAR",
            "OD_FOLLOW",
            "OD_HOMING",
            "OD_OVERLAY",
            "OD_AUTH",
            "OD_HEALTH",
            "OD_BUDDY",
            "OD_CMPPOSE",
            "OD_CMPMOD",
            "OD_CMPEQU",
            "OD_ICON" ];

        let debug = true;

        spawn(proc() {
            //let mut connected = false;
            let mut buf = [0u8, ..65535];
            let mut charlist = Vec::new();
            let mut widgets = HashMap::new();
            //let mut resources = HashMap::new();
            widgets.insert(0, "root".to_string());
            loop {
                let (len,addr) = self.udp_rx.recv_from(buf.as_mut_slice()).unwrap();
                if addr != self.host_addr {
                    println!("wrong host: {}", addr);
                    continue;
                }
                //println!("seceiver: dgram [{}]", buf.slice_to(len).to_hex());
                let mut r = MemReader::new(buf.slice_to(len).to_vec());
                let mtype = r.read_u8().unwrap() as uint;
                if debug { println!("receiver: {}", msg_types[mtype]); }
                match mtype {
                    0 /*SESS*/ => {
                        let sess_error = r.read_u8().unwrap() as uint;
                        if sess_error != 0 {
                            //println!("sess error {}", sess_errors[sess_error]);
                            self.sender_tx.send(());
                            // ??? should we send CLOSE too ???
                            break;
                        }
                        //connected = true;
                        self.receiver_to_beater.send(());
                    },
                    1 /*REL*/ => {
                        let seq = r.read_le_u16().unwrap();
                        //println!("  seq: {}", seq);
                        let mut rel_count = 0u16;
                        while !r.eof() {
                            let rel;
                            let mut rel_type = r.read_u8().unwrap() as uint;
                            if (rel_type & 0x80) != 0 {
                                rel_type &= !0x80;
                                let rel_len = r.read_le_u16().unwrap() as uint;
                                rel = r.read_exact(rel_len).unwrap();
                            } else {
                                rel = r.read_to_end().unwrap();
                            }
                            rel_count += 1;

                            let mut rr = MemReader::new(rel);
                            match rel_type {
                                0  /*NEWWDG*/ => {
                                    let wdg_id = rr.read_le_u16().unwrap();
                                    let wdg_type = String::from_utf8(rr.read_until(0).unwrap()).unwrap();
                                    let wdg_parent = rr.read_le_u16().unwrap();
                                    //pargs = read_list
                                    //cargs = read_list
                                    if debug { println!("  NEWWDG id:{} type:{} parent:{}", wdg_id, wdg_type, wdg_parent); }
                                    widgets.insert(wdg_id as uint, wdg_type);
                                },
                                1  /*WDGMSG*/ => {
                                    let wdg_id = rr.read_le_u16().unwrap();
                                    let msg_name = String::from_utf8(rr.read_until(0).unwrap()).unwrap();
                                    if debug { println!("  WDGMSG id:{} name:{}", wdg_id, msg_name); }
                                    if widgets.find(&(wdg_id as uint)).unwrap().as_slice() == "charlist\0" && msg_name.as_slice() == "add\0" {
                                        let el_type = rr.read_u8().unwrap();
                                        if el_type != 2 { println!("{} NOT T_STR", el_type); continue; }
                                        let char_name = String::from_utf8(rr.read_until(0).unwrap()).unwrap();
                                        if debug { println!("    add char '{}'", char_name); }
                                        charlist.push(char_name);
                                    }
                                },
                                2  /*DSTWDG*/ => {},
                                3  /*MAPIV*/ => {},
                                4  /*GLOBLOB*/ => {},
                                5  /*PAGINAE*/ => {},
                                6  /*RESID*/ => {
                                    let resid = rr.read_le_u16().unwrap();
                                    let resname = String::from_utf8(rr.read_until(0).unwrap()).unwrap();
                                    let resver = rr.read_le_u16().unwrap();
                                    println!("  RESID id:{} name:{} ver:{}", resid, resname, resver);
                                    self.resources.insert(resid, resname);
                                },
                                7  /*PARTY*/ => {},
                                8  /*SFX*/ => {},
                                9  /*CATTR*/ => {},
                                10 /*MUSIC*/ => {},
                                11 /*TILES*/ => {},
                                12 /*BUFF*/ => {},
                                13 /*SESSKEY*/ => {},
                                _ => {
                                    println!("\x1b[31m  UNKNOWN {}\x1b[39;49m", rel_type);
                                },
                            }
                        }
                        //XXX are we handle seq right in the case of overflow ???
                        self.receiver_to_sender.send(ack(seq + (rel_count - 1)));
                    },
                    2 /*ACK*/ => {
                        let seq = r.read_le_u16().unwrap();
                        if debug { println!("  seq: {}", seq); }
                    },
                    3 /*BEAT*/ => {},
                    4 /*MAPREQ*/ => {},
                    5 /*MAPDATA*/ => {},
                    6 /*OBJDATA*/ => {
                        let mut w = MemWriter::new();
                        w.write_u8(7).unwrap(); //OBJACK writer
                        while !r.eof() {
                            /*let fl =*/ r.read_u8().unwrap();
                            let id = r.read_le_u32().unwrap();
                            let frame = r.read_le_i32().unwrap();
                            if debug { println!("  id={} frame={}", id, frame); }
                            w.write_le_u32(id).unwrap();
                            w.write_le_i32(frame).unwrap();
                            let mut obj = Obj::new();
                            obj.frame = frame;
                            loop {
                                let t = r.read_u8().unwrap() as uint;
                                if debug { if t < objdata_types.len() { println!("    {}", objdata_types[t]); } }
                                match t {
                                    0   /*OD_REM*/ => {},
                                    1   /*OD_MOVE*/ => {
                                        let (x,y) = (r.read_le_i32().unwrap(), r.read_le_i32().unwrap());
                                        /*let ia =*/ r.read_le_u16().unwrap();
                                        if debug { println!("      ({},{})", x, y); }
                                        obj.x = x;
                                        obj.y = y;
                                    },
                                    2   /*OD_RES*/ => {
                                        let mut resid = r.read_le_u16().unwrap();
                                        if (resid & 0x8000) != 0 {
                                            resid &= !0x8000;
                                            let sdt_len = r.read_u8().unwrap() as uint;
                                            let _/*sdt*/ = r.read_exact(sdt_len).unwrap();
                                        }
                                        obj.resid = resid;
                                    },
                                    3   /*OD_LINBEG*/ => {
                                        /*let s =*/ (r.read_le_i32().unwrap(), r.read_le_i32().unwrap());
                                        /*let t =*/ (r.read_le_i32().unwrap(), r.read_le_i32().unwrap());
                                        let _/*c*/ = r.read_le_i32();
                                    },
                                    4   /*OD_LINSTEP*/ => {
                                        let l = r.read_le_i32().unwrap();
                                        if debug { println!("      l={}", l); }
                                    },
                                    5   /*OD_SPEECH*/ => {
                                        let _/*zo*/ = r.read_le_u16();
                                        /*let text =*/ String::from_utf8(r.read_until(0).unwrap()).unwrap();
                                    },
                                    6   /*OD_COMPOSE*/ => {
                                        /*let resid =*/ r.read_le_u16().unwrap();
                                    },
                                    7   /*OD_DRAWOFF*/ => {
                                        /*let off =*/ (r.read_le_i32().unwrap(), r.read_le_i32().unwrap());
                                    },
                                    8   /*OD_LUMIN*/ => {
                                        /*let off =*/ (r.read_le_i32().unwrap(), r.read_le_i32().unwrap());
                                        /*let sz =*/ r.read_le_u16().unwrap();
                                        /*let str_ =*/ r.read_u8().unwrap();
                                    },
                                    9   /*OD_AVATAR*/ => {
                                        loop {
                                            let layer = r.read_le_u16().unwrap();
                                            if layer == 65535 { break; }
                                        }
                                    },
                                    10  /*OD_FOLLOW*/ => {
                                        let oid = r.read_le_u32().unwrap();
                                        if oid == 0xff_ff_ff_ff {
                                            /*let xfres =*/ r.read_le_u16().unwrap();
                                            /*let xfname =*/ String::from_utf8(r.read_until(0).unwrap()).unwrap();
                                        }
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
                                    },
                                    12  /*OD_OVERLAY*/ => {
                                        /*let olid =*/ r.read_le_i32().unwrap();
                                        let resid = r.read_le_u16().unwrap();
                                        if (resid & 0x8000) != 0 {
                                            let sdt_len = r.read_u8().unwrap() as uint;
                                            /*let sdt =*/ r.read_exact(sdt_len).unwrap();
                                        }
                                    },
                                    13  /*OD_AUTH*/   => { /* Removed */ },
                                    14  /*OD_HEALTH*/ => {
                                        /*let hp =*/ r.read_u8().unwrap();
                                    },
                                    15  /*OD_BUDDY*/ => {
                                        let name = String::from_utf8(r.read_until(0).unwrap()).unwrap();
                                        if name.len() > 0 {
                                            /*let group =*/ r.read_u8().unwrap();
                                            /*let btype =*/ r.read_u8().unwrap();
                                        }
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
                                    },
                                    19  /*OD_ICON*/ => {
                                        let resid = r.read_le_u16().unwrap();
                                        if resid != 65535 {
                                            /*let ifl =*/ r.read_u8().unwrap();
                                        }
                                    },
                                    255 /*OD_END*/ => { break; },
                                    _   /*UNKNOWN*/ => {}
                                }
                            }
                            self.sender_to_viewer.send((id,obj));
                        }
                        self.receiver_to_sender.send(w.unwrap()); // send OBJACKs
                    },
                    7 /*OBJACK*/ => {},
                    8 /*CLOSE*/ => {
                        self.sender_tx.send(());
                        // ??? should we send CLOSE too ???
                        break;
                    },
                    _ /*UNKNOWN*/ => {
                    }
                }

                if !r.eof() {
                    let _/*remains*/ = r.read_to_end().unwrap();
                    //println!("                       REMAINS {} bytes", remains.len());
                }

                //TODO send REL until reply
                if charlist.len() > 0 {
                    //println!("send play '{}'", charlist[0]);
                    self.receiver_to_sender.send(rel_wdgmsg_play(0, charlist[0].as_slice()));
                    charlist.clear();
                }
            }
        });
    }
    */

    fn connect (&self) {
        //TODO send SESS until reply
        //TODO get username from server responce, not from auth username
        self.main_to_sender.send(sess(self.user.as_slice(), self.cookie.as_slice()));
    }
}



fn main() {
    /*let rel_types = [
        "NEWWDG",
        "WDGMSG",
        "DSTWDG",
        "MAPIV",
        "GLOBLOB",
        "PAGINAE",
        "RESID",
        "PARTY",
        "SFX",
        "CATTR",
        "MUSIC",
        "TILES",
        "BUFF",
        "SESSKEY" ];*/

    //TODO handle keyboard interrupt

    let mut client = Client::new("game.salemthegame.com", 1871, 1870);//.unwrap();
    client.authorize("salvian", "простойпароль");//.unwrap();
    /*
    client.start_sender().unwrap();
    client.start_beater().unwrap();
    client.start_viewer().unwrap();
    client.start_receiver().unwrap();
    */
    client.connect();
    client.main_from_any.recv();

    //let cookie = match authorize(host, auth_port, user, pass) {
    //    Ok(cookie) => cookie,
    //    Err(e) => { println!("error. {}: {}", e.source, e.detail.unwrap()); return; }
    //};
    //println!("success. cookie = [{}]", cookie.as_slice().to_hex());

    let debug = true;
}




















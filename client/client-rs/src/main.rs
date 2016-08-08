extern crate openssl;
extern crate serialize;

use std::io::Writer;
use std::io::MemWriter;
use std::io::net::tcp::TcpStream;
use std::str;
use std::io::net::udp::UdpSocket;
use std::io::net::ip::SocketAddr;

use openssl::crypto::hash::{SHA256, hash};
use openssl::ssl::{Sslv23, SslContext, SslStream};
use serialize::hex::ToHex;

//#![feature(macro_rules)]
//macro_rules! tryio (
//   ($fmt:expr $e:expr) => (
//       (match $e { Ok(e) => e, Err(e) => return Err(MyError{source:$fmt, detail:e.detail}) })
//   )
//)

struct MyError {
    source: &'static str,
    detail: Option<String>,
}

fn authorize (host: &str, port: u16, user: &str, pass: &str) -> Result<Vec<u8>, MyError> {
    println!("authorize at {}:{}", host, port);
    //let stream = TcpStream::connect(host, port).unwrap();
    let stream = match TcpStream::connect(host, port) {
        Ok(e)=>e,
        Err(e)=>return Err(MyError{source:"connect", detail:e.detail})
    };
    //let stream = tryio!(TcpStream::connect(host, port));

    let mut stream = SslStream::new(&SslContext::new(Sslv23).unwrap(), stream).unwrap();

    // send 'pw' command
    // TODO form buffer and send all with one call
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
        return Err(MyError{source:"unexpected server answer", detail:Some(String::from_utf8(msg).unwrap())});
    }

    // send 'cookie' command
    if (msg[0] == ('o' as u8)) && (msg[1] == ('k' as u8)) {
        // TODO form buffer and send all with one call
        stream.write_be_u16(("cookie".as_bytes().len()+1) as u16).unwrap();
        stream.write("cookie".as_bytes()).unwrap();
        stream.write_u8(0u8).unwrap();
        stream.flush().unwrap();
        let length = stream.read_be_u16().ok().expect("read error");
        let msg = stream.read_exact(length as uint).ok().expect("read error");
        //println!("msg='{}'", str::from_utf8(msg.as_slice()).unwrap());
        println!("msg='{}'", msg.as_slice().to_hex());
        //TODO check cookie length
        return Ok(msg.slice_from(3).to_vec());
    }
    return Err(MyError{source:"unexpected server answer", detail:Some(String::from_utf8(msg).unwrap())});
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



/* CONCEPT:
     client.connect()
        start receiver thread
        start transmitter thread
        add task.sess
            while not acked { send sess }
            if sess err != ok => fail
            else {set connected, add task.beat(every 5 sec)}
        add task.wait_for_login_screen_ui
        add task.wdg_msg(0, "focus", 1)
     client.choice("Lemming")
     client...

     client.receiver
        save and ack all rel
*/



fn main() {
    use std::io::net::addrinfo::get_host_addresses;
    use std::io::net::ip::Ipv4Addr;
    use std::io::MemReader;
    use std::collections::smallintmap::SmallIntMap;
    use std::str::from_utf8;

    let host = "game.salemthegame.com";
    let host_ip = get_host_addresses(host).unwrap()[0];
    //let addrs = get_host_addresses(host).unwrap();
    //println!("host ip: {}", addrs);
    //TODO get first ipv4 addr as host addr
    let auth_port: u16 = 1871;
    let port: u16 = 1870;
    let user = "salvian";
    let pass = "простойпароль";

    let cookie = match authorize(host, auth_port, user, pass) {
        Ok(cookie) => cookie,
        Err(e) => { println!("error. {}: {}", e.source, e.detail.unwrap()); return; }
    };
    println!("success. cookie = [{}]", cookie.as_slice().to_hex());

    let host_addr = SocketAddr {ip:host_ip, port:port};
    let any_addr  = SocketAddr {ip:Ipv4Addr(0,0,0,0), port:0u16};
    let mut udp_rx = UdpSocket::bind(any_addr).unwrap();
    let mut udp_tx = udp_rx.clone();

    let (main_tx, sender_rx) = channel();
    let (sender_tx, main_rx) = channel();

    // UDP sender
    spawn(proc() {
        loop {
            let buf: Vec<u8> = sender_rx.recv();
            println!("sender: send {} bytes", buf.len());
            udp_tx.send_to(buf.as_slice(), host_addr).unwrap();
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

    let sess_errors = [
        "OK",
        "AUTH",
        "BUSY",
        "CONN",
        "PVER",
        "EXPR" ];

    let rel_types = [
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
        "SESSKEY" ];

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

    let beater_to_sender = main_tx.clone();
    let (receiver_to_beater, from_receiver) = channel();
    // BEATer
    spawn(proc() {
        use std::io::timer;
        use std::time::Duration;

        from_receiver.recv();
        //send BEAT every 5 sec
        loop {
            beater_to_sender.send(beat());
            timer::sleep(Duration::seconds(5));
        }
    });

    let receiver_to_sender = main_tx.clone();
    // UDP receiver
    spawn(proc() {
        //let mut connected = false;
        let mut buf = [0u8, ..65535];
        let mut charlist = Vec::new();
        let mut widgets = SmallIntMap::new();
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
            println!("seceiver: {}", msg_types[mtype]);
            match mtype {
                0 /*SESS*/ => {
                    let sess_error = r.read_u8().unwrap() as uint;
                    if sess_error != 0 {
                        println!("sess error {}", sess_errors[sess_error]);
                        sender_tx.send(());
                        // ??? should we send CLOSE too ???
                        break;
                    }
                    //connected = true;
                    receiver_to_beater.send(());
                },
                1 /*REL*/ => {
                    let seq = r.read_le_u16().unwrap();
                    println!("  seq: {}", seq);
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
                        if rel_type < rel_types.len() {
                            println!("  {}", rel_types[rel_type]);
                        } else {
                            println!("\x1b[31m  UNKNOWN {}\x1b[39;49m", rel_type);
                        }
                        rel_count += 1;

                        let mut rr = MemReader::new(rel);
                        match rel_type {
                            0 /*NEWWDG*/ => {
                                let wdg_id = rr.read_le_u16().unwrap();
                                let wdg_type = String::from_utf8(rr.read_until(0).unwrap()).unwrap();
                                let wdg_parent = rr.read_le_u16().unwrap();
                                //pargs = read_list
                                //cargs = read_list
                                println!("    id:{} type:{} parent:{}", wdg_id, wdg_type, wdg_parent);
                                widgets.insert(wdg_id as uint, wdg_type);
                            },
                            1 /*WDGMSG*/ => {
                                let wdg_id = rr.read_le_u16().unwrap();
                                let msg_name = String::from_utf8(rr.read_until(0).unwrap()).unwrap();
                                println!("    id:{} name:{}", wdg_id, msg_name);
                                if widgets.find(&(wdg_id as uint)).unwrap().as_slice() == "charlist\0" && msg_name.as_slice() == "add\0" {
                                    let el_type = rr.read_u8().unwrap();
                                    if el_type != 2 { println!("{} NOT T_STR", el_type); continue; }
                                    let char_name = String::from_utf8(rr.read_until(0).unwrap()).unwrap();
                                    println!("add char '{}'", char_name);
                                    charlist.push(char_name);
                                }
                            },
                            _ => {},
                        }
                    }
                    receiver_to_sender.send(ack(seq + (rel_count - 1)));
                },
                2 /*ACK*/ => {
                    let seq = r.read_le_u16().unwrap();
                    println!("  seq: {}", seq);
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
                        println!("  id={} frame={}", id, frame);
                        w.write_le_u32(id).unwrap();
                        w.write_le_i32(frame).unwrap();
                        loop {
                            let t = r.read_u8().unwrap() as uint;
                            if t < objdata_types.len() { println!("    {}", objdata_types[t]); }
                            match t {
                                0   /*OD_REM*/ => {},
                                1   /*OD_MOVE*/ => {
                                    let (x,y) = (r.read_le_i32().unwrap(), r.read_le_i32().unwrap());
                                    /*let ia =*/ r.read_le_u16().unwrap();
                                    println!("      ({},{})", x, y);
                                },
                                2   /*OD_RES*/ => {
                                    let /*mut*/ resid = r.read_le_u16().unwrap();
                                    if (resid & 0x8000) != 0 {
                                        /*resid &= !0x8000;*/
                                        let sdt_len = r.read_u8().unwrap() as uint;
                                        /*let sdt =*/ r.read_exact(sdt_len).unwrap();
                                    }
                                },
                                3   /*OD_LINBEG*/ => {
                                    /*let s =*/ (r.read_le_i32().unwrap(), r.read_le_i32().unwrap());
                                    /*let t =*/ (r.read_le_i32().unwrap(), r.read_le_i32().unwrap());
                                    /*let c =*/ r.read_le_i32();
                                },
                                4   /*OD_LINSTEP*/ => {
                                    let l = r.read_le_i32().unwrap();
                                    println!("      l={}", l);
                                },
                                5   /*OD_SPEECH*/ => {
                                    /*let zo =*/ r.read_le_u16();
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
                    }
                    receiver_to_sender.send(w.unwrap()); // send OBJACKs
                },
                7 /*OBJACK*/ => {},
                8 /*CLOSE*/ => {
                    sender_tx.send(());
                    // ??? should we send CLOSE too ???
                    break;
                },
                _ /*UNKNOWN*/ => {
                }
            }

            if !r.eof() {
                let remains = r.read_to_end().unwrap();
                println!("                       REMAINS {} bytes", remains.len());
            }

            //TODO send REL until reply
            if charlist.len() > 0 {
                println!("send play '{}'", charlist[0]);
                receiver_to_sender.send(rel_wdgmsg_play(0, charlist[0].as_slice()));
                charlist.clear();
            }
        }
    });

    //TODO send SESS until reply
    main_tx.send(sess(user.as_slice(), cookie.as_slice()));
    main_rx.recv();
}





















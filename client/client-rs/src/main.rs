#![feature(convert)]
#![feature(ip_addr)]
#![feature(lookup_host)]
#![feature(associated_consts)]

use std::net::IpAddr;
use std::net::Ipv4Addr;
use std::net::SocketAddr;

extern crate openssl;
use self::openssl::crypto::hash::Type;
use self::openssl::crypto::hash::hash;
use self::openssl::ssl::{SslMethod, SslContext, SslStream};

extern crate rustc_serialize;
use rustc_serialize::hex::ToHex;

extern crate mio;
use mio::Handler;
use mio::Token;
use mio::EventLoop;
use mio::Interest;
use mio::PollOpt;
use mio::ReadHint;
use mio::TryRead;
use mio::TryWrite;
use mio::buf::Buf;
use mio::buf::ByteBuf;
use mio::buf::RingBuf;
use mio::buf::SliceBuf;
use mio::tcp::TcpListener;
use mio::tcp::TcpStream;
use mio::udp::UdpSocket;
use mio::util::Slab;

use std::str;
use std::io::{Error, ErrorKind};
//use std::io::Write;
//use std::fs::File;

mod salem;
use salem::state::*;

mod ai;
use ai::Ai;

mod ai_lua;
use ai_lua::LuaAi;

extern crate image;
//use image::GenericImage;
//use image::ImageBuffer;
//use image::Rgb;
//use image::ImageRgb8;
//use image::PNG;

extern crate byteorder;
use self::byteorder::{LittleEndian, BigEndian, ReadBytesExt, WriteBytesExt};
#[allow(non_camel_case_types)]
type le = LittleEndian;
#[allow(non_camel_case_types)]
type be = BigEndian;

use std::vec::Vec;
use std::io::Cursor;
use std::io::Read;
use std::io::BufRead;
use std::io::Write;
use std::u16;

const UDP: Token = Token(0);
const TCP: Token = Token(1);

struct ControlConn {
    sock: TcpStream,
    //buf: Option<ByteBuf>,
    //mut_buf: Option<MutByteBuf>,
    token: Option<Token>,
    //interest: Interest,
    //url: Option<Url>,
    //text: Option<String>,
    responce: Option<String>
}

impl ControlConn {
    fn new(sock: TcpStream) -> ControlConn {
        ControlConn {
            sock: sock,
            //buf: None,
            //mut_buf: Some(ByteBuf::mut_with_capacity(2048)),
            token: None,
            //interest: Interest::hup(),
            //url: None,
            //text: None,
            responce: None,
        }
    }

    fn writable (&mut self, eloop: &mut EventLoop<AnyHandler>, /*client*/ _: &mut Client) -> std::io::Result<()> {
        //println!("{:?}: writable", self.token);
        //let mut buf = self.buf.take().unwrap();

        //let mut buf = ByteBuf::mut_with_capacity(2048);
        //buf.write_slice(b"hello there!\n");

        //let buf = "TODO".to_string();
        //match self.sock.write(&mut buf.flip()) {
        //match self.sock.write(&mut ByteBuf::from_slice(buf.as_bytes())) {
        match self.responce {
            Some(ref buf) => {
                match self.sock.try_write_buf(&mut ByteBuf::from_slice(buf.as_bytes())) {
                    Ok(None) => {
                        println!("client flushing buf; WOULDBLOCK");
                        //self.buf = Some(buf);
                        //self.interest.insert(Interest::writable());
                        if let Err(e) = eloop.reregister(&self.sock, self.token.unwrap(), Interest::writable(), PollOpt::edge() | PollOpt::oneshot()) {
                            println!("ERROR: failed to re-reg for write: {}", e);
                        }
                    }
                    Ok(Some(/*r*/_)) => {
                        //println!("CONN: we wrote {} bytes!", r);
                        //self.mut_buf = Some(buf.flip());
                        //self.interest.insert(Interest::readable());
                        //self.interest.remove(Interest::writable());
                        //FIXME check that we wrote same byte count as self.responce.len()
                        //FIXME self.responce = None;
                        if let Err(e) = eloop.reregister(&self.sock, self.token.unwrap(), Interest::readable(), PollOpt::edge() | PollOpt::oneshot()) {
                            println!("ERROR: failed to re-reg for read: {}", e);
                        }
                    }
                    Err(e) => panic!("ERROR: not implemented; client err={:?}", e),
                }
            }
            None => {
                panic!("ERROR: conn is writable when there is no responce");
            }
        }
        //eloop.reregister(&self.sock, self.token.unwrap(), self.interest, PollOpt::edge() | PollOpt::oneshot())
        Ok(())
    }

    fn web_responce (state: &mut State, buf: &str) -> Option<String> {
        if buf.starts_with(" ") {
            let body = "<html> \r\n\
                            <head> \r\n\
                                <title></title> \r\n\
                                <script src=\"http://code.jquery.com/jquery-1.11.3.min.js\" stype=\"text/javascript\"></script> \r\n\
                                <script type=\"text/javascript\"> \r\n\
                                    $(document).ready(function(){ \r\n\
                                        $('#getdata-button').on('click', function(){ \r\n\
                                            $.get('http://localhost:33000/data', function(data) { \r\n\
                                                $('#showdata').html(\"<p>\"+data+\"</p>\"); \r\n\
                                            }); \r\n\
                                        }); \r\n\
                                    }); \r\n\
                                </script> \r\n\
                            </head> \r\n\
                            <body> \r\n\
                                <a href=\"#\" id=\"getdata-button\">C</a> \r\n\
                                <div id=\"showdata\"></div> \r\n\
                            </body> \r\n\
                        </html>\r\n\r\n";
            Some(format!("HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\n\r\n", body.len()) + &body)
        } else if buf.starts_with("env ") {
            // {
            //   res:[{id:id,name:name}],
            //   obj:[{},{}],
            //   wid:[{},{}],
            //   map:[z,z,...,z]
            // }

            let mut body = "{\"res\":[".to_string();

            let mut period = "";
            for (id,name) in &state.resources {
                body = body + &format!("\r\n{}{{\"id\":{},\"name\":\"{}\"}}", period, id, name);
                period = ",";
            }
            
            body = body + "],\"obj\":[";

            period = "";
            for o in state.objects.values() {
                let resname = match state.resources.get(&o.resid) {
                    Some(res) => res.as_str(),
                    None      => "null"
                };
                body = body + &format!("\r\n{}{{\"x\":{},\"y\":{},\"resid\":{},\"resname\":\"{}\"}}", period, o.x, o.y, o.resid, resname);
                period = ",";
            }

            body = body + "],\"wid\":[";

            period = "";
            for (id,w) in &state.widgets {
                body = body + &format!("\r\n{}{{\"id\":{},\"name\":\"{}\",\"parent\":\"{}\"}}", period, id, w.typ, w.parent);
                period = ",";
            }

            body = body + "],\"map\":[";

            period = "";
            match state.hero_grid() {
                Some(grid) => {
                    for y in 0..100 {
                        for x in 0..100 {
                            body = body + &format!("{}{}", period, grid.z[x+y*100]);
                            period = ",";
                        }
                    }
                }
                //TODO send one Null instead of 10000 zeroes
                None => {
                    for _ in 0..100 {
                        for _ in 0..100 {
                            body = body + &format!("{}{}", period, 0);
                            period = ",";
                        }
                    }
                }
            }

            body = body + "]}";
            Some(format!("HTTP/1.1 200 OK\r\nContent-Type: text/json\r\nContent-Length: {}\r\nAccess-Control-Allow-Origin: *\r\n\r\n", body.len()) + &body)
        } else if buf.starts_with("objects ") {
            let mut body = String::new();
            for o in state.objects.values() {
                let resname = match state.resources.get(&o.resid) {
                    Some(res) => res.as_str(),
                    None      => "null"
                };
                body = body + &format!("{{\"x\":{},\"y\":{},\"resid\":{},\"resname\":\"{}\"}},", o.x, o.y, o.resid, resname);
            }
            body = "[ ".to_string() + &body[..body.len()-1] + " ]";
            Some(format!("HTTP/1.1 200 OK\r\nContent-Type: text/json\r\nContent-Length: {}\r\nAccess-Control-Allow-Origin: *\r\n\r\n", body.len()) + &body)
        } else if buf.starts_with("widgets ") {
            let mut body = String::new();
            for (id,w) in &state.widgets {
                body = body + &format!("{{\"id\":{},\"name\":\"{}\",\"parent\":\"{}\"}},", id, w.typ, w.parent);
            }
            body = "[ ".to_string() + &body[..body.len()-1] + " ]";
            Some(format!("HTTP/1.1 200 OK\r\nContent-Type: text/json\r\nContent-Length: {}\r\nAccess-Control-Allow-Origin: *\r\n\r\n", body.len()) + &body)
        } else if buf.starts_with("resources ") {
            //TODO
            Some("HTTP/1.1 404 Not Implemented\r\n\r\n".to_string())
        } else if buf.starts_with("go/") {
            //FIXME should NOT be implemented for web. web is for view only
            //println!("GO: {} {}", x, y);
            //if let Err(e) = client.go(x,y) {
            //    println!("ERROR: client.go: {:?}", e);
            //}
            let tmp1: Vec<&str> = buf.split(' ').collect();
            println!("TMP1: {:?}", tmp1);
            let tmp2: Vec<&str> = tmp1[1].split('/').collect();
            println!("TMP2: {:?}", tmp2);
            if tmp2.len() > 3 {
                let /*x*/_: i32 = match str::FromStr::from_str(tmp2[2]) { Ok(v) => v, Err(_) => 0 };
                let /*y*/_: i32 = match str::FromStr::from_str(tmp2[3]) { Ok(v) => v, Err(_) => 0 };
                //self.url = Some(Url::Go(x,y));
            } else {
                //self.url = Some(Url::Go(0,0));
            }
            Some("HTTP/1.1 200 OK\r\n\r\n".to_string())
        } else if buf.starts_with("quit ") {
            if let Err(e) = state.close() {
                println!("ERROR: client.close: {:?}", e);
            }
            Some("HTTP/1.1 200 OK\r\n\r\n".to_string())
        } else {
            Some("HTTP/1.1 404 Not Found\r\n\r\n".to_string())
        }
    }

    fn readable (&mut self, eloop: &mut EventLoop<AnyHandler>, state: &mut State, ai: &mut Ai) -> std::io::Result<()> {
        //println!("{:?}: readable", self.token);
        //self.url = None;
        //self.text = None;
        self.responce = None;
        //let mut buf = self.mut_buf.take().expect("mut_buf.take");
        let mut buf = ByteBuf::mut_with_capacity(2048);
        match self.sock.try_read_buf(&mut buf) {
            Ok(None) => {
                println!("We just got readable, but were unable to read from the socket?");
                eloop.shutdown();
            }
            Ok(Some(0)) => {
                println!("read zero bytes. de-reg this conn");
                if let Err(e) = eloop.deregister(&self.sock) {
                    println!("deregister error: {}", e);
                }
                return Err(Error::new(ErrorKind::Other, "read zero bytes"));
            }
            Ok(Some(/*r*/ _)) => {
                //println!("{:?}: read {} bytes", self.token, r);
                let buf = buf.flip();
                let buf = String::from_utf8_lossy(buf.bytes()).into_owned();
                //println!("CONN read: {}", buf);
                if buf.starts_with("GET /") {
                    let pattern: &[_] = &['\r','\n'];
                    let crlf = buf.find(pattern).unwrap_or(buf.len());
                    self.responce = ControlConn::web_responce(state, &buf[5..crlf]);
                } else {

                    //TODO wrap buf into coroutine and execute it
                    println!("EXEC: {}", buf.as_str());
                    ai.exec(buf.as_str());
                    self.responce = Some("ok\n".to_string());

                    /*
                    if buf.starts_with("q") { // quit
                        match client.close() {
                            Ok(_) => self.text = Some("ok\n".to_string()),
                            Err(_) => self.text = Some("ERROR\n".to_string()),
                        }
                    } else if buf.starts_with("g ") { // go
                        //FIXME use something more siutable. like sscanf()
                        let tmp: Vec<&str> = buf.split(' ').collect();
                        if tmp.len() > 2 {
                            let x: i32 = match str::FromStr::from_str(tmp[1]) { Ok(v) => v, Err(_) => 0 };
                            let y: i32 = match str::FromStr::from_str(tmp[2]) { Ok(v) => v, Err(_) => 0 };
                            let (hx,hy) = client.hero_xy();
                            if let Err(e) = client.go(hx+x,hy+y) {
                                println!("ERROR: client.go: {:?}", e);
                            }
                            self.text = Some("ok\n".to_string());
                        } else {
                            self.text = Some("ERROR\n".to_string());
                        }
                    } else if buf.starts_with("p ") { // pick
                        //FIXME use something more siutable. like sscanf()
                        let tmp: Vec<&str> = buf.split(' ').collect();
                        if tmp.len() > 1 {
                            let obj_id: u32 = match str::FromStr::from_str(tmp[1]) { Ok(v) => v, Err(_) => 0 };
                            if let Err(e) = client.pick(obj_id) {
                                println!("ERROR: client.pick: {:?}", e);
                            }
                            self.text = Some("ok\n".to_string());
                        } else {
                            self.text = Some("ERROR\n".to_string());
                        }
                    } else if buf.starts_with("cp ") { // choose pick
                        //FIXME use something more siutable. like sscanf()
                        let tmp: Vec<&str> = buf.split(' ').collect();
                        if tmp.len() > 1 {
                            let widget_id: u16 = match str::FromStr::from_str(tmp[1]) { Ok(v) => v, Err(_) => 0 };
                            if let Err(e) = client.choose_pick(widget_id) {
                                println!("ERROR: client.choose_pick: {:?}", e);
                            }
                            self.text = Some("ok\n".to_string());
                        } else {
                            self.text = Some("ERROR\n".to_string());
                        }
                    } else if buf.starts_with("i") { // inventory
                        let mut s = String::new();
                        for (xy,resid) in &client.hero.inventory {
                            let &(x,y) = xy;
                            let resname = match client.resources.get(&resid) {
                                Some(res) => res.as_str(),
                                None      => "null"
                            };
                            s = s + &format!("({:2} {:2}) {}\n", x, y, resname);
                        }
                        self.text = Some(s);
                    } else if buf.starts_with("export z") { // export current grid z coordinates to .OBJ
                        //TODO move to fn client.current_map
                        let mut f = try!(File::create("z.obj"));
                        let grid = client.hero_grid();
                        for y in 0..100 {
                            for x in 0..100 {
                                try!(f.write_all(format!("v {} {} {}\n", (y as f32)/50., (grid.z[x+y*100] as f32)/200., (x as f32)/50.).as_bytes()));
                            }
                        }
                        for y in 0..99 {
                            for x in 0..99 {
                                let a = 1+y*100+x;
                                let b = a+1;
                                let c = b+100;
                                let d = c-1;
                                try!(f.write_all(format!("f {} {} {} {}\n", a, b, c, d).as_bytes()));
                            }
                        }
                        self.text = Some("ok\n".to_string());
                    } else if buf.starts_with("export tiles") { // export current grid tiles to .PNG
                        //TODO move to fn client.current_map
                        let mut f = try!(File::create("tiles.png"));
                        let mut img = ImageBuffer::new(100, 100);
                        let grid = client.hero_grid();
                        for y in 0..100 {
                            for x in 0..100 {
                                let color = grid.tiles[y*100+x];
                                img.put_pixel(x as u32, y as u32, Rgb([color,color,color]));
                            }
                        }
                        ImageRgb8(img).save(&mut f, PNG).unwrap();
                        self.text = Some("ok\n".to_string());
                    } else if buf.starts_with("o") { // print objects
                        let mut minx = std::i32::MAX;
                        let mut miny = std::i32::MAX;
                        for o in client.objects.values() {
                            if o.x < minx { minx = o.x; }
                            if o.y < miny { miny = o.y; }
                        }
                        let mut s = String::new();
                        for o in client.objects.values() {
                            let resname = match client.resources.get(&o.resid) {
                                Some(res) => res.as_str(),
                                None      => "null"
                            };
                            //TODO let (x,y) = o.xy - client.hero.xy;
                            let (hx,hy) = client.hero_xy();
                            let rx = o.x - hx;
                            let ry = o.y - hy;
                            let distance = ((rx*rx + ry*ry) as f32).sqrt(); //TODO dist(o.xy, client.hero.xy);
                            //s = s + &format!("({:7}, {:7}) ({:2},{:2}) ({:4}, {:4}) {:5.1} {}\n", o.x, o.y, o.x%11, o.y%11, rx, ry, distance, o.resid, resname);
                            s = s + &format!("({:7}, {:7}) ({:2},{:2}) ({:4}, {:4}) {:5.1} {}\n", o.x, o.y, o.x%11, o.y%11, rx, ry, distance, resname);
                        }
                        self.text = Some(s);
                    } else if buf.starts_with("w") { // print widgets
                        let mut s = String::new();
                        for (id,w) in &client.widgets {
                            let name = match w.name {
                                Some(ref n) => { n.clone() }
                                None        => { "-".to_string() }
                            };
                            s = s + &format!("{:3}<{:3} {:12} {:4}\n", id, w.parent, w.typ, name);
                        }
                        self.text = Some(s);
                    } else if buf.starts_with("s") {
                        let mut s = String::new();
                        //TODO let (x,y) = o.xy - client.hero.xy;
                        let (x,y) = client.hero_xy();
                        let mut mindist = std::f32::MAX;
                        let mut obj = None;
                        for o in client.objects.values() {
                            if o.id != client.hero.obj.unwrap() {
                                let dx = o.x - x;
                                let dy = o.y - y;
                                let distance = ((dx*dx + dy*dy) as f32).sqrt(); //TODO dist(o.xy, client.hero.xy);
                                if distance < mindist {
                                    mindist = distance;
                                    obj = Some(o);
                                }
                            }
                        }
                        let resname = match client.resources.get(&obj.unwrap().resid) {
                            Some(res) => res.as_str(),
                            None      => "null"
                        };
                        let (gridx,gridy) = client.hero_grid_xy();
                        let relx = x - gridx * 1100;
                        let rely = y - gridy * 1100;
                        let obj = obj.unwrap();
                        s = s + &format!("        xy: {},{}\n      \
                                          grid: {},{}\n    \
                                          rel xy: {} {}\n      \
                                          tile: {},{}\n\
                                          xy in tile: {} {}\n\
                                          near: ({}, {}) {:5.1} {} {}\n",
                                                      x, y,
                                                      gridx, gridy,
                                                      relx, rely,
                                                      relx / 11, rely / 11,
                                                      relx % 11, rely % 11,
                                                      obj.x, obj.y, mindist, obj.id, resname);
                        self.text = Some(s);
                    }*/ /*else if buf.starts_with("export ol") { // export current grid ol to .txt
                        //TODO move to fn client.current_map
                        let mut f = try!(File::create("ol.txt"));
                        let hero_obj: &Obj = client.objects.get(&client.hero.obj.unwrap()).unwrap();
                        let mx:i32 = hero_obj.x / 1100;
                        let my:i32 = hero_obj.y / 1100;
                        let map = client.maps.get(&(mx,my)).unwrap();
                        for y in 0..100 {
                            for x in 0..100 {
                                let symbol = match map.ol[x+y*100] {
                                    0 => b" ",
                                    1 => b"+",
                                    2 => b"-",
                                    4 => b"=",
                                    8 => b"O",
                                    16 => b"!",
                                    _ => b"~",
                                };
                                try!(f.write_all(symbol));
                            }
                            try!(f.write_all(b"\n"));
                        }
                        self.text = Some("ok\n".to_string());
                    } */
                }
                //self.interest.remove(Interest::readable());
                //self.interest.insert(Interest::writable());
                eloop.reregister(&self.sock, self.token.unwrap(), Interest::writable(), PollOpt::edge()).unwrap();
            }
            Err(e) => {
                println!("not implemented; client err={:?}", e);
                //self.interest.remove(Interest::readable());
                eloop.shutdown();
            }

        };
        // prepare to provide this to writable
        //FIXME self.buf = Some(buf);
        //FIXME eloop.reregister(&self.sock, self.token.unwrap(), self.interest, PollOpt::edge())
        Ok(())
    }
}

struct AnyHandler<'a, T> {
    sock: UdpSocket,
    addr: std::net::SocketAddr,
    client: &'a mut Client<'a, T>,
    counter: usize,
    tcp_listener: TcpListener,
    conns: Slab<ControlConn>,
    ai: &'a mut Ai,
}

impl<'a> AnyHandler<'a> {
    fn new(sock: UdpSocket, tcp_listener: TcpListener, addr: std::net::SocketAddr, client: &'a mut Client, ai: &'a mut Ai) -> AnyHandler<'a> {
        AnyHandler {
            sock: sock,
            addr: addr,
            client: client,
            counter: 0,
            tcp_listener: tcp_listener,
            conns: Slab::new_starting_at(Token(2), 128),
            ai: ai,
        }
    }

    fn accept (&mut self, eloop: &mut EventLoop<AnyHandler>) -> std::io::Result<()> {
        println!("TCP: new connection");
        let tcp_stream = self.tcp_listener.accept().unwrap().unwrap();
        let conn = ControlConn::new(tcp_stream);
        let tok = self.conns.insert(conn).ok().expect("could not add connection to slab");
        self.conns[tok].token = Some(tok);
        eloop.register_opt(&self.conns[tok].sock, tok, Interest::readable(), PollOpt::edge() | PollOpt::oneshot()).ok().expect("could not reg IO for new conn");
        Ok(())
    }
    
    fn conn_readable (&mut self, eloop: &mut EventLoop<AnyHandler>, tok: Token) -> std::io::Result<()> {
        //println!("conn readable; tok={:?}", tok);
        //if let Err(e) = self.conn(tok).readable(eloop) {
        if let Err(_) = self.conns[tok].readable(eloop, self.client.state, self.ai) {
            self.conns.remove(tok);
        }
        Ok(())
    }

    fn conn_writable (&mut self, eloop: &mut EventLoop<AnyHandler>, tok: Token) -> std::io::Result<()> {
        //println!("conn writable; tok={:?}", tok);
        //self.conn(tok).writable(eloop)
        self.conns[tok].writable(eloop, self.client)
    }

    /*
    fn conn<'b> (&'b mut self, tok: Token) -> &'b mut ControlConn {
        &mut self.conns[tok]
    }
    */
}

impl<'a> Handler for AnyHandler<'a> {
    type Timeout = usize;
    type Message = ();

    fn readable(&mut self, eloop: &mut EventLoop<AnyHandler>, token: Token, _: ReadHint) {
        match token {
            UDP => {
                let mut rx_buf = RingBuf::new(65535);
                self.sock.recv_from(&mut rx_buf).ok().expect("sock.recv");
                let buf: &[u8] = Buf::bytes(&rx_buf);
                if let Err(e) = self.client.state.rx(buf) {
                    println!("ERROR: client.rx: {:?}", e);
                    eloop.shutdown();
                }
            },
            TCP => {
                self.accept(eloop).ok().expect("TCP.accept");
            }
            i => {
                self.conn_readable(eloop, i).unwrap();
            }
        }

        self.client.ai.update(&mut self.client.state);
    }

    fn writable(&mut self, eloop: &mut EventLoop<AnyHandler>, token: Token) {
        match token {
            UDP => {
                match self.client.state.tx() {
                    Some(ebuf) => {
                        self.counter += 1;
                        //if self.counter % 3 == 0 {
                            let mut buf = SliceBuf::wrap(ebuf.buf.as_slice());
                            if let Err(e) = self.sock.send_to(&mut buf, &self.addr) {
                                println!("ERROR: send_to error: {}", e);
                                eloop.shutdown();
                            }
                        //} else {
                        //    println!("DROPPED!");
                        //}
                        
                        if let Some(timeout) = ebuf.timeout {
                            //TODO use returned timeout handle to cancel timeout
                            //println!("set {} timeout {} ms", timeout.seq, timeout.ms);
                            if let Err(e) = eloop.timeout_ms(timeout.seq, timeout.ms) {
                                println!("eloop.timeout FAILED: {:?}", e);
                                eloop.shutdown();
                            }
                        }
                    },
                    None => {}
                }
            }
            TCP => {
                println!("ERROR: writable on tcp listener");
                eloop.shutdown();
            }
            _ => {
                if let Err(e) = self.conn_writable(eloop, token) {
                    println!("ERROR: {:?} conn_writable: {}", token, e);
                }
            }
        }
    }

    fn timeout (&mut self, /*eloop*/ _: &mut EventLoop<AnyHandler>, timeout: usize) {
        self.client.state.timeout(timeout);
    }
}

/* TODO
extern crate nix;
use nix::sys::socket::setsockopt;
use nix::sys::socket::SockLevel;
use nix::sys::socket::SockOpt;

#[derive(Debug,Copy,Clone)]
struct BindToDevice {
    dev_name: &'static str
}

impl BindToDevice {
    fn new (dev_name: &'static str) -> BindToDevice {
        BindToDevice{ dev_name: dev_name}
    }
}

impl SockOpt for BindToDevice {
    type Val = &'static str;
    fn get (&self, fd: RawFd, level: c_int) -> Result<&'static str> { ... }
    fn set () -> ? { ... }
}

//char *opt;
//opt = "eth0";
//setsockopt(sd, SOL_SOCKET, SO_BINDTODEVICE, opt, 4);
nix::sys::socket::setsockopt(sock.as_raw_fd, SockLevel::Socket, BindToDevice::new("wlan0"));
*/

struct DeclAi {
    useless: u64,
}

impl DeclAi {
    fn new () -> DeclAi {
        DeclAi {useless:0}
    }
}

impl Ai for DeclAi {
    fn update (&mut self, /*state*/_: &mut State) {
        println!("PRINT THIS AND DO NOTHING");
    }

    fn exec (&mut self, s: &str) {
        println!("EXEC: {}", s);
    }
    
    fn init (&mut self) {
        println!("INIT");
        self.useless = 42;
    }
}

struct Client<T: Ai> {
    pub serv_ip     : IpAddr,
    pub user        : String,
    pub pass        : String,
    pub cookie      : Vec<u8>,
    
    state: State,
    ai: T,
    //TODO driver: Driver,
}

impl Client {
    pub fn new (user: String, pass: String) -> Client {
        Client {
            serv_ip: IpAddr::V4(Ipv4Addr::new(0,0,0,0)), //TODO use Option(IpAddr)
            user: user,
            pass: pass,
            cookie: Vec::new(),
            state: State::new(),
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
        buf.extend("pw".as_bytes());
        buf.push(0);
        buf.extend(user);
        buf.push(0);
        let pass_hash = hash(Type::SHA256, self.pass.as_bytes());
        assert!(pass_hash.len() == 32);
        buf.extend(pass_hash.as_slice());
        stream.write(buf.as_slice()).unwrap();
        stream.flush().unwrap();

        let mut buf = vec![0,0];
        let len = stream.read(buf.as_mut_slice()).ok().expect("read error");
        if len != 2 { return Err(Error{source:"bytes read != 2",detail:None}); }
        //TODO replace byteorder crate with endian crate ???
        let mut rdr = Cursor::new(buf);
        let len = rdr.read_u16::<be>().unwrap();

        let mut msg = vec![0; len as usize];
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
            buf.extend("cookie".as_bytes());
            buf.push(0);
            stream.write(buf.as_slice()).unwrap();
            stream.flush().unwrap();

            let mut buf = vec![0,0];
            let len = stream.read(buf.as_mut_slice()).ok().expect("read error");
            if len != 2 { return Err(Error{source:"bytes read != 2",detail:None}); }
            //TODO replace byteorder crate with endian crate ???
            let mut rdr = Cursor::new(buf);
            let len = rdr.read_u16::<be>().unwrap();

            let mut msg = vec![0; len as usize];
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
}

fn main () {
    //TODO use PollOpt::edge() | PollOpt::oneshot() for UDP connection and not PollOpt::level() (see how this is doing for TCP conns)
    //TODO handle keyboard interrupt
    //TODO replace all unwraps with normal error handling
    //TODO ADD tests:
    //        for i in range(0u8, 255) {
    //            let mut v = Vec::new();
    //            v.push(i);
    //            println!("{}", Message::from_buf(v.as_slice()));
    //        }
    //TODO highlight ERRORs with RED console color
    //TODO various formatters for Message and other structs output (full, short, type only)
    //TODO print timestamps for all the printlns
    //TODO FIXME use NOM (https://github.com/Geal/nom)
    //TODO FIXME use rusty-tags (https://github.com/dan-t/rusty-tags)

    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        println!("Too few arguments");
        return;
    } else if args.len() > 3 {
        println!("Too many arguments");
        return;
    }
    
    let username = args[1].clone();
    let password = args[2].clone();

    let any = str::FromStr::from_str("0.0.0.0:0").ok().expect("any.from_str");
    let sock = UdpSocket::bound(&any).ok().expect("udp::bound");

    //FIXME sock.connect(&addr);
    //FIXME sock.set_reuseaddr(true).ok().expect("set_reuseaddr");

    let mut ai = LuaAi::new();
    //let mut ai = DeclAi::new()
    ai.init();

    let mut client = Client::new(username, password);
    match client.authorize("game.salemthegame.com", 1871) {
        Ok(()) => { println!("success. cookie = [{}]", client.cookie.as_slice().to_hex()); },
        Err(e) => { println!("authorize error: {:?}", e); return; }
    };

    let mut eloop = EventLoop::new().ok().expect("eloop.new");
    eloop.register_opt(&sock, UDP, Interest::readable() | Interest::writable(), PollOpt::level()).ok().expect("eloop.register(udp)");

    let addr: std::net::SocketAddr = str::FromStr::from_str("127.0.0.1:33000").ok().expect("any.from_str");
    let tcp_listener = TcpListener::bind(&addr).unwrap();
    eloop.register_opt(&tcp_listener, TCP, Interest::readable(), PollOpt::edge()).unwrap();

    let ip = client.serv_ip;
    let mut handler = AnyHandler::new(sock, tcp_listener, std::net::SocketAddr::new(ip, 1870), &mut client, &mut ai);
    handler.client.connect().ok().expect("client.connect()");

    println!("run event loop");
    eloop.run(&mut handler).ok().expect("Failed to run the event loop");
}

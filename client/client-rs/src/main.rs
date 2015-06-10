#![feature(convert)]
#![feature(ip_addr)]
#![feature(collections)]
#![feature(lookup_host)]

extern crate openssl;

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
use std::io::Write;
use std::fs::File;

mod salem;
use salem::client::*;
//use salem::message::Message;
//use salem::message::MessageDirection;

extern crate image;
use image::GenericImage;
use image::ImageBuffer;
use image::Rgb;
use image::ImageRgb8;
use image::PNG;

extern crate libc;
use libc::c_int;

extern crate lua;
use lua::ffi::lua_State;
use lua::State;

const UDP: Token = Token(0);
const TCP: Token = Token(1);

enum Url {
    Root,
    Objects,
    Widgets,
    Resources,
    Go(i32,i32),
    Env,
}

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

    fn writable (&mut self, eloop: &mut EventLoop<AnyHandler>, client: &mut Client) -> std::io::Result<()> {
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
                    Err(e) => println!("ERROR: not implemented; client err={:?}", e),
                }
            }
            None => {
                println!("ERROR: conn is writable when there is no responce");
            }
        }
        //eloop.reregister(&self.sock, self.token.unwrap(), self.interest, PollOpt::edge() | PollOpt::oneshot())
        Ok(())
    }

    fn web_responce (client: &mut Client, buf: &str) -> Option<String> {
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
                        for (id,name) in &client.resources {
                            body = body + &format!("\r\n{}{{\"id\":{},\"name\":\"{}\"}}", period, id, name);
                            period = ",";
                        }
                        
                        body = body + "],\"obj\":[";

                        period = "";
                        for o in client.objects.values() {
                            let resname = match client.resources.get(&o.resid) {
                                Some(res) => res.as_str(),
                                None      => "null"
                            };
                            body = body + &format!("\r\n{}{{\"x\":{},\"y\":{},\"resid\":{},\"resname\":\"{}\"}}", period, o.x, o.y, o.resid, resname);
                            period = ",";
                        }

                        body = body + "],\"wid\":[";

                        period = "";
                        for (id,w) in &client.widgets {
                            body = body + &format!("\r\n{}{{\"id\":{},\"name\":\"{}\",\"parent\":\"{}\"}}", period, id, w.typ, w.parent);
                            period = ",";
                        }

                        body = body + "],\"map\":[";

                        period = "";
                        let map = client.map.grids.get(&grid(client.hero_xy())).unwrap();
                        for y in 0..100 {
                            for x in 0..100 {
                                body = body + &format!("{}{}", period, map.z[x+y*100]);
                                period = ",";
                            }
                        }

                        body = body + "]}";
                        Some(format!("HTTP/1.1 200 OK\r\nContent-Type: text/json\r\nContent-Length: {}\r\nAccess-Control-Allow-Origin: *\r\n\r\n", body.len()) + &body)
        } else if buf.starts_with("objects ") {
                        let mut body = String::new();
                        for o in client.objects.values() {
                            let resname = match client.resources.get(&o.resid) {
                                Some(res) => res.as_str(),
                                None      => "null"
                            };
                            body = body + &format!("{{\"x\":{},\"y\":{},\"resid\":{},\"resname\":\"{}\"}},", o.x, o.y, o.resid, resname);
                        }
                        body = "[ ".to_string() + &body[..body.len()-1] + " ]";
                        Some(format!("HTTP/1.1 200 OK\r\nContent-Type: text/json\r\nContent-Length: {}\r\nAccess-Control-Allow-Origin: *\r\n\r\n", body.len()) + &body)
        } else if buf.starts_with("widgets ") {
                        let mut body = String::new();
                        for (id,w) in &client.widgets {
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
                let x: i32 = match str::FromStr::from_str(tmp2[2]) { Ok(v) => v, Err(_) => 0 };
                let y: i32 = match str::FromStr::from_str(tmp2[3]) { Ok(v) => v, Err(_) => 0 };
                //self.url = Some(Url::Go(x,y));
            } else {
                //self.url = Some(Url::Go(0,0));
            }
            Some("HTTP/1.1 200 OK\r\n\r\n".to_string())
        } else if buf.starts_with("quit ") {
            if let Err(e) = client.close() {
                println!("ERROR: client.close: {:?}", e);
            }
            Some("HTTP/1.1 200 OK\r\n\r\n".to_string())
        } else {
            Some("HTTP/1.1 404 Not Found\r\n\r\n".to_string())
        }
    }

    fn readable (&mut self, eloop: &mut EventLoop<AnyHandler>, client: &mut Client, lua: &mut lua::State) -> std::io::Result<()> {
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
                let mut buf = String::from_utf8_lossy(buf.bytes()).into_owned();
                //println!("CONN read: {}", buf);
                if buf.starts_with("GET /") {
                    let pattern: &[_] = &['\r','\n'];
                    let crlf = buf.find(pattern).unwrap_or(buf.len());
                    self.responce = ControlConn::web_responce(client, &buf[5..crlf]);
                } else {
                    //TODO wrap to fn lua_do (&str) -> String { ... }
                    let status = lua.do_string(buf.as_str());
                    self.responce = match status {
                        lua::ThreadStatus::Ok => {
                            Some("ok\n".to_string())
                        }
                        _ => {
                            match lua.to_type::<String>() {
                                Some(s) => { lua.pop(1); Some(format!("{:?}: {}\n", status, s)) }
                                None    => { Some(format!("{:?}: no info\n", status)) }
                            }
                        }
                    };

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

struct AnyHandler<'a> {
    sock: UdpSocket,
    addr: std::net::SocketAddr,
    client: &'a mut Client,
    counter: usize,
    tcp_listener: TcpListener,
    conns: Slab<ControlConn>,
    lua: &'a mut lua::State,
}

impl<'a> AnyHandler<'a> {
    fn new(sock: UdpSocket, tcp_listener: TcpListener, addr: std::net::SocketAddr, client: &'a mut Client, lua: &'a mut lua::State) -> AnyHandler<'a> {
        AnyHandler {
            sock: sock,
            addr: addr,
            client: client,
            counter: 0,
            tcp_listener: tcp_listener,
            conns: Slab::new_starting_at(Token(2), 128),
            lua: lua,
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
        if let Err(_) = self.conns[tok].readable(eloop, self.client, self.lua) {
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
    
    fn lua_update_environment (&mut self) {
        let wtype = self.lua.get_global("widgets");
        if wtype != lua::Type::Table {
            println!("ERROR: widgets type({:?}) is not a Table", wtype);
            return;
        }
        for w in self.client.widgets.values() {
            self.lua.push_string(w.typ.as_str()); // stack: table(-2) value:string(-1)
            self.lua.raw_seti(-2, w.id as i64);       // table[key] = value (table[0] = "root"), stack: table(-1)
        }
        self.lua.set_global("widgets");
    }

    fn lua_react (&mut self) {
    }
    
    fn lua_dispatch_pendings (&mut self) {
    }
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
                if let Err(e) = self.client.rx(buf) {
                    println!("ERROR: client.rx: {:?}", e);
                    eloop.shutdown();
                }
                self.lua_update_environment();
                self.lua_react();
                self.lua_dispatch_pendings();
            },
            TCP => {
                self.accept(eloop).ok().expect("TCP.accept");
            }
            i => {
                self.conn_readable(eloop, i).unwrap();
            }
        }
    }

    fn writable(&mut self, eloop: &mut EventLoop<AnyHandler>, token: Token) {
        match token {
            UDP => {
                match self.client.tx() {
                    Some(ebuf) => {
                        /*
                        if let Ok((msg,_)) = Message::from_buf(ebuf.buf.as_slice(), MessageDirection::FromClient) {
                            println!("TX: {:?}", msg);
                        } //TODO else println(ERROR:malformed message); eloop_shutdown();
                        */
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
        self.client.timeout(timeout);
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

//TODO FIXME ??? maybe it's possible to pass the client handler here inside the lua_State struct?
//TODO FIXME     maybe even have to modificate lua_State struct
extern "C" fn go (l: *mut lua_State) -> c_int {
    let mut lua = lua::State::from_ptr(l);
    let x = lua.to_integer(1);
    let y = lua.to_integer(2);
    println!("LUA: go: ({},{})", x, y);
    lua.push("client");
    lua.get_table(lua::REGISTRYINDEX);
    let addr = lua.to_integer(-1);
    println!("LUA: go: we received '{}'", addr);
    0
}

fn lua_init (lua: &mut lua::State) {
    //TODO FIXME re-direct all output from stdio to user TCP connection

    lua.open_libs();

    lua.push_string("client");
    lua.push_integer(42);
    lua.set_table(lua::REGISTRYINDEX);
    
    lua.push_fn(Some(go));
    lua.set_global("go");
/*
    lua.new_table();         // stack: table(1,-1)
    lua.push_integer(0);     // stack: table(1,-2) key:int(2,-1)
    lua.push_string("root"); // stack: table(1,-3) key:int(2,-2) value:string(3,-1)
    lua.set_table(-3);       // table[key] = value, stack: table(-1)
    lua.set_global("widgets");
*/
    lua.new_table();         // stack: table(-1)
    lua.push_string("root"); // stack: table(-2) value:string(-1)
    lua.raw_seti(-2, 0);       // table[key] = value (table[0] = "root"), stack: table(-1)
    lua.set_global("widgets");
    
    //TODO wrap to fn lua_do (&str) -> String { ... }
    //TODO load lua code from file
    let status = lua.do_string("
    function wait (msec)
        print('wait ' .. msec .. ' msec')
        -- TODO
        coroutine.yield()
    end
    
    function go_rel (x, y)
        print('go_rel (' .. x .. ',' .. y .. ') msec')
        -- TODO
        coroutine.yield()
    end
    
    function quit ()
        print('quit')
        -- TODO
        coroutine.yield()
    end
    
    main = function ()
        local run = true
        while run == true do
            go_rel(-10,0) -- going up 10 pixels related to our current position
            wait(1000)    -- do nothing for 1 second
            go_rel(0,10)  -- going to position related to our current
            wait(1000)    -- do nothing for 1 second
            go_rel(10,0)  -- going to position related to our current
            wait(1000)    -- do nothing for 1 second
            go_rel(0,-10) -- going to position related to our current
            wait(1000)    -- do nothing for 1 second
        end
    end
    
    co = coroutine.create(main)
    coroutine.resume(co)
    
    ");
    match status {
        lua::ThreadStatus::Ok => {
            println!("lua init: ok\n")
        }
        _ => {
            match lua.to_type::<String>() {
                Some(s) => { lua.pop(1); println!("lua init: {:?}: {}\n", status, s); }
                None    => { println!("lua init: {:?}: no info\n", status); }
            }
        }
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

    let mut lua = lua::State::new();
    lua_init(&mut lua);

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
    let mut handler = AnyHandler::new(sock, tcp_listener, std::net::SocketAddr::new(ip, 1870), &mut client, &mut lua);
    handler.client.connect().ok().expect("client.connect()");

    println!("run event loop");
    eloop.run(&mut handler).ok().expect("Failed to run the event loop");
}

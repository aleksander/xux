#![feature(convert)]
#![feature(ip_addr)]
#![feature(lookup_host)]
#![feature(associated_consts)]

use std::net::IpAddr;
//use std::net::Ipv4Addr;
use std::net::SocketAddr;

extern crate openssl;
use self::openssl::crypto::hash::Type;
use self::openssl::crypto::hash::hash;
use self::openssl::ssl::{SslMethod, SslContext, SslStream};

extern crate rustc_serialize;
use rustc_serialize::hex::ToHex;

use std::str;
//use std::io::{Error, ErrorKind};
//use std::io::Write;
//use std::fs::File;

mod state;
use state::State;

mod message;
use message::Error;

mod ai;
use ai::Ai;

#[cfg(ai = "lua")]
mod ai_lua;
#[cfg(ai = "lua")]
use ai_lua::LuaAi;
#[cfg(ai = "lua")]
type AiImpl = LuaAi;

//#[cfg(ai = "decl")]
mod ai_decl;
//#[cfg(ai = "decl")]
use ai_decl::DeclAi;
//#[cfg(ai = "decl")]
type AiImpl = DeclAi;

//extern crate image;
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
//use std::io::BufRead;
use std::io::Write;
//use std::u16;

#[cfg(driver = "mio")]
mod driver_mio;

//TODO #[cfg(driver = "std")]
mod driver_std;
use driver_std::Driver;
use driver_std::Event;

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

mod web {

    use state::State;
    use std::str;

    pub fn render (buf: &[u8], state: &State) -> String {
        let buf = str::from_utf8(buf).unwrap();
        println!("render: {:?}", buf);
        if buf.starts_with("GET /") {
            let pattern: &[_] = &['\r','\n'];
            let crlf = buf.find(pattern).unwrap_or(buf.len());
            responce(state, &buf[5..crlf]).unwrap_or("HTTP/1.1 404 Not Found\r\n\r\n".to_string())
        } else {
            "HTTP/1.1 404 Not Found\r\n\r\n".to_string()
        }
    }

    fn responce (state: &State, buf: &str) -> Option<String> {
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
            /*FIXME if let Err(e) = state.close() {
                println!("ERROR: client.close: {:?}", e);
            }*/
            Some("HTTP/1.1 200 OK\r\n\r\n".to_string())
        } else {
            Some("HTTP/1.1 404 Not Found\r\n\r\n".to_string())
        }
    }

}

struct Client<A:Ai> {
    //pub serv_ip     : IpAddr,
    //pub user        : String,
    //pub pass        : String,
    //pub cookie      : Vec<u8>,
    state: State,
    ai: A,
    driver: Driver,
}

impl<A:Ai> Client<A> {
    pub fn new (ip: IpAddr, port: u16) -> Client<A> {
        //let mut ai = LuaAi::new();
        //let mut ai = DeclAi::new();
        let mut ai = A::new();
        ai.init();

        Client {
            //serv_ip: IpAddr::V4(Ipv4Addr::new(0,0,0,0)), //TODO use Option(IpAddr)
            //user: user,
            //pass: pass,
            //cookie: Vec::new(),
            state: State::new(),
            ai: ai,
            driver: Driver::new(ip, port).unwrap(),
        }
    }
    
    pub fn authorize (ip: IpAddr, port: u16, user: String, pass: String) -> Result<(String,Vec<u8>),Error> {
        let auth_addr = SocketAddr::new(ip, port);
        println!("authorize {} @ {}", user, auth_addr);
        let stream = std::net::TcpStream::connect(&auth_addr).unwrap();
        let context = SslContext::new(SslMethod::Sslv23).unwrap();
        let mut stream = SslStream::new(&context, stream).unwrap();

        // send 'pw' command
        let user = user.as_bytes();
        let buf_len = (3 + user.len() + 1 + 32) as u16;
        let mut buf: Vec<u8> = Vec::with_capacity((2 + buf_len) as usize);
        buf.write_u16::<be>(buf_len).unwrap();
        buf.extend("pw".as_bytes());
        buf.push(0);
        buf.extend(user);
        buf.push(0);
        let pass_hash = hash(Type::SHA256, pass.as_bytes());
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
        println!("msg='{}'", msg.as_slice().to_hex());
        println!("msg='{:?}'", msg.as_slice());
        if msg.len() < "ok\0\0".len() {
            return Err(Error{source:"'pw' command unexpected answer", detail:Some(String::from_utf8(msg).unwrap())});
        }
        let login = str::from_utf8(&msg[3..msg.len()-1]).unwrap().to_string();

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
            let cookie = msg[3..].to_vec();
            return Ok((login, cookie));
        }
        return Err(Error{source:"'cookie' command unexpected answer", detail:Some(String::from_utf8(msg).unwrap())});
    }

    fn run (&mut self, login: &str, cookie: &[u8]) -> Option<Error> {
        println!("connect {} / {}", login, cookie.to_hex());
        self.state.connect(login, cookie).unwrap();
        loop {
            loop {
                //TODO use iterator
                match self.state.tx() {
                    Some(ebuf) => {
                        self.driver.tx(&ebuf.buf).unwrap();
                        if let Some(timeout) = ebuf.timeout {
                            self.driver.timeout(timeout.seq, timeout.ms);
                        }
                    }
                    None => {
                        break;
                    }
                }
            }
            
            match self.driver.event().unwrap() {
                Event::Rx(buf) => {
                    //println!("event::rx: {} bytes", buf.len());
                    self.state.rx(&buf).unwrap();
                }
                Event::Timeout(seq) => {
                    //println!("event::timeout: {} seq", seq);
                    self.state.timeout(seq);
                }
                Event::Tcp((tx,buf)) => {
                    let reply = web::render(&buf, &self.state);
                    tx.send(reply);
                    //self.driver.reply(reply);
                }
            }
            
            self.ai.update(&mut self.state);
        }
    }
}

fn main () {
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
    if args.len() < 3 || args.len() > 3 {
        println!("wrong argument count");
        println!("usage: {} username password", args[0]);
        return;
    }
    
    let username = args[1].clone();
    let password = args[2].clone();

    let host = {
        let mut ips = ::std::net::lookup_host("game.salemthegame.com").ok().expect("lookup_host");
        ips.next().expect("ip.next").ok().expect("ip.next.ok")
    };
    let ip = host.ip();
    println!("connect to {}", ip);

    match Client::<AiImpl>::authorize(ip, 1871, username, password) {
        Ok((login, cookie)) => { Client::<AiImpl>::new(ip, 1870).run(&login, &cookie); }
        Err(e) => { println!("ERROR: {:?}", e); }
    }
}

























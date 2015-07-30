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

#[macro_use]
extern crate glium;

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

extern crate cgmath;

extern crate camera_controllers;

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

mod web;

mod render;

struct Client<A:Ai> {
    render: render::Render,
    state: State,
    ai: A,
    driver: Driver,
}

impl<A:Ai> Client<A> {
    pub fn new (ip: IpAddr, port: u16) -> Client<A> {
        let mut ai = A::new();
        ai.init();

        Client {
            render: render::Render::new(),
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
                    let reply = web::responce(&buf, &self.state);
                    tx.send(reply).unwrap();
                    //self.driver.reply(reply);
                }
                /*TODO:
                Event::RenderQuit => {
                    self.state.close();
                }
                */
            }

            self.ai.update(&mut self.state);

            if let Some(xy) = self.state.start_point() {
                let (start_x, start_y) = xy;
                let (start_grid_x, start_grid_y) = state::grid(xy);
                while let Some(event) = self.state.next_event() {
                    if let Err(e) = self.render.update(
                        match event {
                            state::Event::Grid(x,y,tiles,z) => render::Event::Grid(x - start_grid_x, y - start_grid_y, tiles, z),
                            state::Event::Obj(x,y)          => render::Event::Obj(x - start_x, y - start_y),
                        }
                    ) {
                        println!("render.update ERROR: {:?}", e);
                        return None /*TODO Some(e)*/;
                    }
                }
            }
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

    let ip = {
        let mut ips = ::std::net::lookup_host("game.salemthegame.com").ok().expect("lookup_host");
        let host = ips.next().expect("ip.next").ok().expect("ip.next.ok");
        host.ip()
    };
    println!("connect to {}", ip);

    match Client::<AiImpl>::authorize(ip, 1871, username, password) {
        Ok((login, cookie)) => { Client::<AiImpl>::new(ip, 1870).run(&login, &cookie); }
        Err(e) => { println!("ERROR: {:?}", e); }
    }
}

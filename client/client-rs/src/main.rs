#![feature(convert)]
#![feature(ip_addr)]
#![feature(lookup_host)]
#![feature(associated_consts)]
#![feature(read_exact)]
//#![feature(plugin)]
//#![plugin(clippy)]

use std::net::IpAddr;
//use std::net::Ipv4Addr;
use std::net::SocketAddr;

#[macro_use]
extern crate log;

extern crate fern;

extern crate openssl;
use self::openssl::crypto::hash::Type;
use self::openssl::crypto::hash::hash;
use self::openssl::ssl::{SslMethod, SslContext, SslStream};

extern crate rustc_serialize;
use rustc_serialize::hex::ToHex;

use std::str;
//use std::u16;
//use std::io::{Error, ErrorKind};
//use std::io::Write;
use std::fs::File;

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

extern crate image;
use image::GenericImage;
use image::ImageBuffer;
use image::Rgb;
use image::ImageRgb8;
use image::PNG;

extern crate byteorder;
use self::byteorder::{LittleEndian, BigEndian, ReadBytesExt, WriteBytesExt};
#[allow(non_camel_case_types)]
type le = LittleEndian;
#[allow(non_camel_case_types)]
type be = BigEndian;

//#[macro_use]
//extern crate glium;

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

//extern crate cgmath;

//extern crate camera_controllers;

extern crate ncurses;

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

mod to_unsigned;
use to_unsigned::ToUnsigned;

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
        info!("authorize {} @ {}", user, auth_addr);
        let stream = std::net::TcpStream::connect(&auth_addr).unwrap();
        let context = SslContext::new(SslMethod::Sslv23).unwrap();
        let mut stream = SslStream::new(&context, stream).unwrap();

        fn msg (buf: Vec<u8>) -> Vec<u8> {
            let mut msg = Vec::new();
            msg.write_u16::<be>(buf.len() as u16).unwrap();
            msg.extend(buf);
            msg
        }

        //TODO use closure instead (no need to pass stream)
        fn command (stream: &mut SslStream<std::net::TcpStream>, cmd: Vec<u8>) -> Result<Vec<u8>,Error> {
            try!(stream.write(msg(cmd).as_slice()));
            try!(stream.flush());

            let len = {
                let mut buf = vec![0; 2];
                try!(stream.read_exact(&mut buf));
                let mut rdr = Cursor::new(buf);
                try!(rdr.read_u16::<be>())
            };

            let mut msg = vec![0; len as usize];
            try!(stream.read_exact(msg.as_mut_slice()));
            debug!("msg: {:?}", msg);
            if (msg.len() < "ok\0".len()) || (msg[0] != b'o') || (msg[1] != b'k') || (msg[2] != 0) {
                //FIXME return raw vec in details, not String
                return Err(Error{source:"unexpected answer", detail:Some(String::from_utf8(msg).unwrap())});
            }
            Ok(msg[3..].to_vec())
        }

        let login = { 
            let mut buf = Vec::new();
            buf.extend("pw".as_bytes());
            buf.push(0);
            buf.extend(user.as_bytes());
            buf.push(0);
            buf.extend(hash(Type::SHA256, pass.as_bytes()).as_slice());
            let msg = try!(command(&mut stream, buf));
            //FIXME use read_strz analog
            str::from_utf8(&msg[..msg.len()-1]).unwrap().to_string()
        };

        let cookie = { 
            let mut buf = Vec::new();
            buf.extend("cookie".as_bytes());
            buf.push(0);
            try!(command(&mut stream, buf))
        };

        Ok((login, cookie))
    }

    fn send_all_enqueued (&mut self) {
        //TODO use iterator
        while let Some(ebuf) = self.state.tx() {
            self.driver.tx(&ebuf.buf).unwrap();
            if let Some(timeout) = ebuf.timeout {
                self.driver.timeout(timeout.seq, timeout.ms);
            }
        }
    }

    fn dispatch_single_event (&mut self) {
        match self.driver.event().unwrap() {
            Event::Rx(buf) => {
                //info!("event::rx: {} bytes", buf.len());
                self.state.rx(&buf).unwrap();
            }
            Event::Timeout(seq) => {
                //info!("event::timeout: {} seq", seq);
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
    }

    fn run (&mut self, login: &str, cookie: &[u8]) -> Option<Error> {
        info!("connect {} / {}", login, cookie.to_hex());
        self.state.connect(login, cookie).unwrap();

        fn grid2png (x: i32, y: i32, t: &[u8], z: &[i16]) {
            let mut f = File::create(format!("{} {}.png", x, y)).unwrap();
            let mut img = ImageBuffer::new(100, 100);
            //let grid = client.hero_grid();
            for y in 0..100 {
                for x in 0..100 {
                    //let color = grid.tiles[y*100+x];
                    let t = t[y*100+x];
                    let z = z[y*100+x];
                    let z = z.to_unsigned();
                    let h = (z >> 8) as u8;
                    let l = z as u8;
                    img.put_pixel(x as u32, y as u32, Rgb([t,h,l]));
                }
            }
            ImageRgb8(img).save(&mut f, PNG).unwrap();
        }

        loop {
            while let Some(event) = self.state.next_event() {
                let event = match event {
                    state::Event::Grid(x,y,tiles,z) => {
                        grid2png(x, y, &tiles, &z);
                        render::Event::Grid(x,y,tiles,z)
                    }
                    state::Event::Obj((x,y)) => {
                        render::Event::Obj(x,y)
                    }
                };
                if let Err(e) = self.render.update(event) {
                    info!("render.update ERROR: {:?}", e);
                    return None /*TODO Some(e)*/;
                }
            }
            self.send_all_enqueued();
            self.dispatch_single_event();
            self.ai.update(&mut self.state);
        }

        /*
        while let None = self.state.start_point() {
            self.send_all_enqueued();
            self.dispatch_single_event();
            self.ai.update(&mut self.state);
        }

        let (start_x, start_y) = match self.state.start_point() {
            Some(xy) => xy,
            None => panic!("this can't be")
        };

        loop {
            while let Some(event) = self.state.next_event() {
                let event = match event {
                    state::Event::Grid(x,y,tiles,z) => render::Event::Grid(x * 1100 - start_x, y * 1100 - start_y, tiles, z),
                    state::Event::Obj((x,y))        => render::Event::Obj(x - start_x, y - start_y),
                };
                //info!("event: {:?}", event);
                if let Err(e) = self.render.update(event) {
                    info!("render.update ERROR: {:?}", e);
                    return None /*TODO Some(e)*/;
                }
            }
            self.send_all_enqueued();
            self.dispatch_single_event();
            self.ai.update(&mut self.state);
        }
        */
    }
}

fn main () {
    let logger_config = fern::DispatchConfig {
        format: Box::new( |msg: &str, level: &log::LogLevel, _location: &log::LogLocation| {
            //format!("[{}][{}] {}", time::now().strftime("%Y-%m-%d][%H:%M:%S").unwrap(), level, msg)
            format!("[{}] {}", level, msg)
        }),
        output: vec![/*fern::OutputConfig::stdout(),*/ fern::OutputConfig::file("log")],
        level: log::LogLevelFilter::Trace,
    };

    if let Err(e) = fern::init_global_logger(logger_config, log::LogLevelFilter::Trace) {
        panic!("Failed to initialize global logger: {}", e);
    }
    
    //trace!("Trace message");
    //debug!("Debug message");
    //info!("Info message");
    //warn!("Warning message");
    //error!("Error message");
    
    //TODO handle keyboard interrupt
    //TODO replace all unwraps with normal error handling
    //TODO ADD tests:
    //        for i in range(0u8, 255) {
    //            let mut v = Vec::new();
    //            v.push(i);
    //            info!("{}", Message::from_buf(v.as_slice()));
    //        }
    //TODO highlight ERRORs with RED console color
    //TODO various formatters for Message and other structs output (full, short, type only)
    //TODO print timestamps for all the printlns
    //TODO FIXME use NOM (https://github.com/Geal/nom)
    //TODO FIXME use rusty-tags (https://github.com/dan-t/rusty-tags)

    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 || args.len() > 3 {
        info!("wrong argument count");
        info!("usage: {} username password", args[0]);
        return;
    }

    let username = args[1].clone();
    let password = args[2].clone();

    let ip = {
        let mut ips = ::std::net::lookup_host("game.salemthegame.com").ok().expect("lookup_host");
        let host = ips.next().expect("ip.next").ok().expect("ip.next.ok");
        host.ip()
    };
    
    info!("connect to {}", ip);

    match Client::<AiImpl>::authorize(ip, 1871, username, password) {
        Ok((login, cookie)) => {
            Client::<AiImpl>::new(ip, 1870).run(&login, &cookie);
        }
        Err(e) => {
            info!("ERROR: {:?}", e);
        }
    }
}


#![feature(lookup_host)]
#![feature(associated_consts)]
//<<<<<<< 4b4fc349b887fbcbfa197fe2b798f0d378433edf
//#![feature(read_exact)]
//#![feature(plugin)]
//#![plugin(clippy)]
//#![deny(//missing_docs,
//        missing_debug_implementations,
//        missing_copy_implementations,
//        trivial_casts,
//        trivial_numeric_casts,
//        //unsafe_code,
//        //unstable_features,
//        unused_import_braces,
//        unused_qualifications)]
//#![feature(zero_one)]
//=======
//>>>>>>> compilation fix

use std::net::IpAddr;
use std::net::SocketAddr;

#[macro_use]
extern crate log;

extern crate fern;

extern crate openssl;
use self::openssl::crypto::hash::Type;
use self::openssl::crypto::hash::hash;
use self::openssl::ssl::{SslMethod, SslContext, SslStream};

extern crate rustc_serialize;
extern crate byteorder;
use rustc_serialize::hex::ToHex;

use std::str;
//<<<<<<< 4b4fc349b887fbcbfa197fe2b798f0d378433edf
//use std::u16;
//use std::io::{Error, ErrorKind};
//use std::io::Write;
//use std::fs::File;
//=======
//>>>>>>> compilation fix

mod state;
use state::State;

mod message;
use message::Error;

mod ai;
use ai::Ai;

//TODO #[cfg(ai = "lua")]

//#[cfg(ai_lua)]
//FIXME mod ai_lua;
//#[cfg(ai_lua)]
//FIXME use ai_lua::LuaAi;
//#[cfg(ai_lua)]
//type AiImpl = LuaAi;

//TODO #[cfg(ai = "decl")]

//#[cfg(feature = "ai_decl")]
mod ai_decl;
//#[cfg(feature = "ai_decl")]
use ai_decl::AiDecl;
//#[cfg(feature = "ai_decl")]
//type AiImpl = AiDecl;

extern crate image;
use image::GenericImage;
use image::ImageBuffer;
use image::Rgb;
use image::ImageRgb8;
use image::PNG;

use byteorder::{LittleEndian, BigEndian, ReadBytesExt, WriteBytesExt};
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

//TODO #[cfg(driver = "mio")]
//#[cfg(driver_mio)]

//FIXME BROKEN! mod driver_mio;

//TODO #[cfg(driver = "std")]
//#[cfg(feature = "driver_std")]
mod driver_std;
//#[cfg(feature = "driver_std")]
use driver_std::DriverStd;
//#[cfg(feature = "driver_std")]
use driver::Event;

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

mod shift_to_unsigned;
use shift_to_unsigned::ShiftToUnsigned;

mod driver;
pub use driver::Driver;

pub fn authorize (ip: IpAddr, port: u16, user: String, pass: String) -> Result<(String,Vec<u8>),Error> {
    let auth_addr = SocketAddr::new(ip, port);
    info!("authorize {} @ {}", user, auth_addr);
    let stream = std::net::TcpStream::connect(&auth_addr).expect("tcpstream::connect");
    let context = SslContext::new(SslMethod::Sslv23).expect("sslsocket::new");
    let mut stream = SslStream::new(&context, stream).expect("sslstream::new");

    fn msg (buf: Vec<u8>) -> Vec<u8> {
        let mut msg = Vec::new();
        msg.write_u16::<be>(buf.len() as u16).expect("authorize.msg.write(buf.len)");
        msg.extend(buf);
        msg
    }

//<<<<<<< 4b4fc349b887fbcbfa197fe2b798f0d378433edf
//    //TODO use closure instead (no need to pass stream)
//    fn command (stream: &mut SslStream<std::net::TcpStream>, cmd: Vec<u8>) -> Result<Vec<u8>,Error> {
//        try!(stream.write(msg(cmd).as_slice()));
//        try!(stream.flush());
//=======
    pub fn authorize (ip: IpAddr, port: u16, user: String, pass: String) -> Result<(String,Vec<u8>),Error> {
        let auth_addr = SocketAddr::new(ip, port);
        info!("authorize {} @ {}", user, auth_addr);
        let stream = std::net::TcpStream::connect(&auth_addr).unwrap();
        let context = SslContext::new(SslMethod::Sslv23).unwrap();
        let mut stream = SslStream::connect(&context, stream).unwrap();

        fn msg (buf: Vec<u8>) -> Vec<u8> {
            let mut msg = Vec::new();
            msg.write_u16::<be>(buf.len() as u16).unwrap();
            msg.extend(buf);
            msg
        }
//>>>>>>> compilation fix

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
            return Err(Error{source:"unexpected answer", detail:Some(String::from_utf8(msg).expect("authorize.command.from_utf8(msg)"))});
        }
        Ok(msg[3..].to_vec())
    }

//<<<<<<< 54bbe1d118f2dbdd7e3a8c483fa9d874350af4f8
    let login = { 
        let mut buf = Vec::new();
        buf.extend("pw".as_bytes());
        buf.push(0);
        buf.extend(user.as_bytes());
        buf.push(0);
        buf.extend(hash(Type::SHA256, pass.as_bytes()).as_slice());
        let msg = try!(command(&mut stream, buf));
        //FIXME use read_strz analog
        str::from_utf8(&msg[..msg.len()-1]).expect("authorize.login.from_utf8()").to_string()
    };

    let cookie = { 
        let mut buf = Vec::new();
        buf.extend("cookie".as_bytes());
        buf.push(0);
        try!(command(&mut stream, buf))
    };

    Ok((login, cookie))
}
//=======
//        let login = {
//            let mut buf = Vec::new();
//            buf.extend("pw".as_bytes());
//            buf.push(0);
//            buf.extend(user.as_bytes());
//            buf.push(0);
//            buf.extend(hash(Type::SHA256, pass.as_bytes()).as_slice());
//            let msg = try!(command(&mut stream, buf));
//            //FIXME use read_strz analog
//            str::from_utf8(&msg[..msg.len()-1]).unwrap().to_string()
//        };
//
//        let cookie = {
//            let mut buf = Vec::new();
//            buf.extend("cookie".as_bytes());
//            buf.push(0);
//            try!(command(&mut stream, buf))
//        };
//>>>>>>> minor

//TODO move to Grid
//TODO grid.to_png(Mapper::first())
fn grid2png (x: i32, y: i32, t: &[u8], z: &[i16]) {
    let mut f = File::create(format!("{} {}.png", x, y)).expect("grid2png.file.create");
    let mut img = ImageBuffer::new(100, 100);
    for y in 0..100 {
        for x in 0..100 {
            let t = t[y*100+x];
            let z = z[y*100+x];
            let z = z.shift_to_unsigned();
            let h = (z >> 8) as u8;
            let l = z as u8;
            let mut r = 0;
            r |= (t >> 0) & 1; r <<= 1;
            r |= (t >> 3) & 1; r <<= 1;
            r |= (t >> 6) & 1; r <<= 1;
            r |= (h >> 4) & 1; r <<= 1;
            r |= (h >> 1) & 1; r <<= 1;
            r |= (l >> 6) & 1; r <<= 1;
            r |= (l >> 3) & 1; r <<= 1;
            r |= (l >> 0) & 1;
            let mut g = 0;
            g |= (t >> 1) & 1; g <<= 1;
            g |= (t >> 4) & 1; g <<= 1;
            g |= (t >> 7) & 1; g <<= 1;
            g |= (h >> 5) & 1; g <<= 1;
            g |= (h >> 2) & 1; g <<= 1;
            g |= (l >> 7) & 1; g <<= 1;
            g |= (l >> 4) & 1; g <<= 1;
            g |= (l >> 1) & 1;
            let mut b = 0;
            b |= (t >> 2) & 1; b <<= 1;
            b |= (t >> 5) & 1; b <<= 1;
            b |= (h >> 7) & 1; b <<= 1;
            b |= (h >> 6) & 1; b <<= 1;
            b |= (h >> 3) & 1; b <<= 1;
            b |= (h >> 0) & 1; b <<= 1;
            b |= (l >> 5) & 1; b <<= 1;
            b |= (l >> 2) & 1;
            /*
            let mut r = 0;
            r |= (t >> 2) & 1; r <<= 1;
            r |= (t >> 3) & 1; r <<= 1;
            r |= (h >> 7) & 1; r <<= 1;
            r |= (h >> 6) & 1; r <<= 1;
            r |= (h >> 1) & 1; r <<= 1;
            r |= (h >> 0) & 1; r <<= 1;
            r |= (l >> 3) & 1; r <<= 1;
            r |= (l >> 2) & 1;
            let mut g = 0;
            g |= (t >> 1) & 1; g <<= 1;
            g |= (t >> 4) & 1; g <<= 1;
            g |= (t >> 7) & 1; g <<= 1;
            g |= (h >> 5) & 1; g <<= 1;
            g |= (h >> 2) & 1; g <<= 1;
            g |= (l >> 7) & 1; g <<= 1;
            g |= (l >> 4) & 1; g <<= 1;
            g |= (l >> 1) & 1;
            let mut b = 0;
            b |= (t >> 0) & 1; b <<= 1;
            b |= (t >> 5) & 1; b <<= 1;
            b |= (t >> 6) & 1; b <<= 1;
            b |= (h >> 4) & 1; b <<= 1;
            b |= (h >> 3) & 1; b <<= 1;
            b |= (l >> 6) & 1; b <<= 1;
            b |= (l >> 5) & 1; b <<= 1;
            b |= (l >> 2) & 1;
            */
            img.put_pixel(x as u32, y as u32, Rgb([g,r,b/*t,h,l*/]));
        }
    }
    ImageRgb8(img).save(&mut f, PNG).expect("grid2png.image.save");
}

struct Client<'a,D:Driver + 'a,A:Ai + 'a> {
    render: render::Render,
    state: State,
    ai: &'a mut A,
    driver: &'a mut D,
}

impl<'a,D:Driver,A:Ai> Client<'a,D,A> {
    pub fn new (driver: &'a mut D, ai: &'a mut A) -> Client<'a,D,A> {
        Client {
            render: render::Render::new(), //TODO Render trait
            state: State::new(),
            ai: ai,
            driver: driver,
        }
    }

    fn send_all_enqueued (&mut self) {
        //TODO use iterator
        while let Some(ebuf) = self.state.tx() {
            self.driver.tx(&ebuf.buf).expect("send_all_enqueued");
            if let Some(timeout) = ebuf.timeout {
                self.driver.timeout(timeout.seq, timeout.ms);
            }
        }
    }

    fn dispatch_single_event (&mut self) {
        match self.driver.event().expect("dispatch_single_event.event") {
            Event::Rx(buf) => {
                //info!("event::rx: {} bytes", buf.len());
                self.state.rx(&buf).expect("dispatch_single_event.rx");
            }
            Event::Timeout(seq) => {
                //info!("event::timeout: {} seq", seq);
                self.state.timeout(seq);
            }
            Event::Tcp((tx,buf)) => {
                let reply = web::responce(&buf, &self.state);
                tx.send(reply).expect("dispatch_single_event.send");
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
        self.state.connect(login, cookie).expect("run.connect");

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
            None => unreachable!() //panic!("this can't be")
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


//TODO fn run_std_lua() { run::<Std,Lua>() }
//TODO fn run<D,A>(ip: IpAddr, username: String, password: String) where D:Driver,A:Ai {
fn run(ip: IpAddr, username: String, password: String) {
    /*ip: IpAddr, port: u16*/
    let mut ai = AiDecl::new();
    ai.init();
    let mut driver = DriverStd::new(ip, 1870).expect("driver::new");

    match authorize(ip, 1871, username, password) {
        Ok((login, cookie)) => {
            Client::new(/*ip, 1870*/&mut driver, &mut ai).run(&login, &cookie);
        }
        Err(e) => {
            info!("ERROR: {:?}", e);
        }
    }
}

fn main () {
    let mut log_open_options = std::fs::OpenOptions::new();
    let log_open_options = log_open_options.create(true).read(true).write(true).truncate(true);
    let logger_config = fern::DispatchConfig {
        format: Box::new( |msg: &str, level: &log::LogLevel, _location: &log::LogLocation| {
            //format!("[{}][{}] {}", time::now().strftime("%Y-%m-%d][%H:%M:%S").unwrap(), level, msg)
            //TODO prefix logs with timestamp(absolute/relative), file name, line number, function name
            format!("[{}] {}", level, msg)
        }),
        output: vec![
            fern::OutputConfig::stdout(), //TODO colorize stdout output: ERROR is RED, WARN is YELLOW etc
            //fern::OutputConfig::file_with_options("log", &log_open_options)
        ],
        level: log::LogLevelFilter::Trace,
    };

    if let Err(e) = fern::init_global_logger(logger_config, log::LogLevelFilter::Trace) {
        panic!("Failed to initialize global logger: {}", e);
    }
    
    trace!("Starting...");
    debug!("Starting...");
    info!("Starting...");
    warn!("Starting...");
    error!("Starting...");
    
    //TODO handle keyboard interrupt
    //TODO replace all unwraps and expects with normal error handling
    //TODO various formatters for Message and other structs output (full "{:f}", short "{:s}", type only "{:t}")
    //TODO use rustfmt precommit hook

    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 || args.len() > 3 {
        info!("wrong argument count");
        info!("usage: {} username password", args[0]);
        return;
    }

    let username = args[1].clone();
    let password = args[2].clone();

    let ip = {
        let mut ips = ::std::net::lookup_host("game.salemthegame.com").expect("lookup_host");
        let host = ips.next().expect("ip.next");
        host.ip()
    };

    info!("connect to {}", ip);
    
    //run::<DriverMio,AiLua>(ip, username, password);
    run(ip, username, password);
}

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

mod render {
    use std::sync::mpsc::channel;
    //use std::sync::mpsc::Receiver;
    use std::sync::mpsc::Sender;
    use std::thread;
    //use state::State;

    #[derive(Debug)]
    pub enum Event {
        Grid(i32,i32,Vec<u8>,Vec<i16>),
    }

    pub struct Render {
        tx: Sender<Event>,
    }

    impl Render {
        pub fn new () -> Render {
            let (tx,rx) = channel();
            thread::spawn(move || {
                    use ::glium::DisplayBuild;
                    use ::glium::Surface;
                    use ::glium::glutin::WindowBuilder;
                    use ::glium::index::NoIndices;
                    use ::glium::VertexBuffer;
                    use ::glium::index::PrimitiveType;
                    use ::glium::glutin;
                    use ::glium::Program;
                    use ::glium::uniforms::EmptyUniforms;
                    use ::glium::draw_parameters::PolygonMode;
                    use ::glium::draw_parameters::DrawParameters;
                    use ::glium::texture::Texture2d;
                    use std::sync::mpsc::TryRecvError;

                    let display = WindowBuilder::new()
                            .with_dimensions(256, 256)
                            .with_title(format!("render"))
                            .build_glium()
                            .unwrap();

                    #[derive(Copy, Clone)]
                    struct Vertex {
                        position: [f32; 2],
                        tex_coord: [f32; 2],
                    }

                    implement_vertex!(Vertex, position, tex_coord);

                    let vertex1 = Vertex { position: [-0.8,  0.8], tex_coord: [0.0, 0.0] };
                    let vertex2 = Vertex { position: [ 0.8,  0.8], tex_coord: [1.0, 0.0] };
                    let vertex3 = Vertex { position: [ 0.8, -0.8], tex_coord: [1.0, 1.0] };
                    let vertex4 = Vertex { position: [-0.8, -0.8], tex_coord: [0.0, 1.0] };
                    let shape = vec![vertex1, vertex2, vertex4,
                                     vertex2, vertex3, vertex4];

                    let vertex_buffer = VertexBuffer::new(&display, shape);
                    let indices = NoIndices(PrimitiveType::TrianglesList);

                    let vertex_shader_src = r#"
                        #version 140
                        in vec2 position;
                        in vec2 tex_coords;
                        out vec2 v_tex_coords;
                        void main() {
                            v_tex_coords = tex_coords;
                            gl_Position = vec4(position, 0.0, 1.0);
                        }
                    "#;

                    let fragment_shader_src = r#"
                        #version 140
                        in vec2 v_tex_coords;
                        out vec4 color;
                        //uniform sampler2D tex;
                        void main() {
                            color = vec4(1.0,1.0,1.0,1.0);//texture(tex, v_tex_coords);
                        }
                    "#;

                    let program = Program::from_source(&display, vertex_shader_src, fragment_shader_src, None).unwrap();

                    let mut time_to_exit = false;

                    /*'ecto_loop:*/ loop {
                        let mut target = display.draw();
                        target.clear_color(0.1, 0.1, 0.1, 1.0);
                        let mut draw_params: DrawParameters = Default::default();
                        draw_params.polygon_mode = PolygonMode::Line;
                        target.draw(&vertex_buffer, &indices, &program, &EmptyUniforms, &draw_params).unwrap();
                        target.finish().unwrap();

                        for ev in display.poll_events() {
                            match ev {
                                glutin::Event::KeyboardInput(_, _, Some(glutin::VirtualKeyCode::Escape)) => /*break 'ecto_loop,*/{time_to_exit = true;break;}
                                glutin::Event::Closed => /*break 'ecto_loop,*/{time_to_exit = true;break;}
                                _ => ()
                            }
                        }
                        if time_to_exit { break; }

                        match rx.try_recv() {
                            Ok(value) => {
                                /*
                                match value {
                                    Event::Grid(x,y,tiles,z) => {
                                        println!("render: received Grid ({},{})", x, y);
                                        //TODO do with iterator and .map() or .zip()
                                        let mut image = Vec::new(); //TODO with_capacity
                                        for y in 0..100 {
                                            let mut row = Vec::new();
                                            for x in 0..100 {
                                                let tile = tiles[y*100 + x];
                                                row.push((tile,tile,tile));
                                            }
                                            image.push(row);
                                        }
                                        let texture = Texture2d::new(&display, image)/*.unwrap()*/;
                                        let uniforms = uniform! {
                                            tex: &texture,
                                        };
                                    }
                                }
                                */
                            }
                            Err(e) => {
                                if let TryRecvError::Disconnected = e {
                                    println!("render: disconnected");
                                    break/* 'ecto_loop*/;
                                }
                            }
                        }
                    }
            });
            Render{tx:tx}
        }

        pub fn update (&mut self, event: Event) {
            self.tx.send(event).unwrap();
        }
    }
}

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
            }

            self.ai.update(&mut self.state);

            if let Some(xy) = self.state.start_point() {
                let (start_grid_x,start_grid_y) = state::grid(xy);
                while let Some(event) = self.state.next_event() {
                    self.render.update(
                        match event {
                            state::Event::Grid(x,y,tiles,z) => render::Event::Grid(start_grid_x - x,start_grid_y - y,tiles,z)
                        }
                    );
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

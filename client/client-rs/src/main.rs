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
    use std::sync::mpsc::SendError;
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
                    use ::glium::texture::/*TODO Compressed*/Texture2d;
                    use ::glium::texture::Texture2dArray;
                    use std::sync::mpsc::TryRecvError;
                    use cgmath;
                    use cgmath::Matrix;
                    use cgmath::FixedArray;

                    let display = WindowBuilder::new()
                            .with_dimensions(512, 512)
                            .with_title(format!("render"))
                            .build_glium()
                            .unwrap();

                    #[derive(Copy, Clone)]
                    struct Vertex {
                        v_pos: [f32; 3],
                        v_col: u8,
                    }

                    implement_vertex!(Vertex, v_pos, v_col);

                    let mut vertex_buffer: VertexBuffer<Vertex> = VertexBuffer::empty(&display, 0).unwrap();
                    let indices = NoIndices(PrimitiveType::TrianglesList);

                    let vertex_shader_src = r#"
                        #version 140
                        in vec3 v_pos;
                        in uint v_col;
                        flat out uint vv_col;

                        uniform mat4 u_model;
                        uniform mat4 u_view;
                        uniform mat4 u_proj;

                        void main() {
                            vv_col = v_col;
                            gl_Position = u_proj * u_view * u_model * vec4(v_pos, 1.0);
                        }
                    "#;

                    let fragment_shader_src = r#"
                        #version 140
                        flat in uint vv_col;
                        out vec4 color;
                        void main() {
                            float c = float(vv_col) / 255.0;
                            color = vec4(c, c, c, 0.0);
                        }
                    "#;

                    //FIXME don't do init here. move it to Render struct new()
                    let program = match Program::from_source(&display, vertex_shader_src, fragment_shader_src, None) {
                        Ok(program) => program,
                        Err(error) => {
                            println!("compile program ERROR: {:?}", error);
                            return;
                        }
                    };

                    let mut landscape = Vec::new();
                    landscape.extend(&[Vertex{v_pos: [-1.0,-1.0,0.0], v_col: 255},
                                       Vertex{v_pos: [-1.0,1.0,0.0], v_col: 255},
                                       Vertex{v_pos: [1.0,1.0,0.0], v_col: 255},
                                       Vertex{v_pos: [-1.0,-1.0,0.0], v_col: 255},
                                       Vertex{v_pos: [1.0,1.0,0.0], v_col: 255},
                                       Vertex{v_pos: [1.0,-1.0,0.0], v_col: 255},
                                        ]);
                    let mut grids_count = 0;

                    let mut camera_x = 1.0;
                    let mut camera_y = 1.0;
                    let mut camera_z = 1.0;

                    let mut dragging = false;
                    let mut dragging_xy = None;
                    let mut zooming = false;
                    let mut zooming_xy = None;


                    /*'ecto_loop:*/ loop {
                        {
                            let mut target = display.draw();
                            target.clear_color(0.1, 0.1, 0.1, 1.0);
                            let mut draw_params: DrawParameters = Default::default();
                            draw_params.polygon_mode = PolygonMode::Line;

                            let view: cgmath::AffineMatrix3<f32> = cgmath::Transform::look_at(
                                &cgmath::Point3::new(camera_x, camera_y, camera_z),
                                &cgmath::Point3::new(0.0, 0.0, 0.0),
                                &cgmath::Vector3::unit_z(),
                            );
                            /*
                            let view: cgmath::AffineMatrix3<f32> = cgmath::Transform::look_at(
                                &cgmath::Point3::new(0.275, 0.275, 1.2),
                                &cgmath::Point3::new(0.275, 0.275, 0.0),
                                &cgmath::Vector3::unit_y(),
                            );
                            */
                            let model_scale = 0.005;
                            let uniforms = uniform! {
                                //u_model: cgmath::Matrix4::identity().into_fixed(),
                                u_model: cgmath::Matrix4::new(model_scale, 0.0, 0.0, 0.0,
                                                              0.0, model_scale, 0.0, 0.0,
                                                              0.0, 0.0, model_scale, 0.0,
                                                              0.0, 0.0, 0.0, 1.0).into_fixed(),
                                u_view: view.mat.into_fixed(),
                                u_proj: cgmath::perspective(cgmath::deg(80.0f32), 1.0/*stream.get_aspect_ratio()*/, 0.1, 1000.0).into_fixed(),
                            };

                            if let Err(e) = target.draw(&vertex_buffer, &indices, &program, &uniforms/*EmptyUniforms*/, &draw_params) {
                                println!("target.draw ERROR: {:?}", e);
                                return;
                            }
                            if let Err(e) = target.finish() {
                                println!("target.finish ERROR: {:?}", e);
                                return;
                            }
                        }

                        for ev in display.poll_events() {
                            match ev {
                                glutin::Event::KeyboardInput(_, _, Some(glutin::VirtualKeyCode::Escape)) |
                                glutin::Event::Closed => {
                                    /*break 'ecto_loop;*/
                                    return;
                                }
                                glutin::Event::MouseInput(glutin::ElementState::Pressed, glutin::MouseButton::Left) => {
                                    dragging = true;
                                }
                                glutin::Event::MouseInput(glutin::ElementState::Released, glutin::MouseButton::Left) => {
                                    dragging = false;
                                    dragging_xy = None;
                                }
                                glutin::Event::MouseInput(glutin::ElementState::Pressed, glutin::MouseButton::Right) => {
                                    zooming = true;
                                }
                                glutin::Event::MouseInput(glutin::ElementState::Released, glutin::MouseButton::Right) => {
                                    zooming = false;
                                    zooming_xy = None;
                                }
                                glutin::Event::MouseMoved((x,y)) => {
                                    if dragging {
                                        dragging_xy = match dragging_xy {
                                            None => Some((x,y)),
                                            Some((mx,my)) => {
                                                camera_x += ((x - mx) as f32) / 1000.0;
                                                camera_z += ((y - my) as f32) / 1000.0;
                                                Some((x,y))
                                            }
                                        }
                                    }
                                    if zooming {
                                        zooming_xy = match zooming_xy {
                                            None => Some((x,y)),
                                            Some((mx,my)) => {
                                                let dy = y - my;
                                                let factor = 1.0 + (dy as f32) / 100.0;
                                                camera_x *= factor;
                                                camera_y *= factor;
                                                camera_z *= factor;
                                                Some((x,y))
                                            }
                                        }
                                    }
                                }
                                _ => ()
                            }
                        }

                        match rx.try_recv() {
                            Ok(value) => {
                                match value {
                                    Event::Grid(gridx,gridy,tiles,z) => {
                                        //let gridx = -gridx;
                                        //let gridy = -gridy;
                                        println!("render: received Grid ({},{})", gridx, gridy);
                                        let minz = {
                                            let mut minz = z[0];
                                            for i in 1 .. 10_000 {
                                                if z[i] < minz {
                                                    minz = z[i];
                                                }
                                            }
                                            minz
                                        };
                                        let mut vertices = Vec::with_capacity(10_000);
                                        for y in 0..100 {
                                            for x in 0..100 {
                                                let index = y*100+x;
                                                let vx = (gridx as f32) * 100.0 + (x as f32);
                                                let vy = (gridy as f32) * 100.0 + (y as f32);
                                                let vz = (z[index] - minz) as f32;
                                                vertices.push([vx,vy,vz]);
                                            }
                                        }
                                        let mut shape = Vec::with_capacity(60_000);
                                        for y in 0..99 {
                                            for x in 0..99 {
                                                let index = y*100+x;
                                                let color = tiles[index];
                                                shape.push(Vertex{v_pos: vertices[index+100], v_col: color});
                                                shape.push(Vertex{v_pos: vertices[index], v_col: color});
                                                shape.push(Vertex{v_pos: vertices[index+1], v_col: color});

                                                shape.push(Vertex{v_pos: vertices[index+100], v_col: color});
                                                shape.push(Vertex{v_pos: vertices[index+1], v_col: color});
                                                shape.push(Vertex{v_pos: vertices[index+101], v_col: color});
                                            }
                                        }
                                        landscape.extend(&shape);
                                        vertex_buffer = VertexBuffer::new(&display, &landscape).unwrap();
                                        grids_count += 1;
                                    }
                                }
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

        pub fn update (&mut self, event: Event) -> Result<(), SendError<Event>> {
            self.tx.send(event)
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
                /*TODO:
                Event::RenderQuit => {
                    self.state.close();
                }
                */
            }

            self.ai.update(&mut self.state);

            if let Some(xy) = self.state.start_point() {
                let (start_grid_x,start_grid_y) = state::grid(xy);
                while let Some(event) = self.state.next_event() {
                    if let Err(e) = self.render.update(
                        match event {
                            state::Event::Grid(x,y,tiles,z) => render::Event::Grid(x - start_grid_x, y - start_grid_y, tiles, z)
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

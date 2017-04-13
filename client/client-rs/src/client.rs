use errors::*;

pub fn authorize(host: &str, port: u16, user: String, pass: String) -> Result<(String, Vec<u8>)> {

    use std::net;
    use std::str;
    use openssl::hash::{hash, MessageDigest};
    use openssl::ssl;
    use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
    use std::io::Cursor;
    use std::io::Read;
    use std::io::Write;

    #[allow(non_camel_case_types)]
    type be = BigEndian;

    info!("authorize {} @ {}:{}", user, host, port);
    let stream = net::TcpStream::connect((host, port)).expect("tcpstream::connect");
    let mut ctx = ssl::SslContext::builder(ssl::SslMethod::tls()).expect("sslContext::builder");
    ctx.set_verify(ssl::SSL_VERIFY_NONE);
    let ctx = ctx.build();
    let ssl = ssl::Ssl::new(&ctx).expect("Ssl::new");
    let mut stream = ssl.connect(stream).expect("Ssl::connect");

    fn msg(buf: Vec<u8>) -> Vec<u8> {
        let mut msg = Vec::new();
        msg.write_u16::<be>(buf.len() as u16).expect("authorize.msg.write(buf.len)");
        msg.extend(buf);
        msg
    }

    // TODO use closure instead (no need to pass stream)
    fn command<S:Read+Write>(mut stream: S, cmd: Vec<u8>) -> Result<Vec<u8>> {
        stream.write(msg(cmd).as_slice()).chain_err(||"unable to write cmd")?;
        stream.flush().chain_err(||"unable to flush")?;

        let len = {
            let mut buf = vec![0; 2];
            stream.read_exact(&mut buf).chain_err(||"unable to read")?;
            let mut rdr = Cursor::new(buf);
            rdr.read_u16::<be>().chain_err(||"unable to read len")?
        };

        let mut msg = vec![0; len as usize];
        stream.read_exact(msg.as_mut_slice()).chain_err(||"unable to read msg")?;
        debug!("msg: {:?}", msg);
        if (msg.len() < "ok\0".len()) || (msg[0] != b'o') || (msg[1] != b'k') || (msg[2] != 0) {
            Err(format!("unexpected answer: '{}'",
                String::from_utf8(msg).chain_err(||"unable to decode msg")?).into())
        } else {
            Ok(msg[3..].to_vec())
        }
    }

    let login = {
        let mut buf = Vec::new();
        buf.extend("pw".as_bytes());
        buf.push(0);
        buf.extend(user.as_bytes());
        buf.push(0);
        buf.extend(hash(MessageDigest::sha256(), pass.as_bytes()).expect("hash").as_slice());
        let msg = command(&mut stream, buf).chain_err(||"unable to 'pw'")?;
        // FIXME use read_strz analog
        str::from_utf8(&msg[..msg.len() - 1]).chain_err(||"unable to decode login")?.to_string()
    };

    let cookie = {
        let mut buf = Vec::new();
        buf.extend("cookie".as_bytes());
        buf.push(0);
        command(&mut stream, buf)?
    };

    Ok((login, cookie))
}

use driver::Driver;
use ai::Ai;
use render::Render;
use state::State;

pub struct Client<'a, D: Driver + 'a, A: Ai + 'a> {
    render: Render,
    state: State,
    ai: &'a mut A,
    driver: &'a mut D,
}

impl<'a, D: Driver, A: Ai> Client<'a, D, A> {
    pub fn new(driver: &'a mut D, ai: &'a mut A) -> Client<'a, D, A> {
        Client {
            render: Render::new(), // TODO Render trait
            state: State::new(),
            ai: ai,
            driver: driver,
        }
    }

    fn send_all_enqueued(&mut self) {
        // TODO use iterator
        while let Some(ebuf) = self.state.tx() {
            self.driver.tx(&ebuf.buf).expect("send_all_enqueued");
            if let Some(timeout) = ebuf.timeout {
                self.driver.timeout(timeout.seq, timeout.ms);
            }
        }
    }

    fn dispatch_single_event(&mut self) {
        use driver;
        use web;

        match self.driver.event().expect("dispatch_single_event.event") {
            driver::Event::Rx(buf) => {
                // info!("event::rx: {} bytes", buf.len());
                self.state.rx(&buf).expect("dispatch_single_event.rx");
            }
            driver::Event::Timeout(seq) => {
                // info!("event::timeout: {} seq", seq);
                self.state.timeout(seq);
            }
            driver::Event::Tcp((tx, buf)) => {
                let reply = web::responce(&buf, &self.state);
                tx.send(reply).expect("dispatch_single_event.send");
                // self.driver.reply(reply);
            }
            // TODO:
            // Event::RenderQuit => {
            // self.state.close();
            // }
            //
        }
    }

    pub fn run(&mut self, login: &str, cookie: &[u8]) -> Result<()> {
        use rustc_serialize::hex::ToHex;
        use state;
        use render;
        use util;

        info!("connect {} / {}", login, cookie.to_hex());
        self.state.connect(login, cookie)?;

        loop {
            while let Some(event) = self.state.next_event() {
                let event = match event {
                    state::Event::Grid(x, y) => {
                        match self.state.map.grids.get(&(x, y)) {
                            Some(ref grid) => {
                                util::grid_to_png(grid.x, grid.y, &grid.tiles, &grid.z);
                                render::Event::Grid(x, y, grid.tiles.clone(), grid.z.clone())
                            }
                            None => {
                                warn!("Event::Grig received, but no such grid!");
                                continue;
                            }
                        }
                    }
                    state::Event::Obj((x, y)) => render::Event::Obj(x, y),
                };
                self.render.update(event).chain_err(||"unable to undate render")?;
            }
            self.send_all_enqueued();
            self.dispatch_single_event();
            self.ai.update(&mut self.state);
        }

        // while let None = self.state.start_point() {
        //     self.send_all_enqueued();
        //     self.dispatch_single_event();
        //     self.ai.update(&mut self.state);
        // }
        //
        // let (start_x, start_y) = match self.state.start_point() {
        //     Some(xy) => xy,
        //     None => unreachable!() //panic!("this can't be")
        // };
        //
        // loop {
        //     while let Some(event) = self.state.next_event() {
        //         let event = match event {
        //             state::Event::Grid(x,y,tiles,z) => render::Event::Grid(x * 1100 - start_x, y * 1100 - start_y, tiles, z),
        //             state::Event::Obj((x,y))        => render::Event::Obj(x - start_x, y - start_y),
        //         };
        //         //info!("event: {:?}", event);
        //         if let Err(e) = self.render.update(event) {
        //             info!("render.update ERROR: {:?}", e);
        //             return None /*TODO Some(e)*/;
        //         }
        //     }
        //     self.send_all_enqueued();
        //     self.dispatch_single_event();
        //     self.ai.update(&mut self.state);
        // }
    }
}

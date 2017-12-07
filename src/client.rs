use driver::Driver;
use ai::Ai;
use render::Render;
use state::State;
use Result;
use failure::err_msg;

pub fn authorize(host: &str, port: u16, user: String, pass: String) -> Result<(String, Vec<u8>)> {

    use std::net;
    use std::str;
    use openssl::hash::{hash2, MessageDigest};
    use openssl::ssl;
    use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
    use std::io::Cursor;
    use std::io::Read;
    use std::io::Write;

    #[allow(non_camel_case_types)]
    type be = BigEndian;

    info!("authorize {} @ {}:{}", user, host, port);
    let stream = net::TcpStream::connect((host, port))?;
    let mut ctx = ssl::SslContext::builder(ssl::SslMethod::tls())?;
    ctx.set_verify(ssl::SSL_VERIFY_NONE);
    let ctx = ctx.build();
    let ssl = ssl::Ssl::new(&ctx)?;
    let mut stream = ssl.connect(stream)?;

    fn msg(buf: Vec<u8>) -> Result<Vec<u8>> {
        let mut msg = Vec::new();
        msg.write_u16::<be>(buf.len() as u16)?;
        msg.extend(buf);
        Ok(msg)
    }

    // TODO use closure instead (no need to pass stream)
    fn command<S:Read+Write>(mut stream: S, cmd: Vec<u8>) -> Result<Vec<u8>> {
        let cmd = msg(cmd)?;
        stream.write(&cmd)?;
        stream.flush()?;

        let len = {
            let mut buf = vec![0; 2];
            stream.read_exact(&mut buf)?;
            let mut rdr = Cursor::new(buf);
            rdr.read_u16::<be>()?
        };

        let mut msg = vec![0; len as usize];
        stream.read_exact(msg.as_mut_slice())?;
        debug!("msg: {:?}", msg);
        if msg.len() < b"ok\0".len() {
            return Err(format_err!("too short answer: {:?}", msg));
        }
        match &msg[..3] {
            b"ok\0" => Ok(msg[3..].to_vec()),
            b"no\0" => {
                let msg = String::from_utf8(msg[3..].to_vec())?;
                //TODO add errors::AuthError(msg)
                Err(err_msg(msg))
            }
            _ => {
                let msg = str::from_utf8(&msg)?;
                Err(format_err!("unexpected answer: '{}'", msg))
            }
        }
    }

    let login = {
        let mut buf = Vec::new();
        buf.extend(b"pw\0");
        buf.extend(user.as_bytes());
        buf.push(0);
        let hash = hash2(MessageDigest::sha256(), pass.as_bytes())?;
        buf.extend(&*hash);
        let msg = command(&mut stream, buf)?;
        // FIXME use read_strz analog
        str::from_utf8(&msg[..msg.len() - 1])?.to_string()
    };

    let cookie = {
        let mut buf = Vec::new();
        buf.extend(b"cookie");
        buf.push(0);
        command(&mut stream, buf)?
    };

    Ok((login, cookie))
}

pub struct Client<'a, D: Driver + 'a, A: Ai + 'a> {
    render: Render, // TODO Render trait
    state: State,
    ai: &'a mut A,
    driver: &'a mut D,
}

impl<'a, D: Driver, A: Ai> Client<'a, D, A> {
    pub fn new(driver: &'a mut D, ai: &'a mut A, render: Render) -> Client<'a, D, A> {
        Client {
            render: render, // TODO Render trait
            state: State::new(),
            ai: ai,
            driver: driver,
        }
    }

    fn send_all_enqueued(&mut self) -> Result<()> {
        // TODO use iterator
        while let Some(ebuf) = self.state.tx() {
            self.driver.tx(&ebuf.buf)?;
            if let Some(timeout) = ebuf.timeout {
                self.driver.timeout(timeout.seq, timeout.ms);
            }
        }
        Ok(())
    }

    fn dispatch_single_event(&mut self) -> Result<()> {
        use driver;
        use web;
        use proto::ObjXY;

        let event = self.driver.event()?;
        match event {
            driver::Event::Rx(buf) => {
                // info!("event::rx: {} bytes", buf.len());
                self.state.rx(&buf)?;
            }
            driver::Event::Timeout(seq) => {
                // info!("event::timeout: {} seq", seq);
                self.state.timeout(seq);
            }
            driver::Event::Tcp((tx, buf)) => {
                let reply = web::responce(&buf, &self.state);
                tx.send(reply)?;
                // self.driver.reply(reply);
            }
            #[cfg(feature = "salem")]
            driver::Event::Render(re) => {
                match re {
                    driver::RenderEvent::Up    => if let Some(ObjXY(x,y)) = self.state.hero_xy() { self.state.go(ObjXY(x,y+100))?; },
                    driver::RenderEvent::Down  => if let Some(ObjXY(x,y)) = self.state.hero_xy() { self.state.go(ObjXY(x,y-100))?; },
                    driver::RenderEvent::Left  => if let Some(ObjXY(x,y)) = self.state.hero_xy() { self.state.go(ObjXY(x-100,y))?; },
                    driver::RenderEvent::Right => if let Some(ObjXY(x,y)) = self.state.hero_xy() { self.state.go(ObjXY(x+100,y))?; },
                    driver::RenderEvent::Quit  => self.state.close()?,
                }
            }
            #[cfg(feature = "hafen")]
            driver::Event::Render(re) => {
                info!("event: {:?}", re);
                match re {
                    driver::RenderEvent::Up    => if let Some(ObjXY(x,y)) = self.state.hero_xy() { self.state.go(ObjXY(x,y+100.0))?; },
                    driver::RenderEvent::Down  => if let Some(ObjXY(x,y)) = self.state.hero_xy() { self.state.go(ObjXY(x,y-100.0))?; },
                    driver::RenderEvent::Left  => if let Some(ObjXY(x,y)) = self.state.hero_xy() { self.state.go(ObjXY(x-100.0,y))?; },
                    driver::RenderEvent::Right => if let Some(ObjXY(x,y)) = self.state.hero_xy() { self.state.go(ObjXY(x+100.0,y))?; },
                    driver::RenderEvent::Quit  => self.state.close()?,
                }
            }
        }
        Ok(())
    }

    pub fn run(&mut self, login: &str, cookie: &[u8]) -> Result<()> {
        use rustc_serialize::hex::ToHex;
        use state;
        use render;
        use util;
        use chrono;

        info!("connect {} / {}", login, cookie.to_hex());
        self.state.connect(login, cookie)?;

        let start_time = &chrono::Local::now().format("%Y-%m-%d %H-%M-%S").to_string();

        loop {
            while let Some(event) = self.state.next_event() {
                let event = match event {
                    state::Event::Grid((x, y)) => {
                        match self.state.map.grids.get(&(x, y)) {
                            Some(ref grid) => {
                                let name = if let Some(ref name) = self.state.hero.name { name } else { "none" };
                                util::grid_to_png(login, name, start_time, grid.x, grid.y, &grid.tiles, &grid.z)?;
                                render::Event::Grid(x, y, grid.tiles.clone(), grid.z.clone(), grid.ol.clone())
                            }
                            None => {
                                warn!("Event::Grig received, but no such grid!");
                                continue;
                            }
                        }
                    }
                    state::Event::Obj(id, xy, resid) => render::Event::Obj(id, xy, resid),
                    state::Event::ObjRemove(id) => render::Event::ObjRemove(id),
                    state::Event::Res(resid,name) => render::Event::Res(resid,name),
                    state::Event::Hero => match self.state.hero.obj {
                        Some(ref hero) => {
                            match hero.xy {
                                Some(xy) => render::Event::Hero(xy),
                                None => panic!("hero's xy is None")
                            }
                        }
                        None => panic!("received Event::Hero while hero.obj is None")
                    },
                    state::Event::Wdg(wdg) => render::Event::Wdg(wdg),
                    state::Event::Tiles(tiles) => render::Event::Tiles(tiles),
                };
                self.render.update(event)?;
            }
            self.send_all_enqueued()?;
            self.dispatch_single_event()?;
            self.ai.update(&mut self.state);
        }
    }
}

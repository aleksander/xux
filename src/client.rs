use driver::Driver;
use ai::Ai;
use state::{self, State};
use Result;
use failure::err_msg;
use std::sync::mpsc::Sender;

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

pub struct Client<'a, A: Ai + 'a> {
    state: State,
    ai: &'a mut A,
}

impl<'a, A: Ai> Client<'a, A> {
    pub fn new(driver: Driver, ai: &'a mut A, hl_que_tx: Sender<state::Event>) -> Client<'a, A> {
        Client {
            state: State::new(hl_que_tx, driver),
            ai: ai,
        }
    }

    pub fn run(&mut self, login: &str, cookie: &[u8]) -> Result<()> {
        info!("connect {} / {}", login, cookie.iter().fold(String::new(), |s,b|format!("{}{:02x}",s,b)));
        self.state.connect(login, cookie)?;
        self.state.login = login.into();
        self.state.run(self.ai)
    }
}

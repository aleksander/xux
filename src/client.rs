use crate::state::State;
use crate::Result;
use failure::{err_msg, format_err};
use std::net;
use std::str;
use openssl::hash::{hash, MessageDigest};
use openssl::ssl;
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::io::Cursor;
use std::io::Read;
use std::io::Write;
use log::{debug, info};

pub fn authorize(host: &str, port: u16, user: String, pass: String) -> Result<(String, Vec<u8>)> {
    #[allow(non_camel_case_types)]
    type be = BigEndian;

    info!("authorize {} @ {}:{}", user, host, port);
    let stream = net::TcpStream::connect((host, port))?;
    let mut ctx = ssl::SslContext::builder(ssl::SslMethod::tls())?;
    ctx.set_verify(ssl::SslVerifyMode::NONE);
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
        let hash = hash(MessageDigest::sha256(), pass.as_bytes())?;
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

pub fn run(host: &str, port: u16, login: &str, cookie: &[u8]) -> Result<()> {
    use std::sync::mpsc::channel;
    use crate::driver;
    use crate::render;
    use crate::ai;

    let driver = driver::new(host, port)?;

    let (hl_que_tx_render, hl_que_rx) = channel();
    render::new(driver.sender(), hl_que_rx);

    let (hl_que_tx_ai, hl_que_rx) = channel();
    ai::new(driver.sender(), hl_que_rx);

    let mut state = State::new(hl_que_tx_render, hl_que_tx_ai, driver);
    state.run(login, cookie)
}

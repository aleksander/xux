extern crate openssl;
extern crate serialize;

use std::io::Writer;
//use std::io::IoError;
use std::io::net::tcp::TcpStream;
//use std::str;
//use std::string;

use openssl::crypto::hash::{SHA256, hash};
use openssl::ssl::{Sslv23, SslContext, SslStream/*, SslVerifyPeer*/};
//use openssl::x509::{X509Generator, X509, DigitalSignature, KeyEncipherment, ClientAuth, ServerAuth, X509StoreContext};
use serialize::hex::ToHex;

struct MyError {
    source: &'static str,
    detail: Option<String>,
}

fn authorize (host: &str, port: u16, user: &str, pass: &str) -> Result<Vec<u8>, MyError> {
    //let stream = TcpStream::connect(host, port).unwrap();
    let stream = match TcpStream::connect(host, port) {
        Ok(e)=>e,
        Err(e)=>return Err(MyError{source:"connect", detail:e.detail})
    };
    let mut stream = SslStream::new(&SslContext::new(Sslv23).unwrap(), stream).unwrap();

    // send 'pw' command
    stream.write_be_u16((3+user.as_bytes().len()+1+32) as u16).unwrap();
    stream.write("pw".as_bytes()).unwrap();
    stream.write_u8(0u8).unwrap();
    stream.write(user.as_bytes()).unwrap();
    stream.write_u8(0u8).unwrap();
    let pass_hash = hash(SHA256, pass.as_bytes());
    assert!(pass_hash.len() == 32u);
    stream.write(pass_hash.as_slice()).unwrap();
    stream.flush().unwrap();
//    stream.write(" there".as_bytes()).unwrap();
//    stream.flush().unwrap();
//    stream.write("GET /\r\n\r\n".as_bytes()).unwrap();
//    stream.flush().unwrap();
//    let buf = stream.read_to_end().ok().expect("read error");
//    print!("{}", str::from_utf8(buf.as_slice()));
    let length = stream.read_be_u16().ok().expect("read error");
    let msg = stream.read_exact(length as uint).ok().expect("read error");
    //println!("msg='{}'", str::from_utf8(msg.as_slice()).unwrap());
    println!("msg='{}'", msg.as_slice().to_hex());

    // send 'cookie' command
    if (msg[0] == ('o' as u8)) && (msg[1] == ('k' as u8)) {
        stream.write_be_u16(("cookie".as_bytes().len()+1) as u16).unwrap();
        stream.write("cookie".as_bytes()).unwrap();
        stream.write_u8(0u8).unwrap();
        stream.flush().unwrap();
        let length = stream.read_be_u16().ok().expect("read error");
        let msg = stream.read_exact(length as uint).ok().expect("read error");
        //println!("msg='{}'", str::from_utf8(msg.as_slice()).unwrap());
        println!("msg='{}'", msg.as_slice().to_hex());
        return Ok(msg.slice_from(3).to_vec());
    }
    return Err(MyError{
        source:"unexpected server answer",
        detail:Some(String::from_utf8(msg).unwrap())
                     });
}

fn main() {
    //let host = "148.251.44.214";
    let host = "game.salemthegame.com";
    let port: u16 = 1871;
    let user = "salvian";
    let pass = "простойпароль";
    println!("authorize at {}:{}", host, port);
    match authorize(host, port, user, pass) {
        Ok(cookie) => println!("success. cookie = [{}]", cookie.as_slice().to_hex()),
        Err(e) => println!("error. {}: {}", e.source, e.detail)
    }
}

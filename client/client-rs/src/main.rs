extern crate openssl;
extern crate serialize;

use std::io::Writer;
//use std::io::IoError;
use std::io::net::tcp::TcpStream;
use std::str;
use std::from_str::FromStr;
//use std::string;
use std::io::net::udp::UdpSocket;
//use std::io::net::udp::UdpStream;
//use std::io::net::ip::Ipv4Addr;
use std::io::net::ip::SocketAddr;


use openssl::crypto::hash::{SHA256, hash};
use openssl::ssl::{Sslv23, SslContext, SslStream};
use serialize::hex::ToHex;

//#![feature(macro_rules)]
//macro_rules! tryio (
//   ($fmt:expr $e:expr) => (
//       (match $e { Ok(e) => e, Err(e) => return Err(MyError{source:$fmt, detail:e.detail}) })
//   )
//)

struct MyError {
    source: &'static str,
    detail: Option<String>,
}

struct SalemClient;

impl SalemClient {

    //fn new () -> SalemClient { SalemClient }

    fn authorize (self, host: &str, port: u16, user: &str, pass: &str) -> Result<Vec<u8>, MyError> {
        //let stream = TcpStream::connect(host, port).unwrap();
        let stream = match TcpStream::connect(host, port) {
            Ok(e)=>e,
            Err(e)=>return Err(MyError{source:"connect", detail:e.detail})
        };
        //let stream = tryio!(TcpStream::connect(host, port));

        let mut stream = SslStream::new(&SslContext::new(Sslv23).unwrap(), stream).unwrap();

        // send 'pw' command
        // TODO form buffer and send all with one call
        stream.write_be_u16((3+user.as_bytes().len()+1+32) as u16).unwrap();
        stream.write("pw".as_bytes()).unwrap();
        stream.write_u8(0u8).unwrap();
        stream.write(user.as_bytes()).unwrap();
        stream.write_u8(0u8).unwrap();
        let pass_hash = hash(SHA256, pass.as_bytes());
        assert!(pass_hash.len() == 32u);
        stream.write(pass_hash.as_slice()).unwrap();
        stream.flush().unwrap();
        let length = stream.read_be_u16().ok().expect("read error");
        let msg = stream.read_exact(length as uint).ok().expect("read error");
        println!("msg='{}'", str::from_utf8(msg.as_slice()).unwrap());
        //println!("msg='{}'", msg.as_slice().to_hex());
        if msg.len() < "ok\0\0".len() {
            return Err(MyError{source:"unexpected server answer", detail:Some(String::from_utf8(msg).unwrap())});
        }

        // send 'cookie' command
        if (msg[0] == ('o' as u8)) && (msg[1] == ('k' as u8)) {
            // TODO form buffer and send all with one call
            stream.write_be_u16(("cookie".as_bytes().len()+1) as u16).unwrap();
            stream.write("cookie".as_bytes()).unwrap();
            stream.write_u8(0u8).unwrap();
            stream.flush().unwrap();
            let length = stream.read_be_u16().ok().expect("read error");
            let msg = stream.read_exact(length as uint).ok().expect("read error");
            //println!("msg='{}'", str::from_utf8(msg.as_slice()).unwrap());
            println!("msg='{}'", msg.as_slice().to_hex());
            //TODO check cookie length
            return Ok(msg.slice_from(3).to_vec());
        }
        return Err(MyError{source:"unexpected server answer", detail:Some(String::from_utf8(msg).unwrap())});
    }

    fn connect (self, cookie: &[u8]) {
        use std::io::MemWriter;

        //let addr = SocketAddr::from_str("game.salemthegame.com:1870");
        let addr: SocketAddr = FromStr::from_str("0.0.0.0:0").unwrap();
        let sock = UdpSocket::bind(addr).unwrap();
        let addr: SocketAddr = FromStr::from_str("148.251.44.214:1870").unwrap();
        let mut stream = sock.connect(addr);

        //unknown=2 proto=Salem ver=33 user=soos cookie=[ .//J..%.....R...G......Q x![..e.b]
        //let mut buf = Vec::new();
        let mut w = MemWriter::new();
        w.write_u8(0u8).unwrap();
        w.write_le_u16(2u16).unwrap();
        w.write("Salem".as_bytes()).unwrap();
        w.write_u8(0u8).unwrap();
        w.write_le_u16(34u16).unwrap();
        w.write("salvian".as_bytes()).unwrap();
        w.write_u8(0u8).unwrap();
        w.write_le_u16(32u16).unwrap();
        w.write(cookie).unwrap();
        stream.write(w.unwrap().as_slice()).unwrap();
        stream.flush().unwrap();

        //let error = stream.read_u8().unwrap();
        //println!("session error = {}", error);
        let mut buf = [0u8, ..65535];
        let len = stream.read(buf.as_mut_slice()).unwrap();
        println!("result = {}", buf.slice_to(len).to_hex());
    }

}



/* CONCEPT:
     client.connect()
        start receiver thread
        start transmitter thread
        add task.sess
            while not acked { send sess }
            if sess err != ok => fail
            else {set connected, add task.beat(every 5 sec)}
        add task.wait_for_login_screen_ui
        add task.wdg_msg(0, "focus", 1)
     client.choice("Lemming")
     client...

     client.receiver
        save and ack all rel
*/



fn main() {
    use std::io::net::addrinfo::get_host_addresses;
    //use std::sync::Arc;

    let host = "game.salemthegame.com";
    let addrs = get_host_addresses(host).unwrap();
    println!("host ip: {}", addrs);
    //TODO get first ipv4 addr as host addr
    let port: u16 = 1871;
    let user = "salvian";
    let pass = "простойпароль";
    let client = SalemClient;

    println!("authorize at {}:{}", host, port);
    let cookie = match client.authorize(host, port, user, pass) {
        Ok(cookie) => cookie,
        Err(e) => { println!("error. {}: {}", e.source, e.detail.unwrap()); return; }
    };
    println!("success. cookie = [{}]", cookie.as_slice().to_hex());





    let addr: SocketAddr = FromStr::from_str("127.0.0.1:65500").unwrap();
    let mut _sock = UdpSocket::bind(addr).unwrap();

    // UDP sender
    let mut sock = _sock.clone();
    spawn(proc() {
        sock.send_to("1. Hello, World!!!".as_bytes(), addr).unwrap();
        sock.send_to("2. Hello, World!!!".as_bytes(), addr).unwrap();
        sock.send_to("3. Hello, World!!!".as_bytes(), addr).unwrap();
    });

    // UDP receiver
    let mut sock = _sock.clone();
    spawn(proc() {
        for _ in range(1u, 3) {
            let mut buf = [0u8, ..128];
            let (len,_) = sock.recv_from(buf.as_mut_slice()).unwrap();
            println!("seceive: {}", buf.slice_to(len).to_hex());
        }
    });




    client.connect(cookie.as_slice());
}





















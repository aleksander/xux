//#![feature(rustc_private)]
#![feature(convert)]
#![feature(ip_addr)]
#![feature(collections)]
#![feature(lookup_host)]

extern crate openssl;

extern crate rustc_serialize;
use rustc_serialize::hex::ToHex;

extern crate mio;
use mio::Handler;
//use mio::Socket;
use mio::Token;
//use mio::NonBlock;
use mio::EventLoop;
use mio::Interest;
use mio::PollOpt;
use mio::ReadHint;
use mio::TryRead;
use mio::TryWrite;
use mio::buf::Buf;
use mio::buf::ByteBuf;
//use mio::buf::MutBuf;
//use mio::buf::MutByteBuf;
use mio::buf::RingBuf;
use mio::buf::SliceBuf;
use mio::tcp::TcpListener;
use mio::tcp::TcpStream;
use mio::udp::UdpSocket;
//use mio::udp::bind;
use mio::util::Slab;

//#[macro_use]
//extern crate log;

//extern crate bytes;
//use bytes::Buf;

use std::str;
use std::io::{Error, ErrorKind};

mod salem;
use salem::client::*;

const UDP: Token = Token(0);
const TCP: Token = Token(1);

enum Url {
    Root,
    Objects,
    Widgets,
    Resources,
    Go(i32,i32),
    //Quit
}

struct ControlConn {
    sock: TcpStream,
    //buf: Option<ByteBuf>,
    //mut_buf: Option<MutByteBuf>,
    token: Option<Token>,
    //interest: Interest,
    url: Option<Url>,
}

impl ControlConn {
    fn new(sock: TcpStream) -> ControlConn {
        ControlConn {
            sock: sock,
            //buf: None,
            //mut_buf: Some(ByteBuf::mut_with_capacity(2048)),
            token: None,
            //interest: Interest::hup(),
            url: None,
        }
    }

    fn writable (&mut self, eloop: &mut EventLoop<AnyHandler>, client: &Client) -> std::io::Result<()> {
        println!("{:?}: writable", self.token);
        //let mut buf = self.buf.take().unwrap();

        //let mut buf = ByteBuf::mut_with_capacity(2048);
        //buf.write_slice(b"hello there!\n");

        /* TODO
           Ok(Control::Dump) => {
        */

        let buf = match self.url {
            Some(ref url) => {
                match url {
                    &Url::Root => { 
                        let body = "<html> \r\n\
                                        <head> \r\n\
                                            <title></title> \r\n\
                                            <script src=\"http://code.jquery.com/jquery-1.11.3.min.js\" stype=\"text/javascript\"></script> \r\n\
                                            <script type=\"text/javascript\"> \r\n\
                                                $(document).ready(function(){ \r\n\
                                                    $('#getdata-button').on('click', function(){ \r\n\
                                                        $.get('http://localhost:33000/data', function(data) { \r\n\
                                                            $('#showdata').html(\"<p>\"+data+\"</p>\"); \r\n\
                                                        }); \r\n\
                                                    }); \r\n\
                                                }); \r\n\
                                            </script> \r\n\
                                        </head> \r\n\
                                        <body> \r\n\
                                            <a href=\"#\" id=\"getdata-button\">C</a> \r\n\
                                            <div id=\"showdata\"></div> \r\n\
                                        </body> \r\n\
                                    </html>\r\n\r\n";
                        format!("HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\n\r\n", body.len()) + &body
                    }
                    &Url::Resources => {
                        //TODO
                        "HTTP/1.1 404 Not Implemented\r\n\r\n".to_string()
                    }
                    &Url::Objects => {
                        let mut body = String::new();
                        //buf.push_all(b"<p><b>Test text</b></p>\r\n");
                        for o in client.objects.values() {
                            //let (x,y) = o.xy;
                            //let resid = o.resid;
                            let resname = match client.resources.get(&o.resid) {
                                Some(res) => res.as_str(),
                                None      => "null"
                            };
                            //buf.write_slice(format!("({:7},{:7}) {:7} {}\n", x, y, resid, resname).as_bytes());
                            //body.push_all(format!("({:7},{:7}) {:7} {}\r\n", x, y, resid, resname).as_bytes());
                            body = body + &format!("{{\"x\":{},\"y\":{},\"resid\":{},\"resname\":\"{}\"}},", o.x, o.y, o.resid, resname);
                        }
                        //let mut buf = Vec::new();
                        body = "[ ".to_string() + &body[..body.len()-1] + " ]";
                        format!("HTTP/1.1 200 OK\r\nContent-Type: text/json\r\nContent-Length: {}\r\nAccess-Control-Allow-Origin: *\r\n\r\n", body.len()) + &body
                    }
                    &Url::Widgets => {
                        let mut body = String::new();
                        for (id,name) in &client.widgets {
                            body = body + &format!("{{\"id\":{},\"name\":\"{}\"}},", id, name);
                        }
                        body = "[ ".to_string() + &body[..body.len()-1] + " ]";
                        format!("HTTP/1.1 200 OK\r\nContent-Type: text/json\r\nContent-Length: {}\r\nAccess-Control-Allow-Origin: *\r\n\r\n", body.len()) + &body
                    }
                    &Url::Go(x,y) => {
                        println!("GO: {} {}", x, y);
                        //TODO
                        "HTTP/1.1 404 Not Implemented\r\n\r\n".to_string()
                    }
                    //else {
                    //    "HTTP/1.1 404 Not Found\r\n\r\n".to_string()
                    //}
                }
            }
            None => {
                return Err(Error::new(ErrorKind::Other, "writable with empty URL"));
            }
        };


        //match self.sock.write(&mut buf.flip()) {
        match self.sock.write(&mut ByteBuf::from_slice(buf.as_bytes())) {
            Ok(None) => {
                println!("client flushing buf; WOULDBLOCK");
                //self.buf = Some(buf);
                //self.interest.insert(Interest::writable());
                if let Err(e) = eloop.reregister(&self.sock, self.token.unwrap(), Interest::writable(), PollOpt::edge() | PollOpt::oneshot()) {
                    println!("ERROR: failed to re-reg for write: {}", e);
                }
            }
            Ok(Some(r)) => {
                println!("CONN: we wrote {} bytes!", r);
                //self.mut_buf = Some(buf.flip());
                //self.interest.insert(Interest::readable());
                //self.interest.remove(Interest::writable());
                if let Err(e) = eloop.reregister(&self.sock, self.token.unwrap(), Interest::readable(), PollOpt::edge() | PollOpt::oneshot()) {
                    println!("ERROR: failed to re-reg for read: {}", e);
                }
            }
            Err(e) => println!("not implemented; client err={:?}", e),
        }
        //eloop.reregister(&self.sock, self.token.unwrap(), self.interest, PollOpt::edge() | PollOpt::oneshot())
        Ok(())
    }

    fn readable (&mut self, eloop: &mut EventLoop<AnyHandler>, client: &mut Client) -> std::io::Result<()> {
        println!("{:?}: readable", self.token);
        //let mut buf = self.mut_buf.take().expect("mut_buf.take");
        let mut buf = ByteBuf::mut_with_capacity(2048);
        match self.sock.read(&mut buf) {
            Ok(None) => {
                println!("We just got readable, but were unable to read from the socket?");
                eloop.shutdown();
            }
            Ok(Some(0)) => {
                println!("read zero bytes. de-reg this conn");
                if let Err(e) = eloop.deregister(&self.sock) {
                    println!("deregister error: {}", e);
                }
                return Err(Error::new(ErrorKind::Other, "read zero bytes"));
            }
            Ok(Some(r)) => {
                println!("{:?}: read {} bytes", self.token, r);
                let buf = buf.flip();
                let buf = String::from_utf8_lossy(buf.bytes());
                //println!("CONN read: {}", buf);
                if buf.starts_with("GET / ") {
                    self.url = Some(Url::Root);
                } else if buf.starts_with("GET /objects ") {
                    self.url = Some(Url::Objects);
                } else if buf.starts_with("GET /widgets ") {
                    self.url = Some(Url::Widgets);
                } else if buf.starts_with("GET /resources ") {
                    self.url = Some(Url::Resources);
                } else if buf.starts_with("GET /go/") {
                    let tmp1: Vec<&str> = buf.split(' ').collect();
                    println!("TMP1: {:?}", tmp1);
                    let tmp2: Vec<&str> = tmp1[1].split('/').collect();
                    println!("TMP2: {:?}", tmp2);
                    if tmp2.len() > 3 {
                        let x: i32 = match str::FromStr::from_str(tmp2[2]) { Ok(v) => v, Err(_) => 0 };
                        let y: i32 = match str::FromStr::from_str(tmp2[3]) { Ok(v) => v, Err(_) => 0 };
                        self.url = Some(Url::Go(x,y));
                    } else {
                        self.url = Some(Url::Go(0,0));
                    }
                } else if buf.starts_with("GET /quit ") {
                    //self.url = Some(Url::Quit);
                    if let Err(e) = client.shutdown() {
                        println!("ERROR: client.shutdown: {:?}", e);
                    }
                } else {
                    //TODO pass buf to Lua interpreter
                }
                //self.interest.remove(Interest::readable());
                //self.interest.insert(Interest::writable());
                eloop.reregister(&self.sock, self.token.unwrap(), Interest::writable(), PollOpt::edge()).unwrap();
            }
            Err(e) => {
                println!("not implemented; client err={:?}", e);
                //self.interest.remove(Interest::readable());
                eloop.shutdown();
            }

        };
        // prepare to provide this to writable
        //FIXME self.buf = Some(buf);
        //FIXME eloop.reregister(&self.sock, self.token.unwrap(), self.interest, PollOpt::edge())
        Ok(())
    }
}

struct AnyHandler<'a> {
    sock: UdpSocket,
    addr: std::net::SocketAddr,
    client: &'a mut Client,
    counter: usize,
    tcp_listener: TcpListener,
    conns: Slab<ControlConn>,
}

impl<'a> AnyHandler<'a> {
    fn new(sock: UdpSocket, tcp_listener: TcpListener, client: &'a mut Client, addr: std::net::SocketAddr) -> AnyHandler<'a> {
        AnyHandler {
            sock: sock,
            addr: addr,
            client: client,
            counter: 0,
            tcp_listener: tcp_listener,
            conns: Slab::new_starting_at(Token(2), 128),
        }
    }

    fn accept (&mut self, eloop: &mut EventLoop<AnyHandler>) -> std::io::Result<()> {
        println!("TCP: new connection");
        let tcp_stream = self.tcp_listener.accept().unwrap().unwrap();
        let conn = ControlConn::new(tcp_stream);
        let tok = self.conns.insert(conn).ok().expect("could not add connection to slab");
        self.conns[tok].token = Some(tok);
        eloop.register_opt(&self.conns[tok].sock, tok, Interest::readable(), PollOpt::edge() | PollOpt::oneshot()).ok().expect("could not reg IO for new conn");
        Ok(())
    }
    
    fn conn_readable (&mut self, eloop: &mut EventLoop<AnyHandler>, tok: Token) -> std::io::Result<()> {
        println!("conn readable; tok={:?}", tok);
        //if let Err(e) = self.conn(tok).readable(eloop) {
        if let Err(_) = self.conns[tok].readable(eloop, self.client) {
            self.conns.remove(tok);
        }
        Ok(())
    }

    fn conn_writable (&mut self, eloop: &mut EventLoop<AnyHandler>, tok: Token) -> std::io::Result<()> {
        println!("conn writable; tok={:?}", tok);
        //self.conn(tok).writable(eloop)
        self.conns[tok].writable(eloop, self.client)
    }

    /*
    fn conn<'b> (&'b mut self, tok: Token) -> &'b mut ControlConn {
        &mut self.conns[tok]
    }
    */
}

impl<'a> Handler for AnyHandler<'a> {
    type Timeout = usize;
    type Message = ();

    fn readable(&mut self, eloop: &mut EventLoop<AnyHandler>, token: Token, _: ReadHint) {
        match token {
            UDP => {
                let mut rx_buf = RingBuf::new(65535);
                self.sock.recv_from(&mut rx_buf).ok().expect("sock.recv");
                {
                    let mut client: &mut Client = self.client;
                    let buf: &[u8] = Buf::bytes(&rx_buf);
                    if let Err(e) = client.rx(buf) {
                        println!("error: {:?}", e);
                        eloop.shutdown();
                    }
                }
            },
            TCP => {
                self.accept(eloop).ok().expect("TCP.accept");
            }
            i => {
                self.conn_readable(eloop, i).unwrap();
            }
        }
    }

    fn writable(&mut self, eloop: &mut EventLoop<AnyHandler>, token: Token) {
        match token {
            UDP => {
                match self.client.tx() {
                    Some(ebuf) => {
                        /*
                        if let Ok((msg,_)) = Message::from_buf(ebuf.buf.as_slice(), MessageDirection::FromClient) {
                            println!("TX: {:?}", msg);
                        } //TODO else println(ERROR:malformed message); eloop_shutdown();
                        */
                        self.counter += 1;
                        //if self.counter % 3 == 0 {
                            let mut buf = SliceBuf::wrap(ebuf.buf.as_slice());
                            if let Err(e) = self.sock.send_to(&mut buf, &self.addr) {
                                println!("send_to error: {}", e);
                                eloop.shutdown();
                            }
                        //} else {
                        //    println!("DROPPED!");
                        //}
                        
                        if let Some(timeout) = ebuf.timeout {
                            //TODO use returned timeout handle to cancel timeout
                            println!("set {} timeout {} ms", timeout.seq, timeout.ms);
                            if let Err(e) = eloop.timeout_ms(timeout.seq, timeout.ms) {
                                println!("eloop.timeout FAILED: {:?}", e);
                                eloop.shutdown();
                            }
                        }
                    },
                    None => {}
                }
            }
            TCP => {
                println!("ERROR: writable on tcp listener");
                eloop.shutdown();
            }
            _ => {
                if let Err(e) = self.conn_writable(eloop, token) {
                    println!("ERROR: {:?} conn_writable: {}", token, e);
                }
            }
        }
    }

    fn timeout (&mut self, /*eloop*/ _: &mut EventLoop<AnyHandler>, timeout: usize) {
        self.client.timeout(timeout);
    }
}

fn main() {
    //TODO use PollOpt::edge() | PollOpt::oneshot() for UDP connection and not PollOpt::level() (see how this is doing for TCP conns)
    //TODO handle keyboard interrupt
    //TODO replace all unwraps with normal error handling
    //TODO ADD tests:
    //        for i in range(0u8, 255) {
    //            let mut v = Vec::new();
    //            v.push(i);
    //            println!("{}", Message::from_buf(v.as_slice()));
    //        }
    //TODO FIXME add username/password prompt, remove plain text username/password from sources


    let any = str::FromStr::from_str("0.0.0.0:0").ok().expect("any.from_str");
    let sock = UdpSocket::bound(&any).ok().expect("udp::bound");

    //FIXME sock.connect(&addr);
    //FIXME sock.set_reuseaddr(true).ok().expect("set_reuseaddr");

    //TODO return Result and match
    let mut client = Client::new();

    //TODO FIXME get login/password from command line instead of storing them here
    match client.authorize("salvian", "простойпароль", "game.salemthegame.com", 1871) {
        Ok(()) => { println!("success. cookie = [{}]", client.cookie.as_slice().to_hex()); },
        Err(e) => { println!("authorize error: {:?}", e); return; }
    };

    let mut eloop = EventLoop::new().ok().expect("eloop.new");
    eloop.register_opt(&sock, UDP, Interest::readable() | Interest::writable(), PollOpt::level()).ok().expect("eloop.register(udp)");

    let addr: std::net::SocketAddr = str::FromStr::from_str("127.0.0.1:33000").ok().expect("any.from_str");
    let tcp_listener = TcpListener::bind(&addr).unwrap();
    eloop.register_opt(&tcp_listener, TCP, Interest::readable(), PollOpt::edge()).unwrap();

    let ip = client.serv_ip;
    let mut handler = AnyHandler::new(sock, tcp_listener, &mut client, std::net::SocketAddr::new(ip, 1870));
    handler.client.connect().ok().expect("client.connect()");

    /*
    if let Err(e) = eloop.timeout_ms(123, 4000) {
        println!("eloop.timeout FAILED: {:?}", e);
        return;
    }
    */
    //FIXME move to reactor part
    /*
    if self.client.ready_to_go() {
        println!("client is ready to GO!");
        if let Err(e) = self.client.go() {
            println!("client.go FAILED: {:?}", e);
            eloop.shutdown();
        }
    }
    */

    println!("run event loop");
    eloop.run(&mut handler).ok().expect("Failed to run the event loop");
}

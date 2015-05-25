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
use mio::buf::MutBuf;
use mio::buf::MutByteBuf;
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

mod salem;
use salem::client::*;

const UDP: Token = Token(0);
const TCP: Token = Token(1);

struct ControlConn {
    sock: TcpStream,
    buf: Option<ByteBuf>,
    mut_buf: Option<MutByteBuf>,
    token: Option<Token>,
    interest: Interest,
}

impl ControlConn {
    fn new(sock: TcpStream) -> ControlConn {
        ControlConn {
            sock: sock,
            buf: None,
            mut_buf: Some(ByteBuf::mut_with_capacity(2048)),
            token: None,
            interest: Interest::hup()
        }
    }

    fn writable (&mut self, event_loop: &mut EventLoop<AnyHandler>) -> std::io::Result<()> {
        let mut buf = self.buf.take().unwrap();
        match self.sock.write(&mut buf) {
            Ok(None) => {
                println!("client flushing buf; WOULDBLOCK");
                self.buf = Some(buf);
                self.interest.insert(Interest::writable());
            }
            Ok(Some(r)) => {
                println!("CONN : we wrote {} bytes!", r);
                self.mut_buf = Some(buf.flip());
                self.interest.insert(Interest::readable());
                self.interest.remove(Interest::writable());
            }
            Err(e) => println!("not implemented; client err={:?}", e),
        }
        event_loop.reregister(&self.sock, self.token.unwrap(), self.interest, PollOpt::edge() | PollOpt::oneshot())
    }

    fn readable (&mut self, eloop: &mut EventLoop<AnyHandler>) -> std::io::Result<()> {
        let mut buf = self.mut_buf.take().unwrap();
        match self.sock.read(&mut buf) {
            Ok(None) => {
                println!("We just got readable, but were unable to read from the socket?");
                eloop.shutdown();
            }
            Ok(Some(r)) => {
                println!("CONN: we read {} bytes: {:?}", r, buf.mut_bytes());
                self.interest.remove(Interest::readable());
                self.interest.insert(Interest::writable());
            }
            Err(e) => {
                println!("not implemented; client err={:?}", e);
                self.interest.remove(Interest::readable());
            }

        };
        // prepare to provide this to writable
        self.buf = Some(buf.flip());
        eloop.reregister(&self.sock, self.token.unwrap(), self.interest, PollOpt::edge())
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
        eloop.register_opt(&self.conns[tok].sock, tok, Interest::readable(), PollOpt::edge() | PollOpt::oneshot()).ok().expect("could not register socket with event loop");
        Ok(())
    }
    
    fn conn_readable (&mut self, eloop: &mut EventLoop<AnyHandler>, tok: Token) -> std::io::Result<()> {
        println!("conn readable; tok={:?}", tok);
        self.conn(tok).readable(eloop)
    }

    fn conn_writable (&mut self, eloop: &mut EventLoop<AnyHandler>, tok: Token) -> std::io::Result<()> {
        println!("conn writable; tok={:?}", tok);
        self.conn(tok).writable(eloop)
    }

    fn conn<'b> (&'b mut self, tok: Token) -> &'b mut ControlConn {
        &mut self.conns[tok]
    }
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
            },
            _ => ()
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

    /* TODO
    Ok(Control::Dump) => {
        for o in objects.values() {
            let (x,y) = o.xy;
            let resid = o.resid;
            let resname = match resources.get(&o.resid) {
                Some(res) => { res.as_slice() },
                None      => { "null" },
            };
            client.control_tx.send(format!("({:7},{:7}) {:7} {}", x, y, resid, resname));
        }
    },
    */

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
    eloop.register_opt(&tcp_listener, TCP, Interest::readable(), PollOpt::edge() | PollOpt::oneshot()).unwrap();

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

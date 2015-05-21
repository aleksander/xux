#![feature(rustc_private)]
#![feature(convert)]
#![feature(ip_addr)]
#![feature(collections)]
#![feature(lookup_host)]

extern crate openssl;
extern crate rustc_serialize;
extern crate mio;

#[macro_use]
extern crate log;

use std::net::UdpSocket;
use std::net::SocketAddr;
use std::str;
use rustc_serialize::hex::ToHex;

mod salem;
//use salem::message::*;
use salem::client::*;

//TODO
/*
enum MsgType {
    REL,
    C_SESS,
    MAPREQ,
}
*/

fn main() {
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

    use mio::Socket;

    const UDP: mio::Token = mio::Token(0);
    const TCP: mio::Token = mio::Token(1);

    struct ControlConn {
        sock: mio::tcp::TcpStream,
        //buf: Option<mio::buf::ByteBuf>,
        //mut_buf: Option<mio::buf::MutByteBuf>,
        token: Option<mio::Token>,
        //interest: mio::Interest
    }

    impl ControlConn {
        fn new(sock: mio::tcp::TcpStream) -> ControlConn {
            ControlConn {
                sock: sock,
                //buf: None,
                //mut_buf: Some(mio::buf::ByteBuf::mut_with_capacity(2048)),
                token: None,
                //interest: mio::Interest::hup()
            }
        }
    }

    struct UdpHandler<'a> {
        sock: mio::NonBlock<mio::udp::UdpSocket>,
        addr: std::net::SocketAddr,
        client: &'a mut Client,
        counter: usize,
        tcp_listener: mio::tcp::TcpListener,
        conns: mio::util::Slab<ControlConn>,
    }

    impl<'a> UdpHandler<'a> {
        fn new(sock: mio::NonBlock<mio::udp::UdpSocket>, tcp_listener: mio::tcp::TcpListener, client:&'a mut Client, addr: std::net::SocketAddr) -> UdpHandler<'a> {
            UdpHandler {
                sock: sock,
                addr: addr,
                client: client,
                counter: 0,
                tcp_listener: tcp_listener,
                conns: mio::util::Slab::new_starting_at(mio::Token(2), 128),
            }
        }

        fn accept (&mut self, event_loop: &mut mio::EventLoop<UdpHandler>) -> std::io::Result<()> {
            println!("TCP: new connection");
            let (sock,_) = self.tcp_listener.accept().unwrap();
            let conn = ControlConn::new(sock);
            let tok = self.conns.insert(conn).ok().expect("could not add connection to slab");
            self.conns[tok].token = Some(tok);
            event_loop.register_opt(&self.conns[tok].sock, tok, mio::Interest::readable(), mio::PollOpt::edge() | mio::PollOpt::oneshot()).ok().expect("could not register socket with event loop");
            Ok(())
        }
    }

    impl<'a> mio::Handler for UdpHandler<'a> {
        type Timeout = usize;
        type Message = ();

        fn readable(&mut self, eloop: &mut mio::EventLoop<UdpHandler>, token: mio::Token, _: mio::ReadHint) {
            match token {
                UDP => {
                    let mut rx_buf = mio::buf::RingBuf::new(65535);
                    self.sock.recv_from(&mut rx_buf).ok().expect("sock.recv");
                    {
                        let mut client: &mut Client = self.client;
                        let buf: &[u8] = mio::buf::Buf::bytes(&rx_buf);
                        if let Err(e) = client.rx(buf) {
                            println!("error: {:?}", e);
                            eloop.shutdown();
                        }
                    }
                },
                TCP => {
                    self.accept(eloop).ok().expect("TCP.accept");
                }
                _ => ()
            }
        }

        fn writable(&mut self, eloop: &mut mio::EventLoop<UdpHandler>, token: mio::Token) {
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
                                let mut buf = mio::buf::SliceBuf::wrap(ebuf.buf.as_slice());
                                if let Err(e) = self.sock.send_to(&mut buf, &self.addr) {
                                    println!("send_to error: {}", e);
                                    eloop.shutdown();
                                }
                            //} else {
                            //    println!("DROPPED!");
                            //}
                            
                            //self.client.txed();

                            if let Some(timeout) = ebuf.timeout {
                                //TODO use returned timeout handle to cancel timeout
                                println!("set {} timeout {} ms", timeout.seq, timeout.ms);
                                if let Err(e) = eloop.timeout_ms(timeout.seq, timeout.ms) {
                                    println!("eloop.timeout FAILED: {:?}", e);
                                    eloop.shutdown();
                                }
                            }

                            //self.start = false;
                        },
                        None => {}
                    }
                },
                _ => ()
            }
        }

        fn timeout (&mut self, /*eloop*/ _: &mut mio::EventLoop<UdpHandler>, timeout: usize) {
            self.client.timeout(timeout);
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
        }
    }

    let any = str::FromStr::from_str("0.0.0.0:0").ok().expect("any.from_str");
    let sock = mio::udp::bind(&any).ok().expect("bind");

    //FIXME sock.connect(&addr);
    sock.set_reuseaddr(true).ok().expect("set_reuseaddr");

    //TODO return Result and match
    let mut client = Client::new();

    //TODO FIXME get login/password from command line instead of storing them here
    match client.authorize("salvian", "простойпароль", "game.salemthegame.com", 1871) {
        Ok(()) => {
            println!("success. cookie = [{}]", client.cookie.as_slice().to_hex());
        },
        Err(e) => {
            println!("authorize error: {:?}", e);
            return;
        }
    };

    let mut eloop = mio::EventLoop::new().ok().expect("eloop.new");
    eloop.register_opt(&sock, UDP, mio::Interest::readable() | mio::Interest::writable(), mio::PollOpt::level()).ok().expect("eloop.register(udp)");

    let addr: std::net::SocketAddr = str::FromStr::from_str("127.0.0.1:33000").ok().expect("any.from_str");
    let tcp_listener = mio::tcp::TcpListener::bind(&addr).unwrap();
    eloop.register_opt(&tcp_listener, TCP, mio::Interest::readable(), mio::PollOpt::edge() | mio::PollOpt::oneshot()).unwrap();

    let ip = client.serv_ip;
    let mut handler = UdpHandler::new(sock, tcp_listener, &mut client, std::net::SocketAddr::new(ip, 1870));
    handler.client.connect().ok().expect("client.connect()");

    /*
    if let Err(e) = eloop.timeout_ms(123, 4000) {
        println!("eloop.timeout FAILED: {:?}", e);
        return;
    }
    */

    info!("run event loop");
    eloop.run(&mut handler).ok().expect("Failed to run the event loop");
}

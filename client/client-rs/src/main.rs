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
use std::collections::LinkedList;
use std::str;
use rustc_serialize::hex::ToHex;
use std::vec::Vec;

mod salem;
use salem::message::*;
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

    struct UdpHandler<'a> {
        sock: mio::NonBlock<mio::udp::UdpSocket>,
        addr: std::net::SocketAddr,
        tx_buf: LinkedList<Vec<u8>>,
        client: &'a mut Client,
        //start: bool,
    }

    impl<'a> UdpHandler<'a> {
        fn new(sock: mio::NonBlock<mio::udp::UdpSocket>, client:&'a mut Client, addr: std::net::SocketAddr) -> UdpHandler<'a> {
            UdpHandler {
                sock: sock,
                addr: addr,
                tx_buf: LinkedList::new(),
                client: client,
                //start: true,
            }
        }
    }

    const CLIENT: mio::Token = mio::Token(0);

    impl<'a> mio::Handler for UdpHandler<'a> {
        type Timeout = usize;
        type Message = ();

        fn readable(&mut self, eloop: &mut mio::EventLoop<UdpHandler>, token: mio::Token, _: mio::ReadHint) {
            match token {
                CLIENT => {
                    let mut rx_buf = mio::buf::RingBuf::new(65535);
                    self.sock.recv_from(&mut rx_buf).ok().expect("sock.recv");
                    let mut client: &mut Client = self.client;
                    let buf: &[u8] = mio::buf::Buf::bytes(&rx_buf);
                    if let Err(e) = client.dispatch_message(buf, &mut self.tx_buf) {
                        println!("error: {:?}", e);
                        eloop.shutdown();
                    }
                },
                _ => ()
            }
        }

        fn writable(&mut self, eloop: &mut mio::EventLoop<UdpHandler>, token: mio::Token) {
            match token {
                CLIENT => {
                    match self.tx_buf.pop_back() {
                        Some(data) => {
                            if let Ok((msg,_)) = Message::from_buf(data.as_slice(),MessageDirection::FromClient) {
                                println!("TX: {:?}", msg);
                            }
                            let mut buf = mio::buf::SliceBuf::wrap(data.as_slice());
                            if let Err(e) = self.sock.send_to(&mut buf, &self.addr) {
                                println!("send_to error: {}", e);
                                eloop.shutdown();
                            }
                            if !self.client.que.is_empty() {
                                //TODO use returned timeout handle to cancel timeout
                                if let Err(e) = eloop.timeout_ms(123, 300) {
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

        fn timeout (&mut self, eloop: &mut mio::EventLoop<UdpHandler>, /*timeout*/ _: usize) {
            let client = &self.client;
            match client.que.front() {
                Some(buf) => {
                    println!("re-enqueue to send by timeout");
                    self.tx_buf.push_front(buf.clone());
                    //TODO use returned timeout handle to cancel timeout
                    if let Err(e) = eloop.timeout_ms(123, 300) {
                        println!("eloop.timeout FAILED: {:?}", e);
                        eloop.shutdown();
                    }
                }
                None => {
                    println!("WARNING: timeout on empty que");
                }
            }
        }
    }

    let hostname = "game.salemthegame.com";

    let any = str::FromStr::from_str("0.0.0.0:0").ok().expect("any.from_str");
    let sock = mio::udp::bind(&any).ok().expect("bind");

    //FIXME sock.connect(&addr);
    sock.set_reuseaddr(true).ok().expect("set_reuseaddr");

    //TODO return Result and match
    let mut client = Client::new(/*"game.salemthegame.com", 1871, 1870*/);

    //TODO FIXME get login/password from command line instead of storing them here
    match client.authorize("salvian", "простойпароль", hostname, 1871) {
        Ok(()) => {
            println!("success. cookie = [{}]", client.cookie.as_slice().to_hex());
        },
        Err(e) => {
            println!("authorize error: {:?}", e);
            return;
        }
    };

    let mut event_loop = mio::EventLoop::new().ok().expect("mio.loop.new");
    event_loop.register_opt(&sock, CLIENT, mio::Interest::readable() |
                                           mio::Interest::writable(),
                                           mio::PollOpt::level()).ok().expect("loop.register_opt");
    let ip = client.serv_ip;
    let mut handler = UdpHandler::new(sock, &mut client, std::net::SocketAddr::new(ip, 1870));
    handler.client.connect(&mut handler.tx_buf); //TODO return Result and match

    info!("run event loop");
    event_loop.run(&mut handler).ok().expect("Failed to run the event loop");
}

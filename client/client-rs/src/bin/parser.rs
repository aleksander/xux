use std::env;
use std::io::Cursor;

#[macro_use]
extern crate nom;
use nom::IResult;
use nom::be_u8;

extern crate pcap;

extern crate sac;
use sac::proto::message::*;

extern crate pnet;
use pnet::packet::ethernet::EthernetPacket;
use pnet::packet::ethernet::EtherTypes::Ipv4;
use pnet::packet::ipv4::Ipv4Packet;
use pnet::packet::Packet;
use pnet::packet::ip::IpNextHeaderProtocols::Udp;
use pnet::packet::udp::UdpPacket;

#[derive(Clone,Copy)]
pub enum MessageDirection {
    FromClient,
    FromServer,
}

fn main() {

    let args: Vec<_> = env::args().collect();
    let nom_parser = if args.len() == 2 {
        false
    } else if args.len() == 3 {
        true
    } else {
        println!("Usage: {} <foo.pcap> [use_nom_parser]", args[0]);
        return;
    };

    let mut capture = pcap::Capture::from_file(&args[1]).expect("pcap::Capture::from_file");

    while let Ok(packet) = capture.next() {
        let eth = EthernetPacket::new(&packet.data[..]).expect("EthernetPacket::new");

        if eth.get_ethertype() != Ipv4 {
            continue;
        }

        let ip = Ipv4Packet::new(eth.payload()).expect("Ipv4Packet::new");

        if ip.get_next_level_protocol() != Udp {
            continue;
        }

        let udp = UdpPacket::new(ip.payload()).expect("UdpPacket::new");

        let dir = if udp.get_destination() == 1870 {
            MessageDirection::FromClient
        } else if udp.get_source() == 1870 {
            MessageDirection::FromServer
        } else {
            continue;
        };

        let dir_str = match dir {
            MessageDirection::FromServer => "SERVER",
            MessageDirection::FromClient => "CLIENT",
        };

        if nom_parser {
            match parse(udp.payload(), dir) {
                IResult::Done(i, o) => {
                    println!("{}: {:?}", dir_str, o);
                    if i.len() > 0 {
                        println!("REMAINS: {} bytes", i.len());
                    }
                }
                IResult::Error(e) => {
                    println!("Error: {:?}", e);
                    break;
                }
                IResult::Incomplete(n) => {
                    println!("Incomplete: {:?}", n);
                    break;
                }
            }
        } else {
            println!("");
            let mut r = Cursor::new(udp.payload());
            match dir {
                MessageDirection::FromClient => {
                    match ClientMessage::from_buf(&mut r) {
                        Ok((msg, remains)) => {
                            println!("CLIENT: {:?}", msg);
                            if let Some(buf) = remains {
                                println!("REMAINS {} bytes", buf.len());
                            }
                        }
                        Err(e) => {
                            println!("FAILED TO PARSE! ERROR: {:?}", e);
                            println!("BUF: {:?}", udp.payload());
                        }
                    }
                }
                MessageDirection::FromServer => {
                    match ServerMessage::from_buf(&mut r) {
                        Ok((msg, remains)) => {
                            println!("SERVER: {:?}", msg);
                            if let Some(buf) = remains {
                                println!("REMAINS {} bytes", buf.len());
                            }
                        }
                        Err(e) => {
                            println!("FAILED TO PARSE! ERROR: {:?}", e);
                            println!("BUF: {:?}", udp.payload());
                        }
                    }
                }
            }
        }
    }
}

fn parse_ssess(i: &[u8]) -> IResult<&[u8], &str> {
    IResult::Done(i, "S SESS")
}

fn parse_csess(i: &[u8]) -> IResult<&[u8], &str> {
    IResult::Done(i, "C SESS")
}

fn parse_rel(i: &[u8]) -> IResult<&[u8], &str> {
    IResult::Done(i, "REL")
}

fn parse_ack(i: &[u8]) -> IResult<&[u8], &str> {
    IResult::Done(i, "ACK")
}

fn parse_beat(i: &[u8]) -> IResult<&[u8], &str> {
    IResult::Done(i, "BEAT")
}

fn parse_mapreq(i: &[u8]) -> IResult<&[u8], &str> {
    IResult::Done(i, "MAPREQ")
}

fn parse_mapdata(i: &[u8]) -> IResult<&[u8], &str> {
    IResult::Done(i, "MAPDATA")
}

fn parse_objdata(i: &[u8]) -> IResult<&[u8], &str> {
    IResult::Done(i, "OBJDATA")
}

fn parse_objack(i: &[u8]) -> IResult<&[u8], &str> {
    IResult::Done(i, "OBJACK")
}

fn parse_close(i: &[u8]) -> IResult<&[u8], &str> {
    IResult::Done(i, "CLOSE")
}

// fn parser(input: &[u8]) -> IResult<&[u8], Msg> {
//    alt!(input,
//        msga_parser => { |res| Msg::A(res) } |
//        msgb_parser => { |res| Msg::B(res) } |
//        msgc_parser => { |res| Msg::C(res) }
//    )
// }

fn parse(i: &[u8], dir: MessageDirection) -> IResult<&[u8], &str> {
    match dir {
        MessageDirection::FromServer => parse_from_server(i),
        MessageDirection::FromClient => parse_from_client(i),
    }
}

fn parse_from_server(i: &[u8]) -> IResult<&[u8], &str> {
    match be_u8(i) {
        IResult::Done(i, o) => {
            match o {
                0 => parse_ssess(i),
                1 => parse_rel(i),
                2 => parse_ack(i),
                3 => IResult::Error(nom::ErrorKind::Tag)/*parse_beat(i)*/,
                4 => IResult::Error(nom::ErrorKind::Tag)/*parse_mapreq(i)*/,
                5 => parse_mapdata(i),
                6 => parse_objdata(i),
                7 => IResult::Error(nom::ErrorKind::Tag)/*parse_objack(i)*/,
                8 => parse_close(i),
                _ => IResult::Error(nom::ErrorKind::Tag)
            }
        }
        IResult::Error(e) => IResult::Error(e),
        IResult::Incomplete(n) => IResult::Incomplete(n),
    }
}

fn parse_from_client(i: &[u8]) -> IResult<&[u8], &str> {
    match be_u8(i) {
        IResult::Done(i, o) => {
            match o {
                0 => parse_csess(i),
                1 => parse_rel(i),
                2 => parse_ack(i),
                3 => parse_beat(i),
                4 => parse_mapreq(i),
                5 => IResult::Error(nom::ErrorKind::Tag)/*parse_mapdata(i)*/,
                6 => IResult::Error(nom::ErrorKind::Tag)/*parse_objdata(i)*/,
                7 => parse_objack(i),
                8 => parse_close(i),
                _ => IResult::Error(nom::ErrorKind::Tag)
            }
        }
        IResult::Error(e) => IResult::Error(e),
        IResult::Incomplete(n) => IResult::Incomplete(n),
    }
}

use std::env;
use std::fs;
//use std::io::Read;

#[macro_use]
extern crate nom;
use nom::IResult;
use nom::be_u8;
use nom::Err;

extern crate pcapng;
//use pcapng::block::parse_blocks;

extern crate client_rs;
use client_rs::message::*;

extern crate pnet;
use pnet::packet::ethernet::EthernetPacket;
use pnet::packet::ethernet::EtherTypes::Ipv4;
use pnet::packet::ipv4::Ipv4Packet;
use pnet::packet::Packet;
use pnet::packet::ip::IpNextHeaderProtocols::Udp;
use pnet::packet::udp::UdpPacket;

fn main () {

    let args: Vec<_> = env::args().collect();
    let nom_parser =
        if args.len() == 2 {
            false
        } else if args.len() == 3 {
            true
        } else {
            println!("Usage: {} <foo.pcapng> [use_nom_parser]", args[0]);
            return;
        };

    let mut f = fs::File::open(&args[1]).expect("file::open");
    //let mut buf: Vec<u8> = Vec::new();
    //let _ = fh.read_to_end(&mut buf);
    let mut r = pcapng::SimpleReader::new(&mut f);

    /*
    match pcapng::block::parse_blocks(&buf[..]) {
        IResult::Done(_, blocks) => {
            for i in blocks {
                println!("{:?}", i.parse());
            }
        }
        IResult::Error(e)      => panic!("Error: {:?}", e),
        IResult::Incomplete(i) => panic!("Incomplete: {:?}", i),

    }
    */

    for (iface, ref packet) in r.packets() {
        if iface.link_type != 1 {
            println!("not ethernet frame");
            continue
        }

        let eth = EthernetPacket::new(&packet.data[..]).expect("eth_pkt::new");

        if eth.get_ethertype() != Ipv4 {
            println!("not ipv4 packet");
            continue
        }

        let ip = Ipv4Packet::new(eth.payload()).expect("ipv4_pkt::new");

        if ip.get_next_level_protocol() != Udp {
            println!("not udp packet");
            continue
        }

        let udp = UdpPacket::new(ip.payload()).expect("udp_pkt::new");

        let dir = if udp.get_destination() == 1870 {
            MessageDirection::FromClient
        } else if udp.get_source() == 1870 {
            MessageDirection::FromServer
        } else {
            println!("not from 1870 or to 1870");
            continue
        };

        /*
        println!("{} > {}, {}:{} > {}:{}",
                 eth.get_source(),
                 eth.get_destination(),
                 ip.get_source(), udp.get_source(),
                 ip.get_destination(), udp.get_destination());
        */

        fn parse_ssess (i: &[u8]) -> IResult<&[u8], &str> {
            IResult::Done(i, "S SESS")
        }

        fn parse_csess (i: &[u8]) -> IResult<&[u8], &str> {
            IResult::Done(i, "C SESS")
        }

        fn parse_rel (i: &[u8]) -> IResult<&[u8], &str> {
            IResult::Done(i, "REL")
        }

        fn parse_ack (i: &[u8]) -> IResult<&[u8], &str> {
            IResult::Done(i, "ACK")
        }

        fn parse_beat (i: &[u8]) -> IResult<&[u8], &str> {
            IResult::Done(i, "BEAT")
        }

        fn parse_mapreq (i: &[u8]) -> IResult<&[u8], &str> {
            IResult::Done(i, "MAPREQ")
        }

        fn parse_mapdata (i: &[u8]) -> IResult<&[u8], &str> {
            IResult::Done(i, "MAPDATA")
        }

        fn parse_objdata (i: &[u8]) -> IResult<&[u8], &str> {
            IResult::Done(i, "OBJDATA")
        }

        fn parse_objack (i: &[u8]) -> IResult<&[u8], &str> {
            IResult::Done(i, "OBJACK")
        }

        fn parse_close (i: &[u8]) -> IResult<&[u8], &str> {
            IResult::Done(i, "CLOSE")
        }

        /*
        fn parser(input: &[u8]) -> IResult<&[u8], Msg> {
            alt!(input,
                msga_parser => { |res| Msg::A(res) } |
                msgb_parser => { |res| Msg::B(res) } |
                msgc_parser => { |res| Msg::C(res) }
            )
        }
        */

        fn parse (i: &[u8], dir: MessageDirection) -> IResult<&[u8], &str> {
            match dir {
                MessageDirection::FromServer => parse_from_server(i),
                MessageDirection::FromClient => parse_from_client(i),
            }
        }

        fn parse_from_server (i: &[u8]) -> IResult<&[u8], &str> {
            match be_u8(i) {
                IResult::Done(i, o) => {
                    match o {
                        0 => parse_ssess(i),
                        1 => parse_rel(i),
                        2 => parse_ack(i),
                        3 => IResult::Error(Err::Code(nom::ErrorKind::Tag))/*parse_beat(i)*/,
                        4 => IResult::Error(Err::Code(nom::ErrorKind::Tag))/*parse_mapreq(i)*/,
                        5 => parse_mapdata(i),
                        6 => parse_objdata(i),
                        7 => IResult::Error(Err::Code(nom::ErrorKind::Tag))/*parse_objack(i)*/,
                        8 => parse_close(i),
                        _ => IResult::Error(Err::Code(nom::ErrorKind::Tag))
                    }
                }
                IResult::Error(e) => IResult::Error(e),
                IResult::Incomplete(n) => IResult::Incomplete(n),
            }
        }

        fn parse_from_client (i: &[u8]) -> IResult<&[u8], &str> {
            match be_u8(i) {
                IResult::Done(i, o) => {
                    match o {
                        0 => parse_csess(i),
                        1 => parse_rel(i),
                        2 => parse_ack(i),
                        3 => parse_beat(i),
                        4 => parse_mapreq(i),
                        5 => IResult::Error(Err::Code(nom::ErrorKind::Tag))/*parse_mapdata(i)*/,
                        6 => IResult::Error(Err::Code(nom::ErrorKind::Tag))/*parse_objdata(i)*/,
                        7 => parse_objack(i),
                        8 => parse_close(i),
                        _ => IResult::Error(Err::Code(nom::ErrorKind::Tag))
                    }
                }
                IResult::Error(e) => IResult::Error(e),
                IResult::Incomplete(n) => IResult::Incomplete(n),
            }
        }

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
            match Message::from_buf(udp.payload(), dir) {
                Ok((msg,remains)) => {
                    println!("{}: {:?}", dir_str, msg);
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

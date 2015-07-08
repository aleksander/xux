use std::env;
use std::fs;
//use std::io::Read;

//extern crate nom;
//use nom::IResult;

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
    if args.len() != 2 {
        println!("Usage: {} <foo.pcapng>", args[0]);
        return;
    }

    let mut f = fs::File::open(&args[1]).unwrap();
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

        let eth = EthernetPacket::new(&packet.data[..]).unwrap();

        if eth.get_ethertype() != Ipv4 {
            println!("not ipv4 packet");
            continue
        }

        let ip = Ipv4Packet::new(eth.payload()).unwrap();

        if ip.get_next_level_protocol() != Udp {
            println!("not udp packet");
            continue
        }

        let udp = UdpPacket::new(ip.payload()).unwrap();

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

        println!("");
        match Message::from_buf(udp.payload(), dir) {
            Ok((msg,remains)) => {
                println!("{:?}", msg);
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

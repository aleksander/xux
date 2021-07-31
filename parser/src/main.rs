use std::env;
use xux::proto::message::*;
use pnet::packet::{
    ethernet::{
        EthernetPacket,
        EtherTypes::Ipv4
    },
    ipv4::Ipv4Packet,
    Packet,
    ip::IpNextHeaderProtocols::Udp,
    udp::UdpPacket,
};
use pcap::Capture;

#[derive(Clone,Copy)]
pub enum MessageDirection {
    FromClient,
    FromServer,
}

fn main() {
    let args: Vec<_> = env::args().collect();
    if args.len() != 2 {
        println!("Usage: {} <foo.pcap>", args[0]);
        return;
    };

    let mut capture = Capture::from_file(&args[1]).expect("pcap::Capture::from_file");

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

        println!("");
        let mut r = udp.payload();
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
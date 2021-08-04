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
use clap::App;

#[derive(Clone,Copy)]
pub enum MessageDirection {
    FromClient,
    FromServer,
}

fn main() {
    let matches = App::new("parser")
        .about("Hafen protocol parser")
        //TODO < -c | -s >
        .arg("-c, --client 'Parse and show client messages only'")
        .arg("-s, --server 'Parse and show server messages only'")
        .arg("<PCAP> 'pcap file to parse'")
        .get_matches();

    let input_file = matches.value_of("PCAP").unwrap();

    let show_client = matches.is_present("client");
    let show_server = matches.is_present("server");
    let show_both = (show_client && show_server) || (!show_client && !show_server);
    let show_client = show_client || show_both;
    let show_server = show_server || show_both;

    let mut capture = Capture::from_file(&input_file).expect("pcap::Capture::from_file");

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

        let mut r = udp.payload();
        match dir {
            MessageDirection::FromClient => {
                if show_client {
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
                    println!("");
                }
            }
            MessageDirection::FromServer => {
                if show_server {
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
                    println!("");
                }
            }
        }
    }
}
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
use pcap::{Capture, Linktype};
use clap::{App, Arg};
use std::num::Wrapping;

#[derive(Clone,Copy)]
pub enum MessageDirection {
    FromClient,
    FromServer,
}

#[derive(Debug,PartialEq)]
enum MsgType { Sess, Rel, Ack, Beat, Mapreq, Mapdata, Objdata, Objack, Close }
impl MsgType {
    fn from_str (t: &str) -> Self {
        use MsgType::*;
        match t {
            "SESS" => Sess,
            "REL" => Rel,
            "ACK" => Ack,
            "BEAT" => Beat,
            "MAPREQ" => Mapreq,
            "MAPDATA" => Mapdata,
            "OBJDATA" => Objdata,
            "OBJACK" => Objack,
            "CLOSE" => Close,
            _ => panic!("unexpected message type")
        }
    }
}

fn main() {
    let matches = App::new("parser")
        .about("Hafen protocol parser")
        //TODO < -c | -s >
        .arg("-c, --client 'Parse and show client messages only'")
        .arg("-s, --server 'Parse and show server messages only'")
        //.arg("-t, --type [TYPE]... 'Show messages of specified types only (can be any of SESS, REL, ACK, BEAT, MAPREQ, MAPDATA, OBJDATA, OBJACK, CLOSE)'")
        .arg(Arg::new("type")
                 .long("type")
                 .short('t')
                 .about("Show messages of specified types only")
                 .takes_value(true)
                 .possible_values(&["SESS", "REL", "ACK", "BEAT", "MAPREQ", "MAPDATA", "OBJDATA", "OBJACK", "CLOSE"])
                 .multiple_occurrences(true)
                 .multiple_values(true)
                 .require_delimiter(true)
                 .use_delimiter(true))
        .arg("-f, --follow 'Follow the REL stream'")
        .arg("<PCAP> 'pcap file to parse'")
        .get_matches();

    let input_file = matches.value_of("PCAP").unwrap();

    let show_client = matches.is_present("client");
    let show_server = matches.is_present("server");
    let show_both = (show_client && show_server) || (!show_client && !show_server);
    let show_client = show_client || show_both;
    let show_server = show_server || show_both;

    let types = if let Some(types) = matches.values_of("type") {
        types.collect::<Vec<_>>()
    } else {
        vec!()
    };
    let types: Vec<MsgType> = types.iter().map(|&t| MsgType::from_str(t)).collect();

    let follow = matches.is_present("follow");
    let mut client_seq = Wrapping(0);
    let mut server_seq = Wrapping(0);

    let mut capture = Capture::from_file(&input_file).expect("pcap::Capture::from_file");

    let datalink = capture.get_datalink();
    println!("capture datalink: {:?}", datalink);

    match datalink {
        Linktype(1) => { println!("ethernet datalink") }
        Linktype(12) => { println!("raw ip datalink") }
        Linktype(other) => panic!("unsupported datalink type {}", other)
    }

    while let Ok(packet) = capture.next() {
        let mut data = packet.data.to_owned(); //FIXME to_owned() required here because of lifetime issue in EthernetPacket::payload()
        if let Linktype(1) = datalink {
            let eth = EthernetPacket::new(&data[..]).expect("EthernetPacket::new");
            if eth.get_ethertype() != Ipv4 {
                continue;
            }
            data = eth.payload().to_owned();
        }
        let ip = Ipv4Packet::new(&data[..]).expect("Ipv4Packet::new");
        if ip.get_next_level_protocol() != Udp {
            continue;
        }
        data = ip.payload().to_owned();
        let udp = UdpPacket::new(&data[..]).expect("UdpPacket::new");
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
                            let show_message =
                                if types.is_empty() {
                                    true
                                } else {
                                    match msg {
                                        ClientMessage::ACK(_) => types.contains(&MsgType::Ack),
                                        ClientMessage::BEAT(_) => types.contains(&MsgType::Beat),
                                        ClientMessage::CLOSE(_) => types.contains(&MsgType::Close),
                                        ClientMessage::MAPREQ(_) => types.contains(&MsgType::Mapreq),
                                        ClientMessage::OBJACK(_) => types.contains(&MsgType::Objack),
                                        ClientMessage::REL(_) => types.contains(&MsgType::Rel),
                                        ClientMessage::SESS(_) => types.contains(&MsgType::Sess),
                                    }
                                };
                            if show_message {
                                if follow && msg.is_rel() {
                                    if let ClientMessage::REL(rels) = msg {
                                        let mut seq = Wrapping(rels.seq);
                                        for rel in &rels.rels {
                                            if seq == client_seq {
                                                println!("{:?}", rel);
                                                client_seq += Wrapping(1);
                                            }
                                            seq += Wrapping(1);
                                        }
                                    }
                                } else {
                                    println!("CLIENT: {:?}", msg);
                                }
                                if let Some(buf) = remains {
                                    println!("REMAINS {} bytes", buf.len());
                                }
                                if ! follow {
                                    println!();
                                }
                            }
                        }
                        Err(e) => {
                            println!("FAILED TO PARSE! ERROR: {:?}", e);
                            println!("BUF: {:?}", udp.payload());
                            println!();
                        }
                    }
                }
            }
            MessageDirection::FromServer => {
                if show_server {
                    match ServerMessage::from_buf(&mut r) {
                        Ok((msg, remains)) => {
                            let show_message =
                                if types.is_empty() {
                                    true
                                } else {
                                    match msg {
                                        ServerMessage::ACK(_) => types.contains(&MsgType::Ack),
                                        ServerMessage::SESS(_) => types.contains(&MsgType::Sess),
                                        ServerMessage::REL(_) => types.contains(&MsgType::Rel),
                                        ServerMessage::MAPDATA(_) => types.contains(&MsgType::Mapdata),
                                        ServerMessage::OBJDATA(_) => types.contains(&MsgType::Objdata),
                                        ServerMessage::CLOSE(_) => types.contains(&MsgType::Close),
                                    }
                                };
                            if show_message {
                                if follow && msg.is_rel() {
                                    if let ServerMessage::REL(rels) = msg {
                                        let mut seq = Wrapping(rels.seq);
                                        for rel in &rels.rels {
                                            if seq == server_seq {
                                                println!("{:?}", rel);
                                                server_seq += Wrapping(1);
                                            }
                                            seq += Wrapping(1);
                                        }
                                    }
                                } else {
                                    println!("SERVER: {:?}", msg);
                                }
                                if let Some(buf) = remains {
                                    println!("REMAINS {} bytes", buf.len());
                                }
                                if ! follow {
                                    println!();
                                }
                            }
                        }
                        Err(e) => {
                            println!("FAILED TO PARSE! ERROR: {:?}", e);
                            println!("BUF: {:?}", udp.payload());
                            println!();
                        }
                    }
                }
            }
        }
    }
}
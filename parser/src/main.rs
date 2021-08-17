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
use clap::{App, Arg};

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
                                println!("CLIENT: {:?}", msg);
                                if let Some(buf) = remains {
                                    println!("REMAINS {} bytes", buf.len());
                                }
                                println!();
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
                                println!("SERVER: {:?}", msg);
                                if let Some(buf) = remains {
                                    println!("REMAINS {} bytes", buf.len());
                                }
                                println!();
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
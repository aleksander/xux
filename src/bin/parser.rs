use std::env;
use std::io::Cursor;

use nom::{
    IResult, named, switch, do_parse, map_res, map, take, take_while, call,
    number::complete::{le_u8, le_u16},
};

use xux::proto;
use xux::proto::message::*;

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

        if nom_parser {
            match dir {
                MessageDirection::FromServer => match parse_server_message(udp.payload()) {
                    Ok((i, o)) => {
                        println!("SERVER: {:?}", o);
                        if i.len() > 0 {
                            println!("REMAINS: {} bytes", i.len());
                        }
                    }
                    Err(e) => {
                        println!("Error: {:?}", e);
                        break;
                    }
                },
                MessageDirection::FromClient => match parse_client_message(udp.payload()) {
                    Ok((i, o)) => {
                        println!("CLIENT: {:?}", o);
                        if i.len() > 0 {
                            println!("REMAINS: {} bytes", i.len());
                        }
                    }
                    Err(e) => {
                        println!("Error: {:?}", e);
                        break;
                    }
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

fn parse_ssess(i: &[u8]) -> IResult<&[u8], proto::sSess> {
    le_u8(i).map(|(i,o)|(i,proto::sSess::new(o)))
}

named!(strz<&[u8], &str>,
    map_res!(
        do_parse!(
            s: take_while!(call!(|c| c != 0)) >>
            take!(1) >>
            (s)
        ),
        std::str::from_utf8
    )
);

#[cfg(test)]

#[cfg(test)]
use nom::Err;
#[cfg(test)]
use nom::Needed;

#[test]
fn test_strz() {
    assert_eq!( strz(b"aaa\0bbb"), Ok((&b"bbb"[..], "aaa")) );
    assert_eq!( strz(b"\0aaa\0bbb"), Ok((&b"aaa\0bbb"[..], "")) );
    assert_eq!( strz(b"aaabbb"), Err(Err::Incomplete(Needed::new(7))) );
}


//fn parse_csess(i: &[u8]) -> IResult<&[u8], proto::cSess>
named!(parse_csess<&[u8], proto::cSess>,
    do_parse!(
        _unknown: le_u16 >>
        _proto: strz >>
        _version: le_u16 >>
        login: strz >>
        cookie_len: le_u16 >>
        cookie: take!(cookie_len) >>
        (proto::cSess::new(login.into(), cookie.into()))
    )
);

#[test]
fn test_parse_csess() {
    assert_eq!(
        parse_csess(b"\x00\x00Salem\0\x00\x00User\0\x20\x00cookiecookiecookiecookiecookie12"),
        Ok((&b""[..], proto::cSess::new("User".into(), "cookiecookiecookiecookiecookie12".into())))
    );
}

fn parse_rel(i: &[u8]) -> IResult<&[u8], proto::Rels> {
    //TODO do parse
    Ok((i, proto::Rels::new(0)))
}

fn parse_ack(i: &[u8]) -> IResult<&[u8], proto::Ack> {
    //TODO do parse
    Ok((i, proto::Ack::new(0)))
}

fn parse_beat(i: &[u8]) -> IResult<&[u8], proto::Beat> {
    //TODO do parse
    Ok((i, proto::Beat))
}

fn parse_mapreq(i: &[u8]) -> IResult<&[u8], proto::MapReq> {
    //TODO do parse
    Ok((i, proto::MapReq::new(0, 0)))
}

fn parse_mapdata(i: &[u8]) -> IResult<&[u8], proto::MapData> {
    //TODO do parse
    Ok((i, proto::MapData::new(0, 0, 0, vec!())))
}

fn parse_objdata(i: &[u8]) -> IResult<&[u8], proto::ObjData> {
    //TODO do parse
    Ok((i, proto::ObjData::new(vec!())))
}

fn parse_objack(i: &[u8]) -> IResult<&[u8], proto::ObjAck> {
    //TODO do parse
    Ok((i, proto::ObjAck::new(vec!())))
}

fn parse_close(i: &[u8]) -> IResult<&[u8], proto::Close> {
    //TODO do parse
    Ok((i, proto::Close))
}

named!(parse_server_message <&[u8], ServerMessage>,
    switch!(le_u8,
        0 => map!(parse_ssess, |o|ServerMessage::SESS(o)) |
        1 => map!(parse_rel, |o|ServerMessage::REL(o)) |
        2 => map!(parse_ack, |o|ServerMessage::ACK(o)) |
        //3
        //4
        5 => map!(parse_mapdata, |o|ServerMessage::MAPDATA(o)) |
        6 => map!(parse_objdata, |o|ServerMessage::OBJDATA(o)) |
        //7
        8 => map!(parse_close, |o|ServerMessage::CLOSE(o))
    )
);

named!(parse_client_message <&[u8], ClientMessage>,
    switch!(le_u8,
        0 => map!(parse_csess, |o|ClientMessage::SESS(o)) |
        1 => map!(parse_rel, |o|ClientMessage::REL(o)) |
        2 => map!(parse_ack, |o|ClientMessage::ACK(o)) |
        3 => map!(parse_beat, |o|ClientMessage::BEAT(o)) |
        4 => map!(parse_mapreq, |o|ClientMessage::MAPREQ(o)) |
        //5
        //6
        7 => map!(parse_objack, |o|ClientMessage::OBJACK(o)) |
        8 => map!(parse_close, |o|ClientMessage::CLOSE(o))
    )
);

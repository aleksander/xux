extern crate sac;

fn go(data: &[u8]) {
    use std::io::Cursor;
    use sac::proto::message::*;
    // fuzzed code goes here
    let mut r = Cursor::new(data);
    println!("{:?}", ServerMessage::from_buf(&mut r));
}

fn main () {
    go(&[0x1,0x8,0x8d,0x0,0x0,0x0,0x0,0x21,0x21,0x0,0x2,0x0,0xe,0x8d,0x0,0x0,0x0,0x0,0x21,0x21,0x0,0x2,0x0]);
    go(&[0x1,0x0,0x0,0x1,0x0,0xe,0x0,0xe,0xef,0xf1,0xf1,0xf1,0x6b,0x6b,0x6b,0x6b,0x1,0x1,0x8,0x8d,
         0x0,0x0,0x0,0x0,0x0,0xdf,0xfe,0x21,0x0,0x0,0x6b,0x6b,0x6b,0x6b,0x6b,0x1,0x0,0x0,0x0,0x0,
         0x0,0x0,0xc,0x0,0x1,0x0,0x6b,0x6b,0x0,0x6b,0x6b,0x6b,0xc,0x6b,0x6b,0x6b,0x0,0x6b,0xdb,0x0,0x0,0xdb,0x3b,0x4]);
    go(&[0x01, 0x01, 0x08, 0x8d, 0x00, 0x00, 0x00, 0x00, 0x06, 0x00, 0x00, 0x00,
        0x0e, 0x0e, 0x0e, 0x0e, 0x0e, 0x0e, 0x0e, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x0e, 0x0e, 0x0e, 0x0e, 0x0e, 0x0e, 0x0e, 0x0e, 0x0e,
        0x0e, 0x0e, 0x0e, 0x0e, 0x0e, 0x0e, 0x0e, 0x0e, 0x0e, 0xb5, 0x0e, 0x0e,
        0x0e, 0x12, 0x12, 0x12, 0x12, 0x12, 0x00, 0x12, 0x21, 0x21, 0x12, 0x01,
        0x00, 0x00, 0x00]);
}
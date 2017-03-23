#![no_main]
extern crate libfuzzer_sys;
extern crate sac;
#[export_name="rust_fuzzer_test_input"]
pub extern fn go(data: &[u8]) {
    use std::io::Cursor;
    use sac::proto::message::*;
    // fuzzed code goes here
    let mut r = Cursor::new(data);
    let _msg = ClientMessage::from_buf(&mut r);
}

use std::sync::mpsc::Sender;
use std::sync::mpsc::RecvError;

pub trait Driver {
    fn tx(&self, buf: &[u8]) -> ::std::io::Result<()>;
    fn timeout(&self, seq: usize, ms: u64);
    fn event(&mut self) -> Result<Event, RecvError>;
}

pub enum Event {
    Rx(Vec<u8>),
    Timeout(usize),
    Tcp((Sender<String>, Vec<u8>)),
}

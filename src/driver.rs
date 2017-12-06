use std::sync::mpsc::Sender;
use Result;

pub trait Driver {
    fn tx(&self, buf: &[u8]) -> Result<()>;
    fn timeout(&self, seq: usize, ms: u64);
    fn event(&mut self) -> Result<Event>;
}

pub enum Event {
    Rx(Vec<u8>),
    Timeout(usize),
    Tcp((Sender<String>, Vec<u8>)),
    Render(RenderEvent),
}

pub enum RenderEvent {
    Up,
    Down,
    Left,
    Right,
    Quit,
}

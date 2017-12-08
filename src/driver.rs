use Result;

pub trait Driver {
    fn tx(&self, buf: &[u8]) -> Result<()>;
    fn timeout(&self, seq: usize, ms: u64);
    fn event(&mut self) -> Result<Event>;
}

#[derive(Debug)]
pub enum Event {
    Rx(Vec<u8>),
    Timeout(usize),
    Render(RenderEvent),
}

#[derive(Debug)]
pub enum RenderEvent {
    Up,
    Down,
    Left,
    Right,
    Quit,
}

use state::Event;
use driver;
use std::sync::mpsc::Sender;

pub struct RenderImpl;
impl RenderImpl {
    pub fn new () -> RenderImpl { RenderImpl }
    pub fn init (&self) {}
    pub fn event (&self, _event: Event) {}
    pub fn update (&mut self, render_tx: &Sender<driver::Event>) -> bool { true }
    pub fn end (self) {}
}

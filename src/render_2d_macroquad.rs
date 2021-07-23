use crate::state::Event;
use crate::driver;
use std::sync::mpsc::Sender;
use macroquad::prelude::*;

pub struct RenderImpl;

impl RenderImpl {
    pub fn new () -> RenderImpl { RenderImpl }
    pub fn init (&self) {}
    pub fn event (&self, _event: Event) {}
    pub fn update (&mut self, _render_tx: &Sender<driver::Event>) -> bool { true }
    pub fn end (self) {}
}

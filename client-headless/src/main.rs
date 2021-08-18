use log::{info, error};
use std::sync::mpsc::{Sender, Receiver, TryRecvError::*};
use anyhow::{anyhow, Result};
use xux::{client, driver, state};
use std::error::Error;

fn main () -> Result<()> {

    info!("Starting...");

    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        error!("wrong argument count");
        error!("usage: {} username password", args[0]);
        return Err(anyhow!("wrong argument count"));
    }

    let username = args[1].clone();
    let password = args[2].clone();

    let host = "game.havenandhearth.com";

    let auth_port = 1871;
    let game_port = 1870;

    let (login, cookie) = client::authorize(host, auth_port, username, password)?;

    let (ll_event_tx, hl_event_rx) = client::run_threaded(host, game_port, login, cookie)?;

    let mut render_ctx = RenderContext::new(ll_event_tx, hl_event_rx);

    loop {
        render_ctx.update();
        //TODO signal handling
    }
    info!("render thread: done");
    Ok(())
}

struct RenderContext {
    event_tx: Sender<driver::Event>,
    event_rx: Receiver<state::Event>,
    should_exit: bool,
}

impl RenderContext {
    fn new (event_tx: Sender<driver::Event>, event_rx: Receiver<state::Event>) -> Self {
        Self {
            should_exit: false,
            event_rx,
            event_tx,
        }
    }
    fn update (&mut self) {
        loop {
            match self.event_rx.try_recv() {
                Ok(_event) => {}
                Err(Empty) => { break; }
                Err(Disconnected) => {
                    info!("render: disconnected from que");
                    self.should_exit = true;
                    break;
                }
            }
        }
    }
}
//extern crate crossbeam_channel as channel;

use std::thread;
use std::sync::mpsc::Sender;
use signal_hook::{iterator::Signals, SIGINT};
use crate::driver::Event;
use crate::Result;
//TODO use crossbeam_channel::channel;

pub fn init (sender: Sender<Event>) -> Result<()> {
    //let (s, r) = channel::bounded(100);
    let signals = Signals::new(&[SIGINT])?;
    thread::Builder::new().name("signals".into()).spawn(move || {
        for _ in signals.forever() {
            if sender.send(Event::SigInt).is_err() {
                break;
            }
        }
    })?;
    Ok(())
}
use std::sync::mpsc::{Sender, Receiver};
use std::thread;
use std::sync::mpsc::TryRecvError::*;
use log::info;
use crate::driver;
use crate::state;

#[cfg(feature = "dump_events")]
mod dumper {
    use std::fs::File;
    use std::io::BufWriter;
    use std::io::Write;
    use render::Event;
    use errors::*;

    pub struct Dumper(BufWriter<File>);

    impl Dumper {
        pub fn init() -> Result<Dumper> {
            let f = File::create("events.dump").chain_err(||"File::create")?;
            let writer = BufWriter::new(f);
            Ok(Dumper(writer))
        }
        pub fn dump(&mut self, event: &Event) -> Result<()> {
            use bincode::{serialize, Infinite};
            let serialized: Vec<u8> = serialize(event, Infinite).chain_err(||"unable to serialize event")?;
            let mut len = serialized.len();
            for _ in 0..8 {
                self.0.write(&[len as u8]).chain_err(||"unable to write serialized len")?;
                len >>= 8;
            }
            self.0.write(&serialized).chain_err(||"unable to write serialized")?;
            Ok(())
        }
    }
}

#[cfg_attr(feature = "render_none", path = "render_none.rs")]
#[cfg_attr(feature = "render_text", path = "render_text.rs")]
#[cfg_attr(feature = "render_2d_piston", path = "render_2d_piston.rs")]
#[cfg_attr(feature = "render_3d_glium", path = "render_3d_glium.rs")]
#[cfg_attr(feature = "render_2d_gfx", path = "render_2d_gfx.rs")]
#[cfg_attr(feature = "render_2d_macroquad", path = "render_2d_macroquad.rs")]
#[cfg_attr(feature = "render_2d_wgpu", path = "render_2d_wgpu.rs")]
mod render_impl;

pub fn new (ll_que_tx: Sender<driver::Event>, hl_que_rx: Receiver<state::Event>) {
    thread::Builder::new().name("render".to_string()).spawn(move || {
        let mut render_impl = render_impl::RenderImpl::new(); //TODO pass render_tx to new()
        #[cfg(feature = "dump_events")]
        let mut dumper = dumper::Dumper::init().expect("unable to create dumper");
        render_impl.init();
        'outer: loop {
            if ! render_impl.update(&ll_que_tx) {
                info!("unable to render_impl.update()");
                break;
            }
            loop {
                match hl_que_rx.try_recv() {
                    Ok(event) => {
                        #[cfg(feature = "dump_events")]
                        dumper.dump(&event).expect("unable to dump event");
                        render_impl.event(event);
                    }
                    Err(Empty) => { break; }
                    Err(Disconnected) => {
                        info!("render: disconnected from que");
                        break 'outer;
                    }
                }
            }
        }
        render_impl.end();
        info!("render thread: done");
    }).expect("unable to create render thread");
}

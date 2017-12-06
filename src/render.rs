use std::sync::mpsc::Sender;
use std::thread;
use driver;
use proto::*;
use state::Wdg;
use std::slice::Iter;
use std::iter::Iterator;
use std::sync::mpsc::channel;
use std::sync::mpsc::TryRecvError::*;
use failure::err_msg;
use Result;

#[derive(Serialize,Deserialize,Debug)]
pub enum Event {
    Tiles(Vec<Tile>),
    Grid(i32, i32, Vec<u8>, Vec<i16>, Vec<u8>),
    Obj(ObjID, ObjXY, ResID),
    ObjRemove(ObjID),
    Res(ResID, String),
    Hero(ObjXY),
    // NewObj(i32,i32),
    // UpdObj(...),
    // AI(...ai desigions...),
    // AI: going to pick obj (ID)
    // AI: going by path (PATH CHAIN)
    Input,
    Wdg(Wdg),
}

struct Widget {
    id: u16,
    name: String,
    children: Vec<Widget>,
    messages: Vec<String>,
}

impl Widget {
    fn new (id: u16, name: String) -> Widget {
        Widget {
            id: id,
            name: name,
            children: Vec::new(),
            messages: Vec::new(),
        }
    }

    fn add (&mut self, wdg: Widget) {
        self.children.push(wdg)
    }

    fn find (&mut self, id: u16) -> Option<&mut Widget> {
        if id == self.id { return Some(self); }
        for wdg in self.children.iter_mut() {
            if wdg.id == id {
                return Some(wdg);
            }
            if let Some(wdg) = wdg.find(id) {
                return Some(wdg);
            }
        }
        None
    }

    fn del (&mut self, id: u16) -> Result<()> {
        let mut index = None;
        for (i,wdg) in self.children.iter_mut().enumerate() {
            if wdg.id == id {
                index = Some(i);
                break;
            }
            if let Ok(()) = wdg.del(id) {
                return Ok(());
            }
        }
        if let Some(i) = index {
            self.children.remove(i);
            return Ok(());
        }
        Err(err_msg("unable to find widget"))
    }

    fn message (&mut self, msg: String) {
        self.messages.push(msg);
    }
}

struct XUi {
    root: Widget,
}

impl XUi {
    fn new () -> XUi {
        XUi {
            root: Widget::new(0, "root".into())
        }
    }

    //fn find_widget (&mut self, id: u16) -> Option<&mut Widget> {
    //    self.root.find(id)
    //}

    fn add_widget (&mut self, id: u16, name: String, parent: u16) -> Result<()> {
        debug!("adding widget {} '{}' [{}]", id, name, parent);
        self.root.find(parent).ok_or(err_msg("unable to find widget"))?.add(Widget::new(id, name));
        Ok(())
    }

    fn del_widget (&mut self, id: u16) -> Result<()> {
        debug!("deleting widget {}", id);
        self.root.del(id)
    }

    fn message (&mut self, id: u16, msg: String) -> Result<()> {
        debug!("message to widget {} '{}'", id, msg);
        self.root.find(id).ok_or(err_msg("unable to find widget"))?.message(msg);
        Ok(())
    }

    fn widgets_iter (&self) -> UiWidgetIter {
        let mut stack = Vec::new();
        stack.push(self.root.children.iter());
        UiWidgetIter {
            stack: stack
        }
    }
}

struct UiWidgetIter <'a> {
    stack: Vec<Iter<'a, Widget>>
}

impl <'a> Iterator for UiWidgetIter <'a> {
    type Item = (usize, &'a Widget);

    fn next(&mut self) -> Option<(usize, &'a Widget)> {
        loop {
            if self.stack.is_empty() { return None; }
            let len = self.stack.len();
            match self.stack[len - 1].next() {
                Some(wdg) => {
                    let next = (len, wdg);
                    if ! wdg.children.is_empty() {
                        self.stack.push(wdg.children.iter());
                    }
                    return Some(next);
                }
                None => {
                    self.stack.pop();
                }
            }
        }
    }
}

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
mod render_impl;

pub struct Render {
    tx: Sender<Event>,
}

impl Render {
    pub fn new(render_tx: Sender<driver::Event>) -> Render {
        let (tx, rx) = channel();

        thread::Builder::new().name("render".to_string()).spawn(move || {
            let mut render_impl = render_impl::RenderImpl::new(); //TODO pass render_tx to new()
            #[cfg(feature = "dump_events")]
            let mut dumper = dumper::Dumper::init().expect("unable to create dumper");
            render_impl.init();
            'outer: loop {
                if ! render_impl.update(&render_tx) { break; }
                loop {
                    match rx.try_recv() {
                        Ok(event) => {
                            #[cfg(feature = "dump_events")]
                            dumper.dump(&event).expect("unable to dump event");
                            render_impl.event(event);
                        }
                        Err(Empty) => { break; }
                        Err(Disconnected) => { break 'outer; }
                    }
                }
            }
            render_impl.end();
        }).expect("unable to create render thread");

        Render {
            tx: tx,
        }
    }

    pub fn update(&mut self, event: Event) -> Result<()> {
        Ok(self.tx.send(event)?)
    }
}

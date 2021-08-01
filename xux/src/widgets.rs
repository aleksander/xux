use std::slice::Iter;
use std::iter::Iterator;
use log::debug;
use crate::Result;
use crate::proto::list::List;
use anyhow::anyhow;

pub struct Widget {
    pub id: u16,
    name: String,
    children: Vec<Widget>,
    pub messages: Vec<(String,Vec<List>)>,
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

    fn find_child_by_name (&self, name: &str) -> Option<&Widget> {
        for child in self.children.iter() {
            if child.name.as_str() == name {
                return Some(child);
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
        Err(anyhow!("unable to find widget"))
    }

    fn message (&mut self, msg: (String,Vec<List>)) {
        self.messages.push(msg);
    }
}

pub struct Widgets {
    root: Widget,
}

impl Widgets {
    pub fn new () -> Widgets {
        Widgets {
            root: Widget::new(0, "root".into())
        }
    }

    pub fn add_widget (&mut self, id: u16, name: String, parent: u16) -> Result<()> {
        debug!("adding widget {} '{}' [{}]", id, name, parent);
        self.root.find(parent).ok_or(anyhow!("unable to find widget"))?.add(Widget::new(id, name));
        Ok(())
    }

    pub fn del_widget (&mut self, id: u16) -> Result<()> {
        debug!("deleting widget {}", id);
        self.root.del(id)
    }

    pub fn message (&mut self, id: u16, msg: (String,Vec<List>)) -> Result<()> {
        debug!("message to widget {} '{}'", id, msg.0);
        self.root.find(id).ok_or(anyhow!("unable to find widget"))?.message(msg);
        Ok(())
    }

    pub fn find_chain (&self, names: &[&str]) -> Option<&Widget> {
        let mut widget = &self.root;
        for name in names.iter() {
            let child = widget.find_child_by_name(name);
            if let Some(child) = child {
                widget = child;
            } else {
                return None;
            }
        }
        Some(widget)
    }

    /*
    fn widgets_iter (&self) -> UiWidgetIter {
        let mut stack = Vec::new();
        stack.push(self.root.children.iter());
        UiWidgetIter {
            stack: stack
        }
    }
    */
}

//TODO FIXME replace by something like in Message List
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

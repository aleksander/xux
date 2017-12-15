use std::sync::mpsc::{Sender, Receiver};
use state;
use driver;
use widgets::Widgets;
use proto::list::List;
use Result;

#[derive(Debug)]
enum AiState {
    WaitForCharList,
    WaitForWorld,
    Walking,
    Walking1,
    Walking2,
    WaitForEnd,
}

#[derive(Debug)]
enum Step {
    A,
    B,
    C,
    D,
}

pub struct Ai {
    state: AiState,
    step: Step,
    cycle: usize,
}

impl Ai {
    pub fn new() -> Ai {
        Ai {
            state: AiState::WaitForCharList,
            step: Step::A,
            cycle: 0,
        }
    }

    /*
    // more looks like RON config
    script = """
        char: create RandomName
        behavior: collect food
    """;
    script = """
        char: create RandomName
        behavior: eat_until_restore_strength fell_trees
    """;
    script = """
        char: choose Клёцка
        behavior: collect curiosities
    """;
    ai_state = "waiting for char list";
    if let Some(ref charlist) = widgets.widget(["root","ccnt","charlist"]) {
        let char_count = charlist.messages.iter().filter(|m|m.name == "add").count();
        if char_count == 0 {
            // no characters were created
            // TODO create a new one
        } else if char_count == 1 {
            // there is only one char
            // TODO select this one or __wait UI__ for user decision
        } else {
            // multiple chars available
            // TODO select first one or __wait UI__ for user decision
        }
        for message in charlist.messages {
            if message.name == "add" {
                if let Some(Str(char_name)) = message.args.get(0) {
                    send(Event(WidgetMessage(charlist.id, "play", [Str(char_name)])
                }
            }
        }
    }
    */

    fn change_state_to (&mut self, state: AiState) {
        info!("AI: {:?} -> {:?}", self.state, state);
        self.state = state;
    }

    fn update(&mut self, que: &Sender<driver::Event>, state: &State) -> Result<()> {
        use self::AiState::*;
        //info!("AI: {:?}", self.state);
        match self.state {
            WaitForCharList => {
                if let Some(ref charlist) = state.widgets.find_chain(&["ccnt","charlist"]) {
                    let chars = charlist.messages.iter().filter(|&&(ref name,_)|name == "add").count();
                    info!("AI: found {} characters on account", chars);
                    if chars > 0 {
                        if let Some(&(_,ref args)) = charlist.messages.iter().filter(|&&(ref name,_)|name == "add").next() {
                            if let Some(&List::Str(ref name)) = args.get(0) {
                                use driver::Event::User;
                                use driver::UserInput::Message;
                                que.send(User(Message(charlist.id, "play".into(), [List::Str(name.clone())].to_vec())))?;
                                self.change_state_to(WaitForWorld);
                            }
                        }
                    }
                }
            }
            WaitForWorld => {
                /*
                if widgets.widget_exists("mapview") &&
                   state.hero_exists() &&
                   state.hero_grid_exists() &&
                   !state.hero_is_moving() {
                    self.change_state_to(Walking);
                }
                */
            }
            Walking => {
                /*
                match state.hero_xy() {
                    Some(ObjXY(x, y)) => {
                        let (dx, dy) = match self.step {
                            Step::A => (10, 0),
                            Step::B => (0, 10),
                            Step::C => (-10, 0),
                            Step::D => (0, -10),
                            //Step::A => (100, 0),
                            //Step::B => (0, 100),
                            //Step::C => (-100, 0),
                            //Step::D => (0, -100),
                        };
                        if let Step::D = self.step {
                            self.cycle += 1;
                        }
                        self.step = match self.step {
                            Step::A => Step::B,
                            Step::B => Step::C,
                            Step::C => Step::D,
                            Step::D => Step::A,
                        };
                        info!("GO: {} {}, {} {}", x, y, dx, dy);
                        //state.go(x + dx, y + dy).expect("ai walking state.go");
                        self.state = AiState::Walking1;
                    }
                    None => {}
                }
                */
            }
            Walking1 => {
                /*
                if state.hero_is_moving() {
                    self.state = AiState::Walking2;
                }
                */
            }
            Walking2 => {
                /*
                if !state.hero_is_moving() {
                    if self.cycle < 1000 {
                        self.state = AiState::Walking;
                    } else {
                        state.close().expect("ai walking2 state.close");
                        self.state = AiState::WaitForEnd;
                    }
                }
                */
            }
            WaitForEnd => {}
        }
        Ok(())
    }

    /*
    fn exec(&mut self, s: &str) {
        info!("AI: EXEC: {}", s);
    }

    fn init(&mut self) {
        info!("AI: INIT");
    }

    fn new() -> AiDecl {
        Self::new()
    }
    */
}

struct State {
    widgets: Widgets,
}

impl State {
    fn new () -> State {
        State {
            widgets: Widgets::new(),
        }
    }
}

pub fn new (ll_que_tx: Sender<driver::Event>, hl_que_rx: Receiver<state::Event>) {
    use std::thread;
    use std::sync::mpsc::TryRecvError::*;

    thread::Builder::new().name("ai".to_string()).spawn(move || {
        let mut ai = Ai::new();
        let mut state = State::new();
        'outer: loop {
            loop {
                match hl_que_rx.try_recv() {
                    Ok(event) => {
                        use state::Event::*;
                        match event {
                            Tiles(_) => {}
                            Grid(_) => {}
                            Obj(_, _, _) => {}
                            ObjRemove(_) => {}
                            Res(_, _) => {}
                            Hero(_) => {}
                            Wdg(action) => {
                                use state::Wdg::*;
                                match action {
                                    New(id, name, parent_id) => {
                                        state.widgets.add_widget(id, name, parent_id).expect("unable to add_widget");
                                    }
                                    Msg(id, name, args) => {
                                        state.widgets.message(id, (name, args)).expect("unable to add message");
                                    }
                                    Del(id) => {
                                        state.widgets.del_widget(id).expect("unable to del_widget");
                                    }
                                }
                            }
                            Hearthfire(_) => {}
                        }
                    }
                    Err(Empty) => { break; }
                    Err(Disconnected) => { break 'outer; }
                }
            }
            //XXX maybe return Option<Timeout> and if it is Some(timeout) than call
            //ai.update() on Event or on Timeout
            //and if it is None than wait on Event only
            //XXX ??? call ai.update after every Event or after all Events in batch received ???
            ai.update(&ll_que_tx, &state).expect("unable to update AI");
        }
    }).expect("unable to create render thread");
}

/*
pub fn pick(&mut self, obj_id: u32) -> Result<()> {
    info!("PICK");
    let id = self.widget_id("mapview", None).expect("mapview widget is not found");
    let name = "click".to_string();
    let mut args = Vec::new();
    let xy = {
        match self.objects.get(&obj_id) {
            Some(obj) => {
                match obj.xy {
                    Some(xy) => xy.into(),
                    None => panic!("pick(): picking object has no XY"),
                }
            }
            None => panic!("pick(): picking object is not found"),
        }
    };
    args.push(List::Coord((863, 832))); //TODO set some random coords in the center of screen
    args.push(List::Coord(xy));
    args.push(List::Int(3));
    args.push(List::Int(0));
    args.push(List::Int(0));
    args.push(List::Int(obj_id as i32));
    args.push(List::Coord(xy));
    args.push(List::Int(0));
    args.push(List::Int(-1));
    let mut rels = Rels::new(0);
    rels.append(Rel::WDGMSG(WdgMsg::new(id, name, args)));
    self.enqueue_to_send(ClientMessage::REL(rels))?;
    Ok(())
}
*/

/*
pub fn choose_pick(&mut self, wdg_id: u16) -> Result<()> {
    info!("CHOOSE PICK");
    let name = "cl".to_string();
    let mut args = Vec::new();
    args.push(List::Int(0));
    args.push(List::Int(0));
    let mut rels = Rels::new(0);
    rels.append(Rel::WDGMSG(WdgMsg::new(wdg_id, name, args)));
    self.enqueue_to_send(ClientMessage::REL(rels))?;
    Ok(())
}
*/

/*
 CLIENT
  REL  seq=4
   WDGMSG len=65
    id=6 name=click
      COORD : [907, 755]        Coord pc
      COORD : [39683, 36377]    Coord mc
      INT : 1                   int clickb
      INT : 0                   ui.modflags()
      INT : 0                   inf.ol != null
      INT : 325183464           (int)inf.gob.id
      COORD : [39737, 36437]    inf.gob.rc
      INT : 0                   inf.ol.id
      INT : -1                  inf.r.id or -1

 CLIENT
  REL  seq=5
   WDGMSG len=36
    id=6 name=click
      COORD : [1019, 759]        Coord pc
      COORD : [39709, 36386]     Coord mc
      INT : 1                    int clickb
      INT : 0                    ui.modflags()

 private class Click extends Hittest {
     int clickb;

     private Click(Coord c, int b) {
         super(c);
         clickb = b;
     }

     protected void hit(Coord pc, Coord mc, ClickInfo inf) {
         if(inf == null) {
             wdgmsg("click", pc, mc, clickb, ui.modflags());
         } else {
             if(inf.ol == null) {
                 wdgmsg("click", pc, mc, clickb, ui.modflags(), 0, (int)inf.gob.id, inf.gob.rc, 0, getid(inf.r));
             } else {
                 wdgmsg("click", pc, mc, clickb, ui.modflags(), 1, (int)inf.gob.id, inf.gob.rc, inf.ol.id, getid(inf.r));
             }
         }
     }
 }
*/

/*
pub fn hero_exists(&self) -> bool {
    match self.hero.obj {
        Some(_) => true,
        None => false,
    }
}
*/

/*
pub fn hero_grid_exists(&self) -> bool {
    match self.hero_grid_xy() {
        Some(xy) => self.map.grids.contains(&xy),
        None => false,
    }
}
*/

/*
pub fn hero_movement(&self) -> Option<Movement> {
    match self.hero.obj {
        Some(ref hero) => hero.movement,
        None => None,
    }
}
*/

/*
pub fn hero_is_moving(&self) -> bool {
    self.hero_movement().is_some()
}
*/

/*
pub fn widget_exists(&self, typ: &str, name: Option<String>) -> bool {
    match self.widget_id(typ, name) {
        Some(_) => true,
        None => false,
    }
}
*/


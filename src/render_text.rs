use ncurses::*;

fn init () {
//XXX could alternatively use: termion, rustbox, rustty
initscr();

if let None = curs_set(CURSOR_VISIBILITY::CURSOR_INVISIBLE) {
    warn!("set cursor failed");
};
}

pub fn update (/*&mut self,*/ render_tx: &Sender<driver::Event>) -> bool {
thread::spawn(move || {
    let mut counter = 0;
    let mut last_event = "NONE".to_owned();
    loop {
        clear();
        mvprintw(0, 0, &format!("counter: {} ", counter));
        mvprintw(1, 0, &last_event);
        refresh();
        match rx.try_recv() {
            Ok(value) => {
                counter += 1;
                match value {
                    Event::Grid(x, y, _tiles, _z, _ol) => {
                        last_event = format!("GRID: {} {}", x, y);
                    }
                    Event::Obj(id, ObjXY(x, y), _resid) => {
                        last_event = format!("OBJ: {} {} {}", id, x, y);
                    }
                    Event::ObjRemove(_id) => {}
                    Event::Hero(ObjXY(x, y)) => {
                        last_event = format!("HERO: {} {}", x, y);
                    }
                    Event::Input => {
                        //last_event = format!("INPUT");
                        return;
                    }
                    _ => {}
                }
            }
            Err(Empty) => {
                //info!("render: disconnected");
                //return;
            }
            Err(Disconnected) => { break; }
        }
    }
});
}

/*
   let input_tx = caller_tx.clone();
   thread::spawn(move || {
   loop {
   getch();
   match input_tx.send(Event::Input) {
   Ok(()) => {}
   Err(_) => break,
   }
   }
   });
   */

fn end () {
    endwin();
}

use std::sync::mpsc::Sender;
//use std::sync::mpsc::SendError;
use std::thread;
use driver;
use ncurses::*;
use deque::{self, Stolen};
use proto::*;
use state::Wdg;
use errors::*;

#[derive(Debug)]
pub enum Event {
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
        Err("unable to find widget".into())
    }

    fn message (&mut self, msg: String) {
        self.messages.push(msg);
    }
}

struct Ui {
    root: Widget,
}

impl Ui {
    fn new () -> Ui {
        Ui {
            root: Widget::new(0, "root".into())
        }
    }

    fn find_widget (&mut self, id: u16) -> Option<&mut Widget> {
        self.root.find(id)
    }

    fn add_widget (&mut self, id: u16, name: String, parent: u16) -> Result<()> {
        debug!("adding widget {} '{}' [{}]", id, name, parent);
        self.root.find(parent).ok_or::<Error>("unable to find widget".into())?.add(Widget::new(id, name));
        Ok(())
    }

    fn del_widget (&mut self, id: u16) -> Result<()> {
        debug!("deleting widget {}", id);
        self.root.del(id)
    }

    fn message (&mut self, id: u16, msg: String) -> Result<()> {
        debug!("message to widget {} '{}'", id, msg);
        self.root.find(id).ok_or::<Error>("unable to find widget".into())?.message(msg);
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

use std::slice::Iter;
use std::iter::Iterator;

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

//TODO implement as Render trait implementations
pub enum RenderKind {
    No,
    Tui,
    TwoD,
    ThreeD,
}

pub struct Render {
    kind: RenderKind,
    worker: deque::Worker<Event>,
}

impl Drop for Render {
    fn drop(&mut self) {
        match self.kind {
            RenderKind::No => {
            }
            RenderKind::Tui => {
                endwin();
            }
            RenderKind::TwoD => {
            }
            RenderKind::ThreeD => {
            }
        }
    }
}

impl Render {
    pub fn new(kind: RenderKind, render_tx: Sender<driver::Event>) -> Render {

        //TODO try coco::deque instead
        let (worker, stealer) = deque::new();

        match kind {
            RenderKind::No => {
                thread::spawn(move || {
                    loop {
                        match stealer.steal() {
                            Stolen::Data(event) => {
                                match event {
                                    _ => {}
                                }
                            }
                            Stolen::Empty => {}
                            Stolen::Abort => {}
                        }
                    }
                });
            }
            RenderKind::Tui => {
                //XXX could alternatively use: termion, rustbox, rustty
                initscr();

                if let None = curs_set(CURSOR_VISIBILITY::CURSOR_INVISIBLE) {
                    warn!("set cursor failed");
                };

                thread::spawn(move || {
                    let mut counter = 0;
                    let mut last_event = "NONE".to_owned();
                    loop {
                        clear();
                        mvprintw(0, 0, &format!("counter: {} ", counter));
                        mvprintw(1, 0, &last_event);
                        refresh();
                        match stealer.steal() {
                            Stolen::Data(value) => {
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
                            Stolen::Empty => {
                                //info!("render: disconnected");
                                //return;
                            }
                            Stolen::Abort => {}
                        }
                    }
                });

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
            }
            RenderKind::TwoD => {
                thread::spawn(move || {
                    use piston_window::{self as pw, PistonWindow, WindowSettings, Glyphs, TextureSettings, Texture, texture, text, Key, Transformed};
                    //use std::sync::mpsc::TryRecvError;
                    use std::collections::BTreeMap;
                    use image;

                    //let opengl = OpenGL::V3_2;
                    let mut window: PistonWindow =
                        WindowSettings::new("Render", [800, 600])
                            //.opengl(opengl)
                            .vsync(true)
                            .samples(16)
                            .build().unwrap();
                    let font = "/usr/share/fonts/TTF/DejaVuSansMono.ttf";
                    let factory = window.factory.clone();
                    let mut glyphs = Glyphs::new(font, factory, TextureSettings::new()).unwrap();
                    let mut origin = None;
                    let mut objects = BTreeMap::new();
                    let mut zoom = 1.0;
                    let mut dragging = false;
                    let mut hero = ObjXY::new();
                    let mut grids = BTreeMap::new();
                    let mut delta_height = 0;
                    let mut command_line = false;
                    let mut command = ":: ".to_string();
                    let mut show_objtypes = false;
                    let mut resources = BTreeMap::new();
                    let mut highlighted_tiles = 0u8;
                    let mut show_tiles = false;
                    let mut show_widgets = false;
                    let mut widgets = BTreeMap::new();
                    let mut ui = Ui::new();
                    let mut show_borders = false;

                    //TODO const palette
                    //TODO bind palette to resource names
                    let mut palette = [image::Rgba([0,0,0,255]); 256];
                    palette[76] = image::Rgba([ 169, 223, 191, 255]); //76 ("gfx/tiles/field", 35)
                    palette[77] = image::Rgba([ 249, 231, 159, 255]); //77 ("gfx/tiles/beach", 14)
                    palette[87] = image::Rgba([ 88, 214, 141, 255]); //87 ("gfx/tiles/flowermeadow", 135)
                    palette[88] = image::Rgba([ 40, 180, 99, 255]); //88 ("gfx/tiles/grass", 149)
                    palette[102] = image::Rgba([ 20, 90, 50, 255]); //102 ("gfx/tiles/pinebarren", 55)
                    palette[108] = image::Rgba([ 231, 76, 60, 255]); //108 ("gfx/tiles/sombrebramble", 63)
                    palette[111] = image::Rgba([ 25, 111, 61, 255]); //111 ("gfx/tiles/wald", 137)
                    palette[113] = image::Rgba([ 147, 81, 22, 255]); //113 ("gfx/tiles/dirt", 52)
                    palette[115] = image::Rgba([ 21, 67, 96, 255]); //115 ("gfx/tiles/deep", 10)
                    palette[118] = image::Rgba([ 33, 97, 140, 255]); //118 ("gfx/tiles/water", 32)

                    'outer: while let Some(e) = window.next() {
                        match e {
                            pw::Event::Loop(pw::Loop::Update(_)) => {
                                loop {
                                    match stealer.steal() {
                                        Stolen::Data(event) => {
                                            //println!("RENDER: {:?}", event);
                                            match event {
                                                Event::Grid(x,y,tiles,heights,owning) => {
                                                    let mut img = image::ImageBuffer::new(100, 100);
                                                    for y in 0..100 {
                                                        for x in 0..100 {
                                                            let index = tiles[y*100+x] as usize;
                                                            let color =
                                                                if owning[y*100+x] == 0 {
                                                                    palette[index]
                                                                } else {
                                                                    let r: u8 = palette[index][0];
                                                                    let g = palette[index][1];
                                                                    let b = palette[index][2];
                                                                    let a = palette[index][3];
                                                                    image::Rgba([r.saturating_add(50u8),g,b,a])
                                                                };
                                                            img.put_pixel(x as u32, y as u32, color);
                                                        }
                                                    }
                                                    let texture = Texture::from_image(&mut window.factory, &img, &TextureSettings::new().filter(texture::Filter::Nearest)).unwrap();
                                                    grids.insert((x,y), (tiles,heights,owning,texture));
                                                }
                                                Event::Obj(id,xy,resid) => { objects.insert(id, (xy,resid)); }
                                                Event::ObjRemove(ref id) => { objects.remove(id); }
                                                Event::Res(id,name) => { resources.insert(id, name); }
                                                Event::Hero(xy) => {
                                                    if origin.is_none() { origin = Some(xy); }
                                                    hero = xy;
                                                }
                                                Event::Input => break,
                                                Event::Wdg(Wdg::New(id,name,parent)) => {
                                                    widgets.insert(id,(name.clone(),parent));
                                                    ui.add_widget(id,name,parent).expect("unable to ui.add_widget");
                                                }
                                                Event::Wdg(Wdg::Msg(id,name)) => {
                                                    ui.message(id,name).expect("unable to ui.message");
                                                }
                                                Event::Wdg(Wdg::Del(id)) => {
                                                    widgets.remove(&id);
                                                    ui.del_widget(id).expect("unable to ui.del_widget");
                                                }
                                            }
                                        }
                                        Stolen::Empty => break,
                                        Stolen::Abort => {}
                                    }
                                }
                            }
                            pw::Event::Loop(pw::Loop::Render(render)) => {
                                window.draw_2d(&e, |c, g| {
                                    pw::clear([0.0; 4], g);
                                    if let Some(ObjXY(ox,oy)) = origin {

                                        let t = c.transform.trans(render.width as f64 / 2.0, render.height as f64 / 2.0).zoom(zoom);
                                        let t = t.trans(-ox as f64, -oy as f64);

                                        if show_tiles {
                                            let (gx,gy) = hero.grid();
                                            let t = t.zoom(11.0);
                                            for &(gridx,gridy) in [(gx-1,gy-1),(gx,gy-1),(gx+1,gy-1),
                                            (gx-1,gy  ),(gx,gy  ),(gx+1,gy  ),
                                            (gx-1,gy+1),(gx,gy+1),(gx+1,gy+1)].iter() {
                                                if let Some(&(ref _tiles, ref heights, ref _owning, ref texture)) = grids.get(&(gridx,gridy)) {

                                                    let t = t.trans((gridx*100) as f64, (gridy*100) as f64);

                                                    pw::image(texture, t, g);

                                                    if show_borders {
                                                        for y in 0..99 {
                                                            for x in 0..99 {
                                                                use shift_to_unsigned::ShiftToUnsigned;

                                                                let i = y*100+x;
                                                                let z = heights[i].shift_to_unsigned();
                                                                let zx = heights[i+1].shift_to_unsigned();
                                                                let zy = heights[i+100].shift_to_unsigned();
                                                                let dx = if z > zx { z - zx } else { zx - z };
                                                                let dy = if z > zy { z - zy } else { zy - z };
                                                                if dx > delta_height || dy > delta_height {
                                                                    let lx = x as f64;
                                                                    let ly = y as f64;
                                                                    let lcolor = [0.3, 0.3, 0.3, 1.0];
                                                                    let lsize = 0.1;
                                                                    if dx > delta_height {
                                                                        pw::line(lcolor, lsize, [lx, ly, lx + 1.0, ly], t, g);
                                                                    }
                                                                    if dy > delta_height {
                                                                        pw::line(lcolor, lsize, [lx, ly, lx, ly + 1.0], t, g);
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }

                                        for &(ObjXY(x,y),resid) in objects.values() {
                                            let (cx, cy) = (x as f64, y as f64);
                                            #[cfg(feature = "salem")]
                                            //FIXME check res_name=="*claim" but not ID==2951
                                            let color = if resid == 2951 {[1.0, 0.0, 0.0, 1.0]} else {[1.0, 1.0, 1.0, 1.0]};
                                            #[cfg(feature = "hafen")]
                                            let color = [1.0, 1.0, 1.0, 1.0];
                                            pw::rectangle(color, [cx as f64 - 2.0, cy as f64 - 2.0, 4.0, 4.0], t, g);
                                        }

                                        pw::rectangle([0.2, 0.2, 1.0, 1.0], [hero.0 as f64 - 2.0, hero.1 as f64 - 2.0, 4.0, 4.0], t, g);
                                        pw::rectangle([1.0, 1.0, 1.0, 1.0], [hero.0 as f64 - 0.5, hero.1 as f64 - 0.5, 1.0, 1.0], t, g);

                                        if show_objtypes {
                                            let mut objtypes = BTreeMap::new();
                                            for &(_,resid) in objects.values() {
                                                let mut obj = objtypes.entry(resid).or_insert(0);
                                                *obj += 1;
                                            }

                                            let mut i = 0;
                                            for (resid,count) in objtypes.iter() {
                                                let res = if let Some(name) = resources.get(resid) { name } else { "???" };
                                                text::Text::new_color([0.3, 1.0, 0.4, 1.0], 9).draw(
                                                    &format!("{:6} {:6} {}", count, resid, res),
                                                    &mut glyphs,
                                                    &c.draw_state,
                                                    c.transform.trans(200.0, 20.0 + i as f64), g);
                                                i += 9; //TODO += font.height
                                            }
                                        }

                                        if show_widgets {
                                            let mut i = 0;
                                            for (depth,wdg) in ui.widgets_iter() {
                                                text::Text::new_color([1.0, 1.0, 1.0, 1.0], 9).draw(
                                                    &format!("{} {} {}", "- ".repeat(depth), wdg.id, wdg.name),
                                                    &mut glyphs,
                                                    &c.draw_state,
                                                    c.transform.trans(20.0, 20.0 + i as f64), g);
                                                /*
                                                for msg in wdg.messages.iter() {
                                                    text::Text::new_color([0.2, 0.2, 0.2, 1.0], 9).draw(
                                                        &format!("{} {}", "- ".repeat(depth + 1), msg),
                                                        &mut glyphs,
                                                        &c.draw_state,
                                                        c.transform.trans(20.0, 20.0 + i as f64), g);
                                                    i += 9; //TODO += font.height
                                                }
                                                */
                                                i += 9; //TODO += font.height
                                            }
                                            /*
                                            for (id, &(ref name, parent)) in widgets.iter() {
                                                text::Text::new_color([1.0, 1.0, 1.0, 1.0], 9).draw(
                                                    &format!("{:6} {:6} {}", id, parent, name),
                                                    &mut glyphs,
                                                    &c.draw_state,
                                                    c.transform.trans(20.0, 20.0 + i as f64), g);
                                                i += 9; //TODO += font.height
                                            }
                                            */
                                        }

                                        if command_line {
                                            text::Text::new_color([0.3, 1.0, 0.4, 1.0], 12).draw(
                                                &command,
                                                &mut glyphs,
                                                &c.draw_state,
                                                //TODO draw at the bottom of the window
                                                c.transform.trans(10.0, 20.0), g);
                                        }
                                    }
                                });
                            }
                            pw::Event::Input(pw::Input::Button(pw::ButtonArgs{state: pw::ButtonState::Press, button: pw::Button::Mouse(pw::MouseButton::Left), ..})) => dragging = true,
                            pw::Event::Input(pw::Input::Button(pw::ButtonArgs{state: pw::ButtonState::Release, button: pw::Button::Mouse(pw::MouseButton::Left), ..})) => dragging = false,
                            #[cfg(feature = "salem")]
                            pw::Event::Input(pw::Input::Move(pw::Motion::MouseRelative(x,y))) => if dragging {
                                if let Some(ObjXY(ox,oy)) = origin {
                                    origin = Some(ObjXY(ox - (x / zoom) as i32, oy - (y / zoom) as i32));
                                }
                            },
                            #[cfg(feature = "hafen")]
                            pw::Event::Input(pw::Input::Move(pw::Motion::MouseRelative(x,y))) => if dragging {
                                if let Some(ObjXY(ox,oy)) = origin {
                                    origin = Some(ObjXY(ox - (x / zoom), oy - (y / zoom)));
                                }
                            },
                            pw::Event::Input(pw::Input::Move(pw::Motion::MouseScroll(_,y))) => zoom *= if y > 0.0 { 1.05 } else { 0.95 },
                            pw::Event::Input(pw::Input::Button(pw::ButtonArgs{state: pw::ButtonState::Press, button: pw::Button::Keyboard(key), ..})) => {
                                match key {
                                    Key::A => if command_line { command += "a"; },
                                    Key::B => if command_line { command += "b"; } else { show_borders = ! show_borders; },
                                    Key::C => if command_line { command += "c"; },
                                    Key::D => if command_line { command += "d"; },
                                    Key::E => if command_line { command += "e"; },
                                    Key::F => if command_line { command += "f"; },
                                    Key::G => if command_line { command += "g"; },
                                    Key::H => if command_line { command += "h"; },
                                    Key::I => if command_line { command += "i"; },
                                    Key::J => if command_line { command += "j"; },
                                    Key::K => if command_line { command += "k"; },
                                    Key::L => if command_line { command += "l"; },
                                    Key::M => if command_line { command += "m"; },
                                    Key::N => if command_line { command += "n"; },
                                    Key::O => if command_line { command += "o"; } else { show_objtypes = ! show_objtypes; },
                                    Key::P => if command_line { command += "p"; },
                                    Key::Q => if command_line { command += "q"; },
                                    Key::R => if command_line { command += "r"; },
                                    Key::S => if command_line { command += "s"; },
                                    Key::T => if command_line { command += "t"; } else { show_tiles = ! show_tiles; },
                                    Key::U => if command_line { command += "u"; },
                                    Key::V => if command_line { command += "v"; },
                                    Key::W => if command_line { command += "w"; } else { show_widgets = ! show_widgets; },
                                    Key::X => if command_line { command += "x"; },
                                    Key::Y => if command_line { command += "y"; },
                                    Key::Z => if command_line { command += "z"; },
                                    Key::D1 => if command_line { command += "1"; },
                                    Key::D2 => if command_line { command += "2"; },
                                    Key::D3 => if command_line { command += "3"; },
                                    Key::D4 => if command_line { command += "4"; },
                                    Key::D5 => if command_line { command += "5"; },
                                    Key::D6 => if command_line { command += "6"; },
                                    Key::D7 => if command_line { command += "7"; },
                                    Key::D8 => if command_line { command += "8"; },
                                    Key::D9 => if command_line { command += "9"; } else { highlighted_tiles = highlighted_tiles.wrapping_sub(1); },
                                    Key::D0 => if command_line { command += "0"; } else { highlighted_tiles = highlighted_tiles.wrapping_add(1); },
                                    Key::Space => if command_line { command += " "; },
                                    Key::PageUp => delta_height += 1,
                                    Key::PageDown => if delta_height > 0 { delta_height -= 1; },
                                    Key::Up    => { render_tx.send(driver::Event::Render(driver::RenderEvent::Up)).expect("unable to send Render::Up"); }
                                    Key::Down  => { render_tx.send(driver::Event::Render(driver::RenderEvent::Down)).expect("unable to send Render::Down"); }
                                    Key::Left  => { render_tx.send(driver::Event::Render(driver::RenderEvent::Left)).expect("unable to send Render::Left"); }
                                    Key::Right => { render_tx.send(driver::Event::Render(driver::RenderEvent::Right)).expect("unable to send Render::Right"); }
                                    //Key::PageUp => zoom *= 1.2,
                                    //Key::PageDown => zoom *= 0.8,
                                    Key::Return => {
                                        if command_line {
                                            //TODO execute command
                                        }
                                        command_line = !command_line;
                                        command = ":: ".to_string();
                                    }
                                    Key::Escape => if command_line { command_line = false; } else { break; },
                                    _ => {}
                                }
                            }
                            pw::Event::Input(pw::Input::Close(_)) => break,
                            _ => {}
                        }
                    }
                    render_tx.send(driver::Event::Render(driver::RenderEvent::Quit)).expect("unable to send Render::Quit");
                });
            }
            RenderKind::ThreeD => {
                // 3D UI
                /*
                         thread::spawn(move || {
                                 use ::glium::DisplayBuild;
                                 use ::glium::Surface;
                                 use ::glium::glutin::WindowBuilder;
                                 use ::glium::index::NoIndices;
                                 use ::glium::VertexBuffer;
                                 use ::glium::index::PrimitiveType;
                                 use ::glium::glutin;
                                 use ::glium::Program;
                                 //use ::glium::uniforms::EmptyUniforms;
                                 use ::glium::draw_parameters::DrawParameters;
                                 //use ::glium::texture::/*TODO Compressed*/Texture2d;
                                 //use ::glium::texture::Texture2dArray;
                                 use ::glium::DepthTest;
                                 use std::sync::mpsc::TryRecvError;
                                 use cgmath;
                                 //use cgmath::Matrix;
                                 use cgmath::FixedArray;
                                 use camera_controllers as camera;

                                 let display = WindowBuilder::new()
                                         .with_dimensions(1024, 1024)
                                         .with_title(format!("render"))
                                         .with_depth_buffer(24)
                                         .build_glium()
                                         .unwrap();

                                 #[derive(Copy, Clone)]
                                 struct Vertex {
                                     v_pos: [f32; 3],
                                     v_col: [f32; 3],
                                 }

                                 implement_vertex!(Vertex, v_pos, v_col);

                                 let mut vertex_buffer: VertexBuffer<Vertex> = VertexBuffer::empty(&display, 0).unwrap();
                                 let indices = NoIndices(PrimitiveType::TrianglesList);

                                 let vertex_shader_src = r#"
                                     #version 140
                                     in vec3 v_pos;
                                     in vec3 v_col;
                                     flat out vec3 vv_col;

                                     uniform mat4 u_model;
                                     uniform mat4 u_view;
                                     uniform mat4 u_proj;

                                     void main() {
                                         vv_col = v_col;
                                         gl_Position = u_proj * u_view * u_model * vec4(v_pos, 1.0);
                                     }
                                 "#;

                                 let fragment_shader_src = r#"
                                     #version 140
                                     flat in vec3 vv_col;
                                     out vec4 color;
                                     void main() {
                                         //float c = float(vv_col) / 255.0;
                                         color = vec4(vv_col, 1.0);
                                     }
                                 "#;

                                 //FIXME don't do init here. move it to Render struct new()
                                 let program = match Program::from_source(&display, vertex_shader_src, fragment_shader_src, None) {
                                     Ok(program) => program,
                                     Err(error) => {
                                         info!("compile program ERROR: {:?}", error);
                                         return;
                                     }
                                 };

                                 let mut landscape = Vec::new();

                                 {
                                     let col = [1.0; 3];
                                     landscape.extend(&[Vertex{v_pos: [-300.0, 0.0,-300.0], v_col: col},
                                                        Vertex{v_pos: [-300.0, 0.0, 300.0], v_col: col},
                                                        Vertex{v_pos: [ 300.0, 0.0, 300.0], v_col: col},
                                                        Vertex{v_pos: [-300.0, 0.0,-300.0], v_col: col},
                                                        Vertex{v_pos: [ 300.0, 0.0, 300.0], v_col: col},
                                                        Vertex{v_pos: [ 300.0, 0.0,-300.0], v_col: col}]);
                                 }

                                 let mut grids_count = 0;

                                 //let mut camera_x = 1.0;
                                 //let mut camera_y = 1.0;
                                 //let mut camera_z = 1.0;

                                 let mut dragging = false;
                                 let mut dragging_xy = None;
                                 let mut zooming = false;
                                 let mut zooming_xy = None;

                                 let mut camera = camera::OrbitZoomCamera::new([0.0, 0.0, 0.0], camera::OrbitZoomCameraSettings::default().pitch_speed(1.0).orbit_speed(0.004));
                                 camera.distance = 5.0;

                                 let model_scale = 0.001;

                                 /*'ecto_loop:*/ loop {
                                     {
                                         let mut target = display.draw();
                                         target.clear_color_and_depth((0.1, 0.1, 0.1, 1.0), 1.0);

                                         let draw_params = DrawParameters {
                                             depth_test: DepthTest::IfLess,
                                             depth_write: true,
                                             //polygon_mode: ::glium::draw_parameters::PolygonMode::Line,
                                             .. Default::default()
                                         };

                                         /*
                                         let view: cgmath::AffineMatrix3<f32> = cgmath::Transform::look_at(
                                             &cgmath::Point3::new(camera_x, camera_y, camera_z),
                                             &cgmath::Point3::new(0.0, 0.0, 0.0),
                                             &cgmath::Vector3::unit_z(),
                                         );
                                         */

                                         let uniforms = uniform! {
                                             //u_model: cgmath::Matrix4::identity().into_fixed(),
                                             u_model: cgmath::Matrix4::new(model_scale, 0.0, 0.0, 0.0,
                                                                           0.0, model_scale, 0.0, 0.0,
                                                                           0.0, 0.0, model_scale, 0.0,
                                                                           0.0, 0.0, 0.0, 1.0).into_fixed(),
                                             //u_view: view.mat.into_fixed(),
                                             u_view: camera.camera(0.0).orthogonal(),
                                             u_proj: cgmath::perspective(cgmath::deg(60.0f32), 1.0/*stream.get_aspect_ratio()*/, 0.1, 1000.0).into_fixed(),
                                         };

                                         if let Err(e) = target.draw(&vertex_buffer, &indices, &program, &uniforms/*EmptyUniforms*/, &draw_params) {
                                             info!("target.draw ERROR: {:?}", e);
                                             return;
                                         }
                                         if let Err(e) = target.finish() {
                                             info!("target.finish ERROR: {:?}", e);
                                             return;
                                         }
                                     }

                                     for ev in display.poll_events() {
                                         match ev {
                                             glutin::Event::KeyboardInput(_, _, Some(glutin::VirtualKeyCode::Escape)) |
                                             glutin::Event::Closed => {
                                                 /*break 'ecto_loop;*/
                                                 return;
                                             }
                                             glutin::Event::MouseInput(glutin::ElementState::Pressed, glutin::MouseButton::Left) => {
                                                 dragging = true;
                                             }
                                             glutin::Event::MouseInput(glutin::ElementState::Released, glutin::MouseButton::Left) => {
                                                 dragging = false;
                                                 dragging_xy = None;
                                             }
                                             glutin::Event::MouseInput(glutin::ElementState::Pressed, glutin::MouseButton::Right) => {
                                                 zooming = true;
                                             }
                                             glutin::Event::MouseInput(glutin::ElementState::Released, glutin::MouseButton::Right) => {
                                                 zooming = false;
                                                 zooming_xy = None;
                                             }
                                             glutin::Event::MouseMoved((x,y)) => {
                                                 if dragging {
                                                     dragging_xy = match dragging_xy {
                                                         None => Some((x,y)),
                                                         Some((mx,my)) => {
                                                             //camera_x += ((x - mx) as f32) / 1000.0;
                                                             //camera_z += ((y - my) as f32) / 1000.0;
                                                             //camera.control_camera((x - mx) as f32, (y - my) as f32);
                                                             /*FIXME*/ //camera.control_camera(-(x - mx) as f32, -(y - my) as f32);
                                                             Some((x,y))
                                                         }
                                                     }
                                                 }
                                                 if zooming {
                                                     zooming_xy = match zooming_xy {
                                                         None => Some((x,y)),
                                                         Some((_mx,_my)) => {
                                                             //let dy = y - my;
                                                             //let factor = 1.0 + (dy as f32) / 100.0;
                                                             //camera_x *= factor;
                                                             //camera_y *= factor;
                                                             //camera_z *= factor;
                                                             Some((x,y))
                                                         }
                                                     }
                                                 }
                                             }
                                             _ => ()
                                         }
                                     }

                                     //loop {
                                         match rx.try_recv() {
                                             Ok(value) => {
                                                 match value {
                                                     Event::Grid(gridx,gridy,tiles,z) => {
                                                         //info!("render: received Grid ({},{})", gridx, gridy);
                                                         if grids_count == 0 {
                                                             camera.target = [0.0, z[0] as f32 * model_scale, 0.0];
                                                         }

                                                         let mut vertices = Vec::with_capacity(10_000);
                                                         for y in 0..100 {
                                                             for x in 0..100 {
                                                                 let index = y*100+x;
                                                                 let vx = (gridx + x as i32 * 11) as f32;
                                                                 let vy = (gridy + y as i32 * 11) as f32;
                                                                 let vz = z[index] as f32 * 4.0;
                                                                 //vertices.push([vx,vy,vz]);
                                                                 vertices.push([vx,vz,vy]);
                                                             }
                                                         }
                                                         let mut shape = Vec::with_capacity(60_000);
                                                         for y in 0..99 {
                                                             for x in 0..99 {
                                                                 let index = y*100+x;
                                                                 let color = [tiles[index] as f32 / 255.0; 3];
                                                                 shape.push(Vertex{v_pos: vertices[index+100], v_col: color});
                                                                 shape.push(Vertex{v_pos: vertices[index], v_col: color});
                                                                 shape.push(Vertex{v_pos: vertices[index+1], v_col: color});

                                                                 shape.push(Vertex{v_pos: vertices[index+100], v_col: color});
                                                                 shape.push(Vertex{v_pos: vertices[index+1], v_col: color});
                                                                 shape.push(Vertex{v_pos: vertices[index+101], v_col: color});
                                                             }
                                                         }
                                                         landscape.extend(&shape);
                                                         info!("render: vertices {}, faces {}, quads {}", landscape.len(), landscape.len()/3, landscape.len()/6);
                                                         vertex_buffer = VertexBuffer::new(&display, &landscape).unwrap();
                                                         grids_count += 1;
                                                     }
                                                     Event::Obj(x,y) => {
                                                         //info!("render: received Obj ({},{})", x, y);
                                                         let x = x as f32;
                                                         let y = y as f32;

                                                         let mut vertices = Vec::with_capacity(4);
                                                         vertices.push([x-3.0, 0.01, y-3.0]);
                                                         vertices.push([x-3.0, 0.01, y+3.0]);
                                                         vertices.push([x+3.0, 0.01, y+3.0]);
                                                         vertices.push([x+3.0, 0.01, y-3.0]);

                                                         let col = [0.0, 1.0, 0.0];
                                                         let mut mesh = Vec::with_capacity(6);
                                                         mesh.push(Vertex{v_pos: vertices[0], v_col: col});
                                                         mesh.push(Vertex{v_pos: vertices[1], v_col: col});
                                                         mesh.push(Vertex{v_pos: vertices[2], v_col: col});

                                                         mesh.push(Vertex{v_pos: vertices[2], v_col: col});
                                                         mesh.push(Vertex{v_pos: vertices[3], v_col: col});
                                                         mesh.push(Vertex{v_pos: vertices[0], v_col: col});

                                                         landscape.extend(&mesh);
                                                         vertex_buffer = VertexBuffer::new(&display, &landscape).unwrap();
                                                     }
                                                 }
                                             }
                                             Err(e) => {
                                                 if let TryRecvError::Disconnected = e {
                                                     info!("render: disconnected");
                                                     //break/* 'ecto_loop*/;
                                                     return;
                                                 }// else {
                                                 //    break;
                                                 //}
                                             }
                                         }
                                     //}

                                 }
                         });
                */
            }
        }

        Render {
            kind: kind,
            worker: worker,
        }
    }

    pub fn update(&mut self, event: Event) {
        self.worker.push(event)
    }
}

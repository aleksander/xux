use render::{Event, Ui};
use proto::{ObjXY, ObjID, ResID};
use state::Wdg;
use driver;
use piston_window::{self as pw, PistonWindow, WindowSettings, Glyphs, TextureSettings, Texture, G2dTexture, text, Key, Transformed};
use piston_window::texture::Filter::Nearest;
//use piston_window::OpenGL;
//use std::sync::mpsc::TryRecvError;
use std::collections::BTreeMap;
use image;
use std::sync::mpsc::Sender;

pub struct RenderImpl {
    //TODO const palette
    //TODO bind palette to resource names
    palette: [image::Rgba<u8>; 256],
    window: PistonWindow,
    grids: BTreeMap<(i32,i32), (Vec<u8>,Vec<i16>,Vec<u8>,G2dTexture)>,
    objects: BTreeMap<ObjID, (ObjXY,ResID)>,
    origin: Option<ObjXY>,
            zoom: f64,
            dragging: bool,
            hero: ObjXY,
            delta_height: u16,
            command_line: bool,
            command: String,
            show_objtypes: bool,
            resources: BTreeMap<ResID, String>,
            highlighted_tiles: u8,
            show_tiles: bool,
            show_widgets: bool,
            widgets: BTreeMap<u16, (String, u16)>,
            ui: Ui,
            show_borders: bool,
            glyphs: Glyphs,
}

impl RenderImpl {
    pub fn new () -> RenderImpl {
        let window: PistonWindow = WindowSettings::new("Render", [800, 600])
                //.opengl(opengl)
                //.vsync(true)
                //.samples(16)
                .srgb(false)
                .build().unwrap();
        //let opengl = OpenGL::V3_0;
        let font = "DejaVuSansMono.ttf";
        let factory: pw::GfxFactory = window.factory.clone();
        let glyphs = Glyphs::new(font, factory, TextureSettings::new()).unwrap();
        RenderImpl {
            palette: [image::Rgba([0,0,0,255]); 256],
            window: window,
            grids: BTreeMap::new(),
            objects: BTreeMap::new(),
            origin: None,
            zoom: 1.0,
            dragging: false,
            hero: ObjXY::new(),
            delta_height: 0,
            command_line: false,
            command: ":: ".to_string(),
            show_objtypes: false,
            resources: BTreeMap::new(),
            highlighted_tiles: 0u8,
            show_tiles: false,
            show_widgets: false,
            widgets: BTreeMap::new(),
            ui: Ui::new(),
            show_borders: false,
            glyphs: glyphs,
        }
    }

    pub fn init (&mut self) {
        self.palette[76] = image::Rgba([ 169, 223, 191, 255]); //76 ("gfx/tiles/field", 35)
        self.palette[77] = image::Rgba([ 249, 231, 159, 255]); //77 ("gfx/tiles/beach", 14)
        self.palette[87] = image::Rgba([ 88, 214, 141, 255]); //87 ("gfx/tiles/flowermeadow", 135)
        self.palette[88] = image::Rgba([ 40, 180, 99, 255]); //88 ("gfx/tiles/grass", 149)
        self.palette[102] = image::Rgba([ 20, 90, 50, 255]); //102 ("gfx/tiles/pinebarren", 55)
        self.palette[108] = image::Rgba([ 231, 76, 60, 255]); //108 ("gfx/tiles/sombrebramble", 63)
        self.palette[111] = image::Rgba([ 25, 111, 61, 255]); //111 ("gfx/tiles/wald", 137)
        self.palette[113] = image::Rgba([ 147, 81, 22, 255]); //113 ("gfx/tiles/dirt", 52)
        self.palette[115] = image::Rgba([ 21, 67, 96, 255]); //115 ("gfx/tiles/deep", 10)
        self.palette[118] = image::Rgba([ 33, 97, 140, 255]); //118 ("gfx/tiles/water", 32)
    }

    pub fn event (&mut self, event: Event) {
        //println!("RENDER: {:?}", event);
        match event {
            Event::Grid(x,y,tiles,heights,owning) => {
                let mut img = image::ImageBuffer::new(100, 100);
                for y in 0..100 {
                    for x in 0..100 {
                        let index = tiles[y*100+x] as usize;
                        let color =
                            if owning[y*100+x] == 0 {
                                self.palette[index]
                            } else {
                                let r: u8 = self.palette[index][0];
                                let g = self.palette[index][1];
                                let b = self.palette[index][2];
                                let a = self.palette[index][3];
                                image::Rgba([r.saturating_add(50u8),g,b,a])
                            };
                        img.put_pixel(x as u32, y as u32, color);
                    }
                }
                let texture = Texture::from_image(&mut self.window.factory, &img, &TextureSettings::new().filter(Nearest)).unwrap();
                self.grids.insert((x,y), (tiles,heights,owning,texture));
            }
            Event::Obj(id,xy,resid) => {
                //TODO ??? separate static objects like trees and
                //dynamic objects like rabbits to two
                //different caches
                self.objects.insert(id, (xy,resid));
            }
            Event::ObjRemove(ref id) => { self.objects.remove(id); }
            Event::Res(id,name) => { self.resources.insert(id, name); }
            Event::Hero(xy) => {
                if self.origin.is_none() { self.origin = Some(xy); }
                self.hero = xy;
            }
            Event::Input => { /*TODO*/ }
            Event::Wdg(Wdg::New(id,name,parent)) => {
                self.widgets.insert(id,(name.clone(),parent));
                self.ui.add_widget(id,name,parent).expect("unable to ui.add_widget");
            }
            Event::Wdg(Wdg::Msg(id,name)) => {
                self.ui.message(id,name).expect("unable to ui.message");
            }
            Event::Wdg(Wdg::Del(id)) => {
                self.widgets.remove(&id);
                self.ui.del_widget(id).expect("unable to ui.del_widget");
            }
            Event::Tiles(_tiles) => { /*TODO*/ }
        }
    }

    pub fn update (&mut self, render_tx: &Sender<driver::Event>) -> bool {
        while let Some(e) = self.window.next() {
            match e {
                pw::Event::Loop(pw::Loop::Update(_)) => {
                }
                pw::Event::Loop(pw::Loop::Render(render)) => {

                    //self.window.draw_2d(&e,

                    //);

                    use piston_window::RenderEvent;
                    use piston_window::OpenGLWindow;
                    use gfx_graphics;

                    if let Some(args) = e.render_args() {
                        self.window.window.make_current();

                        {
                        let ref mut g = gfx_graphics::GfxGraphics::new(
                            &mut self.window.encoder,
                            &self.window.output_color,
                            &self.window.output_stencil,
                            &mut self.window.g2d
                            );
                        let c = pw::Context::new_viewport(args.viewport());

                        pw::clear([0.0; 4], g);
                        if let Some(ObjXY(ox,oy)) = self.origin {

                            let t = c.transform.trans(render.width as f64 / 2.0, render.height as f64 / 2.0).zoom(self.zoom);
                            let t = t.trans(-ox as f64, -oy as f64);

                            if self.show_tiles {
                                let (gx,gy) = self.hero.grid();
                                let t = t.zoom(11.0);
                                for &(gridx,gridy) in [(gx-1,gy-1),(gx,gy-1),(gx+1,gy-1),
                                (gx-1,gy  ),(gx,gy  ),(gx+1,gy  ),
                                (gx-1,gy+1),(gx,gy+1),(gx+1,gy+1)].iter() {
                                    if let Some(&(ref _tiles, ref heights, ref _owning, ref texture)) = self.grids.get(&(gridx,gridy)) {

                                        let t = t.trans((gridx*100) as f64, (gridy*100) as f64);

                                        pw::image(texture, t, g);

                                        if self.show_borders {
                                            for y in 0..99 {
                                                for x in 0..99 {
                                                    use shift_to_unsigned::ShiftToUnsigned;

                                                    let i = y*100+x;
                                                    let z = heights[i].shift_to_unsigned();
                                                    let zx = heights[i+1].shift_to_unsigned();
                                                    let zy = heights[i+100].shift_to_unsigned();
                                                    let dx = if z > zx { z - zx } else { zx - z };
                                                    let dy = if z > zy { z - zy } else { zy - z };
                                                    if dx > self.delta_height || dy > self.delta_height {
                                                        let lx = x as f64;
                                                        let ly = y as f64;
                                                        let lcolor = [0.3, 0.3, 0.3, 1.0];
                                                        let lsize = 0.1;
                                                        if dx > self.delta_height {
                                                            pw::line(lcolor, lsize, [lx, ly, lx + 1.0, ly], t, g);
                                                        }
                                                        if dy > self.delta_height {
                                                            pw::line(lcolor, lsize, [lx, ly, lx, ly + 1.0], t, g);
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }

                            for &(ObjXY(x,y),_resid) in self.objects.values() {
                                let (cx, cy) = (x as f64, y as f64);
                                let color = [1.0, 1.0, 1.0, 1.0];
                                pw::rectangle(color, [cx as f64 - 2.0, cy as f64 - 2.0, 4.0, 4.0], t, g);
                            }

                            pw::rectangle([0.2, 0.2, 1.0, 1.0], [self.hero.0 as f64 - 2.0, self.hero.1 as f64 - 2.0, 4.0, 4.0], t, g);
                            pw::rectangle([1.0, 1.0, 1.0, 1.0], [self.hero.0 as f64 - 0.5, self.hero.1 as f64 - 0.5, 1.0, 1.0], t, g);

                            if self.show_objtypes {
                                let mut objtypes = BTreeMap::new();
                                for &(_,resid) in self.objects.values() {
                                    let obj = objtypes.entry(resid).or_insert(0);
                                    *obj += 1;
                                }

                                let mut i = 0;
                                for (resid,count) in objtypes.iter() {
                                    let res = if let Some(name) = self.resources.get(resid) { name } else { "???" };
                                    text::Text::new_color([0.3, 1.0, 0.4, 1.0], 9).draw(
                                        &format!("{:6} {:6} {}", count, resid, res),
                                        &mut self.glyphs,
                                        &c.draw_state,
                                        c.transform.trans(200.0, 20.0 + i as f64), g).expect("unable to draw text");
                                    i += 9; //TODO += font.height
                                }
                            }

                            if self.show_widgets {
                                let mut i = 0;
                                for (depth,wdg) in self.ui.widgets_iter() {
                                    text::Text::new_color([1.0, 1.0, 1.0, 1.0], 9).draw(
                                        &format!("{} {} {}", "- ".repeat(depth), wdg.id, wdg.name),
                                        &mut self.glyphs,
                                        &c.draw_state,
                                        c.transform.trans(20.0, 20.0 + i as f64), g).expect("unable to draw text");
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

                            if self.command_line {
                                text::Text::new_color([0.3, 1.0, 0.4, 1.0], 12).draw(
                                    &self.command,
                                    &mut self.glyphs,
                                    &c.draw_state,
                                    //TODO draw at the bottom of the window
                                    c.transform.trans(10.0, 20.0), g).expect("unable to draw text");
                            }
                        }
                        }
                        /*
                        if self.window.g2d.colored_offset > 0 {
                            g.flush_colored();
                        }
                        */
                        self.window.encoder.flush(&mut self.window.device);
                    }

                }
                pw::Event::Input(pw::Input::Button(pw::ButtonArgs{state: pw::ButtonState::Press, button: pw::Button::Mouse(pw::MouseButton::Left), ..})) => self.dragging = true,
                pw::Event::Input(pw::Input::Button(pw::ButtonArgs{state: pw::ButtonState::Release, button: pw::Button::Mouse(pw::MouseButton::Left), ..})) => self.dragging = false,
                pw::Event::Input(pw::Input::Move(pw::Motion::MouseRelative(x,y))) => if self.dragging {
                    if let Some(ObjXY(ox,oy)) = self.origin {
                        self.origin = Some(ObjXY(ox - (x / self.zoom), oy - (y / self.zoom)));
                    }
                },
                pw::Event::Input(pw::Input::Move(pw::Motion::MouseScroll(_,y))) => self.zoom *= if y > 0.0 { 1.05 } else { 0.95 },
                pw::Event::Input(pw::Input::Button(pw::ButtonArgs{state: pw::ButtonState::Press, button: pw::Button::Keyboard(key), ..})) => {
                    match key {
                        Key::A => if self.command_line { self.command += "a"; },
                        Key::B => if self.command_line { self.command += "b"; } else { self.show_borders = ! self.show_borders; },
                        Key::C => if self.command_line { self.command += "c"; },
                        Key::D => if self.command_line { self.command += "d"; },
                        Key::E => if self.command_line { self.command += "e"; },
                        Key::F => if self.command_line { self.command += "f"; },
                        Key::G => if self.command_line { self.command += "g"; },
                        Key::H => if self.command_line { self.command += "h"; },
                        Key::I => if self.command_line { self.command += "i"; },
                        Key::J => if self.command_line { self.command += "j"; },
                        Key::K => if self.command_line { self.command += "k"; },
                        Key::L => if self.command_line { self.command += "l"; },
                        Key::M => if self.command_line { self.command += "m"; },
                        Key::N => if self.command_line { self.command += "n"; },
                        Key::O => if self.command_line { self.command += "o"; } else { self.show_objtypes = ! self.show_objtypes; },
                        Key::P => if self.command_line { self.command += "p"; },
                        Key::Q => if self.command_line { self.command += "q"; },
                        Key::R => if self.command_line { self.command += "r"; },
                        Key::S => if self.command_line { self.command += "s"; },
                        Key::T => if self.command_line { self.command += "t"; } else { self.show_tiles = ! self.show_tiles; },
                        Key::U => if self.command_line { self.command += "u"; },
                        Key::V => if self.command_line { self.command += "v"; },
                        Key::W => if self.command_line { self.command += "w"; } else { self.show_widgets = ! self.show_widgets; },
                        Key::X => if self.command_line { self.command += "x"; },
                        Key::Y => if self.command_line { self.command += "y"; },
                        Key::Z => if self.command_line { self.command += "z"; },
                        Key::D1 => if self.command_line { self.command += "1"; },
                        Key::D2 => if self.command_line { self.command += "2"; },
                        Key::D3 => if self.command_line { self.command += "3"; },
                        Key::D4 => if self.command_line { self.command += "4"; },
                        Key::D5 => if self.command_line { self.command += "5"; },
                        Key::D6 => if self.command_line { self.command += "6"; },
                        Key::D7 => if self.command_line { self.command += "7"; },
                        Key::D8 => if self.command_line { self.command += "8"; },
                        Key::D9 => if self.command_line { self.command += "9"; } else { self.highlighted_tiles = self.highlighted_tiles.wrapping_sub(1); },
                        Key::D0 => if self.command_line { self.command += "0"; } else { self.highlighted_tiles = self.highlighted_tiles.wrapping_add(1); },
                        Key::Space => if self.command_line { self.command += " "; },
                        Key::PageUp => self.delta_height += 1,
                        Key::PageDown => if self.delta_height > 0 { self.delta_height -= 1; },
                        Key::Up    => { render_tx.send(driver::Event::Render(driver::RenderEvent::Up)).expect("unable to send Render::Up"); }
                        Key::Down  => { render_tx.send(driver::Event::Render(driver::RenderEvent::Down)).expect("unable to send Render::Down"); }
                        Key::Left  => { render_tx.send(driver::Event::Render(driver::RenderEvent::Left)).expect("unable to send Render::Left"); }
                        Key::Right => { render_tx.send(driver::Event::Render(driver::RenderEvent::Right)).expect("unable to send Render::Right"); }
                        //Key::PageUp => zoom *= 1.2,
                        //Key::PageDown => zoom *= 0.8,
                        Key::Return => {
                            if self.command_line {
                                //TODO execute command
                            }
                            self.command_line = !self.command_line;
                            self.command = ":: ".to_string();
                        }
                        Key::Escape => if self.command_line { self.command_line = false; } else { return false; },
                        _ => {}
                    }
                }
                pw::Event::Input(pw::Input::Close(_)) => { return false; }
                _ => {}
            }
        }
        //render_tx.send(driver::Event::Render(driver::RenderEvent::Quit)).expect("unable to send Render::Quit");

        true
    }

    pub fn end (self) {}
}

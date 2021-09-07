#![feature(stmt_expr_attributes)]
#![feature(destructuring_assignment)]

mod behavior_trees;

use std::{
    collections::BTreeMap,
    default::Default,
    fs::File,
    io::BufReader,
    sync::mpsc::{Receiver, Sender, TryRecvError::*},
};

use anyhow::anyhow;
use log::{debug, error, info, warn};
use macroquad::prelude::{
    BLACK, BLUE, Camera2D, clear_background, Color, DrawTextureParams, FilterMode, is_mouse_button_down, is_mouse_button_pressed, is_quit_requested, mouse_position, mouse_wheel, MouseButton,
    next_frame, prevent_quit, Rect, screen_height, screen_width, set_camera, vec2, Vec2, WHITE,
};
use ron::de::from_reader;

use xux::{
    client, driver,
    proto::{ObjID, ObjXY, ResID},
    Result, state,
    state::{Wdg, Surface},
};
use egui::Pos2;
use xux::widgets::Widgets;
use behavior_tree::Node;
use crate::behavior_trees::root;
use std::cell::RefCell;
use std::rc::Rc;

#[macroquad::main("2d-macroquad-egui")]
async fn main() -> Result<()> {
    env_logger::builder().format_target(true).format_module_path(true).format_level(true).format_timestamp(None).init();

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

    //TODO take all authorisation information from the GUI (maybe cache values in .config after the user first time enters them)

    let (login, cookie) = client::authorize(host, auth_port, username, password)?;
    let (ll_event_tx, hl_event_rx) = client::run_threaded(host, game_port, login, cookie)?;
    let mut render_ctx = RenderContext::new(ll_event_tx, hl_event_rx);

    prevent_quit();
    loop {
        clear_background(BLACK);
        render_ctx.update();

        //TODO signal handling

        if render_ctx.should_exit {
            break;
        }
        render_ctx.draw();
        next_frame().await
    }
    info!("render thread: done");
    Ok(())
}

pub struct XuxState {
    event_tx: Sender<driver::Event>,
    event_rx: Receiver<state::Event>,
    widgets: xux::widgets::Widgets,
    resources: BTreeMap<ResID, String>,
    objects: BTreeMap<ObjID, ((ObjXY, f64), ResID)>,
    hero: (ObjXY, f64),
    login: String,
    name: String,
    timestamp: String,
}

struct RenderContext {
    xux_state: Rc<RefCell<XuxState>>,
    behavior_tree: Box<dyn Node>,
    //TODO
    //struct RenderState {
    tile_colors: BTreeMap<String, [u8; 4]>,
    palette: [[u8; 4]; 256],
    grids_tiles: Vec<(i32, i32, macroquad::texture::Texture2D)>, // (x,y,tiles)
    grids_heights: Vec<(i32, i32, Vec<f32>)>, // (x,y,heights)
    // }
    //TODO
    //struct RenderConfig {
    show_tiles: bool,
    show_heights: bool,
    show_owning: bool,
    // }
    should_exit: bool,
    target: Vec2,
    target_isnt_set: bool,
    zoom: f32,
    mouse: Vec2,
    camera: Camera2D,
    marks: Vec<Vec2>,
    height_threshold: f32,
}

impl RenderContext {
    fn new(event_tx: Sender<driver::Event>, event_rx: Receiver<state::Event>) -> RenderContext {
        //#[cfg(feature = "dump_events")]
        //let mut dumper = dumper::Dumper::init().expect("unable to create dumper");

        let xux_state = Rc::new(RefCell::new(XuxState {
            event_tx: event_tx,
            event_rx: event_rx,
            widgets: Widgets::new(),
            resources: BTreeMap::new(),
            objects: BTreeMap::new(),
            hero: (ObjXY(0.0, 0.0), 0.0),
            login: "nobody".into(), //FIXME set to login on login
            name: "noone".into(),   //FIXME set to hero name on hero choise
            timestamp: chrono::Local::now().format("%Y-%m-%d %H-%M-%S").to_string(),
        }));

        let behavior_tree = Box::new(root(xux_state.clone()));

        RenderContext {
            xux_state,
            behavior_tree,
            tile_colors: {
                let f = File::open("tile_colors.ron").expect("unable to open tile_colors.ron");
                from_reader(BufReader::new(f)).expect("unable to deserialize")
            },
            palette: [[0, 0, 250, 255]; 256],
            grids_tiles: Vec::new(),
            grids_heights: Vec::new(),
            show_tiles: true,
            show_heights: true,
            show_owning: true,
            should_exit: false,
            target: vec2(0.0, 0.0),
            target_isnt_set: true,
            zoom: 1.0,
            mouse: vec2(0.0, 0.0),
            camera: Camera2D::default(),
            marks: Vec::new(),
            height_threshold: 20.0,
        }
    }

    fn tiles_to_png(&self, surface: &Surface) -> Result<()> {
        match surface.tiles() {
            Some(ref tiles) => {
                //FIXME remove expect(), propagate error up
                util::tiles_to_png(&self.xux_state.borrow().login, &self.xux_state.borrow().name, &self.xux_state.borrow().timestamp, surface.x(), surface.y(), tiles, &self.palette /* TODO &s.z */).expect("Unable to save tiles");
                Ok(())
            }
            None => {
                warn!("surface is tileless");
                Ok(())
            }
        }
    }

    fn event(&mut self, event: state::Event) {
        /* TODO app.event(event) */
        match event {
            /*state::Event::Tiles(tiles) => {
                debug!("RENDER: tiles");
            }*/
            state::Event::Surface(ref surface) => {
                debug!("RENDER: surface v{} ({}, {})", surface.version(), surface.x(), surface.y());
                if let Some(tileres) = surface.tileres() {
                    for tile in tileres {
                        debug!("RENDER: tile {} {}", tile.id, tile.name);
                        if let Some(color) = self.tile_colors.get(&tile.name) {
                            self.palette[tile.id as usize] = *color;
                        } else {
                            warn!("RENDER: tile '{}' not found in 'tile_colors.ron'", tile.name);
                        }
                    }
                }
                self.tiles_to_png(surface).expect("Unable to save tiles");
                //TODO app.rebuild_grid_cache(...)
                //XXX FIXME TODO one BIG mesh with all grids in it ?
                //or individual buffer+pipe.data for every grid ?
                //or use texture-per-grid and don't care at all
                if let Some(tiles) = surface.tiles() {
                    let mut texture_data = Vec::with_capacity(100 * 100 * 4);
                    for &tile in tiles {
                        let pixel = self.palette[tile as usize];
                        texture_data.push(pixel[0]);
                        texture_data.push(pixel[1]);
                        texture_data.push(pixel[2]);
                        texture_data.push(pixel[3]);
                    }
                    let texture = macroquad::texture::Texture2D::from_rgba8(100, 100, texture_data.as_slice());
                    texture.set_filter(FilterMode::Nearest);
                    self.grids_tiles.push((surface.x(), surface.y(), texture));
                }
                #[cfg(TODO)]
                let owning = ObjTex::plane_from_owning(1100.0, x, y, ol.as_ref()).bake(self.main_color.clone(), &mut self.factory);
                #[cfg(TODO)]
                self.grids_owning.push(owning);
                if let Some(heights) = surface.heights() {
                    self.grids_heights.push((surface.x(), surface.y(), heights.to_vec()));
                } else {
                    warn!("RENDER: surface without heights");
                }
            }
            state::Event::Obj(id, (xy, angle), resid) => {
                debug!("RENDER: obj ({}, {}) {} {}", xy.0, xy.1, angle, resid);
                //TODO ??? separate static objects like trees and
                //dynamic objects like rabbits to two
                //different caches
                self.xux_state.borrow_mut().objects.insert(id, ((xy, angle), resid));
            }
            state::Event::ObjRemove(ref id) => {
                debug!("RENDER: obj remove {}", id);
                self.xux_state.borrow_mut().objects.remove(id);
            }
            state::Event::Hero(position) => {
                debug!("RENDER: hero ({}, {}) {}", position.0 .0, position.0 .1, position.1);
                //TODO ??? add to objects
                self.xux_state.borrow_mut().hero = position;
                if self.target_isnt_set {
                    self.target = vec2(self.xux_state.borrow().hero.0 .0 as f32, self.xux_state.borrow().hero.0 .1 as f32);
                    self.target_isnt_set = false;
                }
            }
            state::Event::Res(id, name) => {
                debug!("RENDER: res {} {}", id, name);
                self.xux_state.borrow_mut().resources.insert(id, name);
            }
            state::Event::Wdg(action) => {
                match action {
                    Wdg::New(id, name, parent_id) => {
                        self.xux_state.borrow_mut().widgets.add_widget(id, name, parent_id).expect("unable to add_widget");
                    }
                    Wdg::Msg(id, name, args) => {
                        self.xux_state.borrow_mut().widgets.message(id, (name, args)).expect("unable to add message");
                    }
                    Wdg::Del(id) => {
                        self.xux_state.borrow_mut().widgets.del_widget(id).expect("unable to del_widget");
                    }
                    Wdg::Add(_id) => {
                        warn!("Wdg::Add should be handled");
                    }
                }
            }
        }
    }

    fn update(&mut self) {
        self.handle_events();
        self.handle_input();
        self.ai_tick();
    }

    fn handle_events (&mut self) {
        loop {
            let event = self.xux_state.borrow().event_rx.try_recv();
            match event {
                Ok(event) => {
                    //#[cfg(feature = "dump_events")]
                    //dumper.dump(&event).expect("unable to dump event");
                    self.event(event);
                }
                Err(Empty) => {
                    break;
                }
                Err(Disconnected) => {
                    info!("render: disconnected from que");
                    self.should_exit = true;
                    break;
                }
            }
        }
    }

    fn handle_input (&mut self) {
        let mut ui_hovered = false;
        egui_macroquad::ui(|egui_ctx| {
            let response = egui::Window::new("Окно №1").show(egui_ctx, |ui| {
                ui.vertical(|ui| {
                    ui.checkbox(&mut self.show_tiles, "Show tiles");
                    ui.checkbox(&mut self.show_heights, "Show heights");
                    ui.checkbox(&mut self.show_owning, "Show owning");
                    ui.add(egui::Slider::new(&mut self.height_threshold, 0.0..=30.0));
                    if ui.button("exit").clicked() {
                        //TODO maybe instead tell self.AI to exit to correctly finish current task and don't do any tasks AI doing currently
                        self.xux_state.borrow().event_tx.send(driver::Event::User(driver::UserInput::Quit)).expect("unable to send User::Quit");
                    }
                    let mut objects_by_resid: BTreeMap<ResID, usize> = BTreeMap::new();
                    for (_, resid) in self.xux_state.borrow().objects.values() {
                        *objects_by_resid.entry(*resid).or_default() += 1;
                    }
                    for (resid, quantity) in objects_by_resid.iter() {
                        ui.horizontal(|ui| {
                            if let Some(resname) = self.xux_state.borrow().resources.get(resid) {
                                ui.label(resname);
                            } else {
                                let resid = resid.to_string();
                                ui.label(resid);
                            }
                            ui.label(quantity.to_string());
                        });
                    }
                });
            });
            if let Some(ref response) = response {
                let mouse = mouse_position();
                ui_hovered = response.rect.contains(Pos2::new(mouse.0, mouse.1));
            }
        });

        self.zoom *= 1.0 + mouse_wheel().1 / 10.0;

        let mouse = Vec2::from(mouse_position());
        let delta = self.mouse - mouse;
        self.mouse = mouse;

        if is_mouse_button_down(MouseButton::Right) {
            self.target += delta / self.zoom;
        }

        self.camera = Camera2D::from_display_rect(Rect::new(0.0, 0.0, screen_width(), screen_height()));
        self.camera.target += self.target;
        self.camera.target -= vec2(screen_width() / 2.0, screen_height() / 2.0);
        self.camera.zoom *= self.zoom;

        if is_mouse_button_pressed(MouseButton::Left) && ! ui_hovered {
            let mark = self.camera.screen_to_world(mouse);
            self.marks.push(mark);
            self.xux_state.borrow().event_tx.send(driver::Event::User(driver::UserInput::Go(mark[0], mark[1]))).expect("unable to send User::Quit");
        }

        if is_quit_requested() {
            self.xux_state.borrow().event_tx.send(driver::Event::User(driver::UserInput::Quit)).expect("unable to send User::Quit");
        }
    }

    fn ai_tick (&mut self) {
        let mut stack = Some(vec!());
        self.behavior_tree.tick(0, &mut stack);
        for &(offset, ref notice) in stack.unwrap().iter() {
            debug!("{}{}", "   ".repeat(offset), notice);
        }
    }

    fn draw(&self) {
        set_camera(&self.camera);
        self.draw_tiles();
        self.draw_heights();
        self.draw_owning();
        self.draw_objects();
        self.draw_marks();
        self.draw_hero();
        //set_default_camera();
        self.draw_gui();
    }

    fn draw_tiles(&self) {
        //TODO self.config.show_tiles: RenderConfig
        if self.show_tiles {
            for &(x, y, texture) in self.grids_tiles.iter() {
                let x = x as f32 * 1100.0;
                let y = y as f32 * 1100.0;
                //debug!("RENDER: draw texture at ({}, {})", x, y);
                let params = DrawTextureParams {
                    dest_size: Some(Vec2::new(1100.0, 1100.0)),
                    ..Default::default()
                };
                macroquad::texture::draw_texture_ex(texture, x, y, WHITE, params);
            }
        }
    }

    fn draw_owning(&self) {
        if self.show_owning {
            #[cfg(TODO)]
            for t in self.state.grids_owning.iter() {
                self.state.encoder.draw(&t.slice, &self.state.pso_tex, &t.data);
            }
        }
    }

    fn draw_heights(&self) {
        if self.show_heights {
            for &(x, y, ref heights) in self.grids_heights.iter() {
                for tx in 0..99 {
                    for ty in 0..99 {
                        let h00 = heights[ty * 100 + tx];
                        let h10 = heights[ty * 100 + tx + 1];
                        let h01 = heights[(ty + 1) * 100 + tx];
                        let h11 = heights[(ty + 1) * 100 + tx + 1];
                        let dh0 = (h00 - h10).abs();
                        let dh1 = (h00 - h01).abs();
                        let dh2 = (h10 - h11).abs();
                        let dh3 = (h01 - h11).abs();
                        let dh = dh0.max(dh1).max(dh2).max(dh3);
                        if dh >= self.height_threshold {
                            let x = x as f32 * 1100.0 + tx as f32 * 11.0 + 1.0;
                            let y = y as f32 * 1100.0 + ty as f32 * 11.0 + 1.0;
                            let w = 9.0;
                            let h = 9.0;
                            macroquad::shapes::draw_rectangle(x, y, w, h, BLACK);
                        }
                    }
                }
            }
        }
    }

    fn draw_objects(&self) {
        for ((ObjXY(x, y), angle), _) in self.xux_state.borrow().objects.values() {
            const OBJ_SIZE: f64 = 4.0;
            {
                let x = (x - OBJ_SIZE / 2.0) as f32;
                let y = (y - OBJ_SIZE / 2.0) as f32;
                let w = OBJ_SIZE as f32;
                let h = OBJ_SIZE as f32;
                macroquad::shapes::draw_rectangle(x, y, w, h, WHITE);
            }
            direction_marker(*x as f32, *y as f32, *angle as f32, OBJ_SIZE as f32, WHITE);
        }
    }

    fn draw_marks(&self) {
        use macroquad::prelude::*;
        for mark in &self.marks {
            const OBJ_SIZE: f32 = 2.0;
            let x = mark[0] - OBJ_SIZE / 2.0;
            let y = mark[1] - OBJ_SIZE / 2.0;
            let w = OBJ_SIZE;
            let h = OBJ_SIZE;
            draw_rectangle(x, y, w, h, RED);
        }
    }

    fn draw_hero(&self) {
        const OBJ_SIZE: f32 = 4.0;
        let x = self.xux_state.borrow().hero.0 .0 as f32 - OBJ_SIZE / 2.0;
        let y = self.xux_state.borrow().hero.0 .1 as f32 - OBJ_SIZE / 2.0;
        let w = OBJ_SIZE;
        let h = OBJ_SIZE;
        macroquad::shapes::draw_rectangle(x, y, w, h, BLUE);
        direction_marker(self.xux_state.borrow().hero.0 .0 as f32, self.xux_state.borrow().hero.0 .1 as f32, self.xux_state.borrow().hero.1 as f32, OBJ_SIZE as f32, BLUE);
    }

    fn draw_gui(&self) {
        egui_macroquad::draw();
    }
}

fn direction_marker(x: f32, y: f32, angle: f32, size: f32, color: Color) {
    let rot = nalgebra::Rotation2::new(angle);
    //TODO let obj = nalgebra::geometry::Translation::from([x, y]);
    let p0 = rot * (nalgebra::Point2::new(1.0, 0.5) * size);
    let p1 = rot * (nalgebra::Point2::new(1.0, -0.5) * size);
    let p2 = rot * (nalgebra::Point2::new(2.0, 0.0) * size);
    let obj = vec2(x, y);
    let p0 = vec2(p0.x as f32, p0.y as f32) + obj;
    let p1 = vec2(p1.x as f32, p1.y as f32) + obj;
    let p2 = vec2(p2.x as f32, p2.y as f32) + obj;
    macroquad::shapes::draw_triangle(p0, p1, p2, color);
}

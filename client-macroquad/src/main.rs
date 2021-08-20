#![feature(stmt_expr_attributes)]

use std::{
    collections::BTreeMap,
    sync::mpsc::{
        Sender,
        Receiver,
        TryRecvError::*,
    },
    io::BufReader,
    fs::File,
    default::Default,
};
use macroquad::prelude::{prevent_quit, clear_background, next_frame, BLACK, WHITE, BLUE, Vec2, Camera2D, mouse_wheel, mouse_position, FilterMode, MouseButton, vec2, is_mouse_button_down, is_mouse_button_pressed, Rect, screen_width, screen_height, is_quit_requested, set_camera, DrawTextureParams, Color};
use xux::{
    Result,
    client,
    driver,
    state,
    proto::{ResID, ObjID, ObjXY},
};
use anyhow::anyhow;
use log::{error, info, warn, debug};
use ron::de::from_reader;
use xux::state::{Surface, WdgID};
use xux::state::Event::Obj;

#[macroquad::main("2d-macroquad-egui")]
async fn main () -> Result<()> {

    env_logger::builder()
        .format_target(false)
        .format_module_path(false)
        .format_level(true)
        .format_timestamp(None)
        .init();

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

        // Process keys, mouse etc.
        render_ctx.update();

        //TODO signal handling

        if render_ctx.should_exit { break; }

        render_ctx.draw();
        next_frame().await
    }
    info!("render thread: done");
    Ok(())
}

struct RenderContext {
    event_tx: Sender<driver::Event>,
    event_rx: Receiver<state::Event>,
    //TODO
    //struct State {
    widgets: BTreeMap<WdgID, (String, WdgID)>, //TODO add Vec<messages> to every widget
    resources: BTreeMap<ResID, String>,
    objects: BTreeMap<ObjID, ((ObjXY,f64), ResID)>,
    hero: (ObjXY, f64),
    hf_x: f32,
    hf_y:f32,
    login: String,
    name: String,
    timestamp: String,
    //}
    //TODO
    //struct RenderState {
    tile_colors: BTreeMap<String,[u8;4]>,
    palette: [[u8; 4]; 256],
    grids_tiles: Vec<(i32, i32, macroquad::texture::Texture2D)>,
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
}

impl RenderContext {
    fn new (event_tx: Sender<driver::Event>, event_rx: Receiver<state::Event>) -> RenderContext {

        //#[cfg(feature = "dump_events")]
        //let mut dumper = dumper::Dumper::init().expect("unable to create dumper");

        RenderContext {
            event_tx: event_tx,
            event_rx: event_rx,
            widgets: BTreeMap::new(),
            resources: BTreeMap::new(),
            objects: BTreeMap::new(),
            hero: (ObjXY(0.0, 0.0), 0.0),
            hf_x: 0.0,
            hf_y: 0.0,
            login: "nobody".into(), //FIXME set to login on login
            name: "noone".into(), //FIXME set to hero name on hero choise
            timestamp: chrono::Local::now().format("%Y-%m-%d %H-%M-%S").to_string(),
            tile_colors: {
                let f = File::open("tile_colors.ron").expect("unable to open tile_colors.ron");
                from_reader(BufReader::new(f)).expect("unable to deserialize")
            },
            palette: [[0,0,250,255]; 256],
            grids_tiles: Vec::new(),
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
        }
    }

    fn tiles_to_png (&self, surface: &Surface) -> Result<()> {
        match surface.tiles() {
            Some(ref tiles) => {
                //FIXME remove expect(), propagate error up
                util::tiles_to_png(&self.login, &self.name, &self.timestamp, surface.x(), surface.y(), tiles /* TODO &s.z */).expect("Unable to save tiles");
                Ok(())
            },
            None => {
                warn!("surface is tileless");
                Ok(())
            },
        }
    }

    fn event (&mut self, event: state::Event) {
        /* TODO app.event(event) */
        match event {
            /*state::Event::Tiles(tiles) => {
                debug!("RENDER: tiles");
            }*/
            state::Event::Surface(ref surface) => {
                debug!("RENDER: surface v{} ({}, {})", surface.version(), surface.x(), surface.y());
                self.tiles_to_png(surface).expect("Unable to save tiles");
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
                    // ObjTex::plane_from_tiles(1100.0, x, y, tiles.as_ref(), &self.palette).bake(self.main_color.clone(), &mut self.factory);
                    self.grids_tiles.push((surface.x(), surface.y(), texture));
                }
                #[cfg(TODO)]
                let owning = ObjTex::plane_from_owning(1100.0, x, y, ol.as_ref())
                    .bake(self.main_color.clone(), &mut self.factory);
                #[cfg(TODO)]
                self.grids_owning.push(owning);
                //let heights = ObjCol::grid_from_heights(100, 11.0, grid_x, grid_y, heights.as_ref(), 1.0)
                //    .bake(main_color.clone(), &mut factory, threshold);
                #[cfg(TODO)]
                let heights = ObjCol::grid_from_heights2(100, 11.0, x, y, z.as_ref())
                    .bake(self.main_color.clone(), &mut self.factory, self.threshold);
                #[cfg(TODO)]
                self.grids_heights.push(heights);
            }
            state::Event::Obj(id,(xy,angle),resid) => {
                debug!("RENDER: obj ({}, {}) {} {}", xy.0, xy.1, angle, resid);
                //TODO ??? separate static objects like trees and
                //dynamic objects like rabbits to two
                //different caches
                self.objects.insert(id, ((xy,angle), resid));
            }
            state::Event::ObjRemove(ref id) => {
                debug!("RENDER: obj remove {}", id);
                self.objects.remove(id);
            }
            state::Event::Hero(position) => {
                debug!("RENDER: hero ({}, {}) {}", position.0.0, position.0.1, position.1);
                //TODO ??? add to objects
                self.hero = position;
                if self.target_isnt_set {
                    self.target = vec2(self.hero.0.0 as f32, self.hero.0.1 as f32);
                    self.target_isnt_set = false;
                }
            }
            state::Event::Res(id, name) => {
                debug!("RENDER: res {} {}", id, name);
                self.resources.insert(id, name);
            }
            state::Event::Wdg(state::Wdg::New(id,name,parent)) => {
                debug!("RENDER: wdg new {} {} {}", id, name, parent);
                self.widgets.insert(id,(name.clone(),parent));
                //self.xui.add_widget(id,name,parent).expect("unable to ui.add_widget");
            }
            state::Event::Wdg(state::Wdg::Msg(id, name, _args)) => {
                debug!("RENDER: wdg msg {} {}", id, name);
                //self.xui.message(id,(name,args)).expect("unable to ui.message");
            }
            state::Event::Wdg(state::Wdg::Del(id)) => {
                debug!("RENDER: wdg del {}", id);
                self.widgets.remove(&id);
                //self.xui.del_widget(id).expect("unable to ui.del_widget");
            }
            state::Event::Hearthfire(ObjXY(x,y)) => {
                debug!("RENDER: hearthfire ({}, {})", x, y);
                self.hf_x = x as f32;
                self.hf_y = y as f32;
            }
        }
    }

    fn update (&mut self) {
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

        if is_mouse_button_pressed(MouseButton::Left) {
            let mark = self.camera.screen_to_world(mouse);
            self.marks.push(mark);
            self.event_tx.send(driver::Event::User(driver::UserInput::Go(mark[0], mark[1]))).expect("unable to send User::Quit");
        }

        if is_quit_requested() {
            self.event_tx.send(driver::Event::User(driver::UserInput::Quit)).expect("unable to send User::Quit");
        }

        loop {
            match self.event_rx.try_recv() {
                Ok(event) => {
                    //#[cfg(feature = "dump_events")]
                    //dumper.dump(&event).expect("unable to dump event");
                    self.event(event);
                }
                Err(Empty) => { break; }
                Err(Disconnected) => {
                    info!("render: disconnected from que");
                    self.should_exit = true;
                    break;
                }
            }
        }

        egui_macroquad::ui(|egui_ctx| {
            egui::Window::new("Окно №1").show(egui_ctx, |ui| {
                ui.vertical(|ui| {
                    if ui.button("exit").clicked() {
                        self.event_tx.send(driver::Event::User(driver::UserInput::Quit)).expect("unable to send User::Quit");
                    }
                    let mut objects_by_resid: BTreeMap<ResID, usize> = BTreeMap::new();
                    for (_, resid) in self.objects.values() {
                        *objects_by_resid.entry(*resid).or_default() += 1;
                    }
                    for (resid,quantity) in objects_by_resid.iter() {
                        ui.horizontal(|ui| {
                            if let Some(resname) = self.resources.get(resid) {
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
        });
    }

    fn draw (&self) {
        set_camera(&self.camera);
        self.draw_tiles();
        self.draw_owning();
        self.draw_heights();
        self.draw_objects();
        self.draw_marks();
        self.draw_hero();
        //set_default_camera();
        self.draw_gui();
    }

    fn draw_tiles (&self) {
        //TODO self.config.show_tiles: RenderConfig
        if self.show_tiles {
            for &(x, y, texture) in self.grids_tiles.iter() {
                let x = x as f32 * 1100.0;
                let y = y as f32 * 1100.0;
                //debug!("RENDER: draw texture at ({}, {})", x, y);
                let params = DrawTextureParams {
                    dest_size: Some(Vec2::new(1100.0, 1100.0)),
                    .. Default::default()
                };
                macroquad::texture::draw_texture_ex(texture, x, y, WHITE, params);
            }
        }
    }

    fn draw_owning (&self) {
        if self.show_owning {
            #[cfg(TODO)] for t in self.state.grids_owning.iter() {
                self.state.encoder.draw(&t.slice, &self.state.pso_tex, &t.data);
            }
        }
    }

    fn draw_heights (&self) {
        if self.show_heights {
            #[cfg(TODO)] for t in self.state.grids_heights.iter() {
                self.state.encoder.draw(&t.slice, &self.state.pso_col, &t.data);
            }
        }
    }

    fn draw_objects (&self) {
        for ((ObjXY(x,y),angle),_) in self.objects.values() {
            const OBJ_SIZE: f64 = 4.0;
            {
                let x = (x + -OBJ_SIZE / 2.0) as f32;
                let y = (y + -OBJ_SIZE / 2.0) as f32;
                let w = OBJ_SIZE as f32;
                let h = OBJ_SIZE as f32;
                macroquad::shapes::draw_rectangle(x, y, w, h, WHITE);
            }
            direction_marker(*x as f32, *y as f32, *angle as f32, OBJ_SIZE as f32, WHITE);
        }
    }

    fn draw_marks (&self) {
        use macroquad::prelude::*;
        for mark in &self.marks {
            const OBJ_SIZE: f32 = 2.0;
            let x = mark[0] + - OBJ_SIZE / 2.0;
            let y = mark[1] + - OBJ_SIZE / 2.0;
            let w = OBJ_SIZE;
            let h = OBJ_SIZE;
            draw_rectangle(x, y, w, h, RED);
        }
    }

    fn draw_hero (&self) {
        const OBJ_SIZE: f32 = 4.0;
        let x = self.hero.0.0 as f32 - OBJ_SIZE / 2.0;
        let y = self.hero.0.1 as f32 - OBJ_SIZE / 2.0;
        let w = OBJ_SIZE;
        let h = OBJ_SIZE;
        macroquad::shapes::draw_rectangle(x, y, w, h, BLUE);
        direction_marker(self.hero.0.0 as f32, self.hero.0.1 as f32, self.hero.1 as f32, OBJ_SIZE as f32, BLUE);
    }

    fn draw_gui (&self) {
        egui_macroquad::draw();
    }
}

fn direction_marker (x: f32, y: f32, angle: f32, size: f32, color: Color) {
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
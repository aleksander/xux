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
use macroquad::prelude::*;
use xux::{
    Result,
    client,
    driver,
    state,
    proto::{ResID, ObjID, ObjXY},
};
use anyhow::anyhow;
use log::trace;
use ron::de::from_reader;
use xux::state::Surface;

#[macroquad::main("2d-macroquad-egui")]
async fn main () -> Result<()> {
    //let log_file_name = "xux.hafen.log";

    fern::Dispatch::new()
        .level(log::LevelFilter::Debug)
        .level_for("xux", log::LevelFilter::Debug)
        .chain(std::io::stdout())
        /*.chain(
            //fern::log_file(log_file_name)
            std::fs::OpenOptions::new()
                .write(true)
                .create(true)
                .append(false)
                .open(log_file_name)?)*/
        .apply()?;

    trace!("Starting...");
    debug!("Starting...");
    info!("Starting...");
    warn!("Starting...");
    error!("Starting...");

    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        info!("wrong argument count");
        info!("usage: {} username password", args[0]);
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

        //TODO correct network session termination on window closing
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
    widgets: BTreeMap<u16, (String, u16)>, //TODO add Vec<messages> to every widget
    resources: BTreeMap<ResID, String>,
    objects: BTreeMap<ObjID, (ObjXY, ResID)>,
    hero_x: f32,
    hero_y: f32,
    hero_isnt_set_yet: bool,
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
            hero_x: 0.0,
            hero_y: 0.0,
            hero_isnt_set_yet: true,
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
            state::Event::Obj(id,xy,resid) => {
                debug!("RENDER: obj ({}, {})", xy.0, xy.1);
                //TODO ??? separate static objects like trees and
                //dynamic objects like rabbits to two
                //different caches
                self.objects.insert(id, (xy, resid));
            }
            state::Event::ObjRemove(ref id) => {
                debug!("RENDER: obj remove {}", id);
                self.objects.remove(id);
            }
            state::Event::Hero(ObjXY(x,y)) => {
                debug!("RENDER: hero ({}, {})", x, y);
                //TODO ??? add to objects
                self.hero_x = x as f32;
                self.hero_y = y as f32;
                if self.hero_isnt_set_yet {
                    self.target = vec2(self.hero_x, self.hero_y);
                    self.hero_isnt_set_yet = false;
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
                    ui.label("Test");
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
        for (ObjXY(x,y),_) in self.objects.values() {
            const OBJ_SIZE: f64 = 4.0;
            let x = (x + - OBJ_SIZE / 2.0) as f32;
            let y = (y + - OBJ_SIZE / 2.0) as f32;
            let w = OBJ_SIZE as f32;
            let h = OBJ_SIZE as f32;
            macroquad::shapes::draw_rectangle(x, y, w, h, WHITE);
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
        let x = self.hero_x - OBJ_SIZE / 2.0;
        let y = self.hero_y - OBJ_SIZE / 2.0;
        let w = OBJ_SIZE;
        let h = OBJ_SIZE;
        macroquad::shapes::draw_rectangle(x, y, w, h, BLUE);
    }

    fn draw_gui (&self) {
        egui_macroquad::draw();
    }
}
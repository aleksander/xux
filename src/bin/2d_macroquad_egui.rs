#![feature(stmt_expr_attributes)]

use std::{
    collections::BTreeMap,
    sync::mpsc::{
        Sender,
        TryRecvError::*,
    }
};
use macroquad::prelude::*;
use xux::{
    Result,
    client,
    driver,
    state,
    proto::{ResID, ObjID, ObjXY},
};
use failure::{err_msg, format_err};
use log::trace;
use ron::de::from_reader;
use std::io::BufReader;
use std::fs::File;

#[macroquad::main("2d-macroquad-egui")]
async fn main () -> Result<()> {
    #[cfg(feature = "salem")]
    let log_file_name = "xux.salem.log";
    #[cfg(feature = "hafen")]
    let log_file_name = "xux.hafen.log";

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
        return Err(err_msg("wrong argument count"));
    }

    let username = args[1].clone();
    let password = args[2].clone();

    #[cfg(feature = "salem")]
    let host = "game.salemthegame.com";
    #[cfg(feature = "hafen")]
    let host = "game.havenandhearth.com";

    let auth_port = 1871;
    let game_port = 1870;

    //TODO take all authorisation information from the GUI (maybe cache values in .config after the user first time enters them)

    let (login, cookie) = client::authorize(host, auth_port, username, password)?;

    let (ll_event_tx, hl_event_rx) = client::run_threaded(host, game_port, login, cookie)?;

    //#[cfg(feature = "dump_events")]
    //let mut dumper = dumper::Dumper::init().expect("unable to create dumper");

    let mut render_ctx = RenderContext::new(ll_event_tx);

    'outer: loop {
        clear_background(BLACK);

        // Process keys, mouse etc.
        render_ctx.update();

        loop {
            match hl_event_rx.try_recv() {
                Ok(event) => {
                    //#[cfg(feature = "dump_events")]
                    //dumper.dump(&event).expect("unable to dump event");
                    render_ctx.event(event);
                }
                Err(Empty) => { break; }
                Err(Disconnected) => {
                    info!("render: disconnected from que");
                    break 'outer;
                }
            }
        }

        egui_macroquad::ui(|egui_ctx| {
            egui::Window::new("Окно №1")
                .show(egui_ctx, |ui| {
                    ui.label("Test");
                });
        });

        // Draw things before egui
        render_ctx.draw();

        egui_macroquad::draw();

        // Draw things after egui

        next_frame().await
    }
    info!("render thread: done");
    Ok(())
}

struct RenderContext {
    event_tx: Sender<driver::Event>,
    #[warn(TODO)]
    //struct State {
    widgets: BTreeMap<u16, (String, u16)>, //TODO add Vec<messages> to every widget
    resources: BTreeMap<ResID, String>,
    objects: BTreeMap<ObjID, (ObjXY, ResID)>,
    hero_x: f32,
    hero_y: f32,
    hf_x: f32,
    hf_y:f32,
    //}
    #[warn(TODO)]
    //struct RenderState {
    tile_colors: BTreeMap<String,[u8;4]>,
    palette: [[u8; 4]; 256],
    shift: [f32; 2], //TODO replace by camera.{x,y}
    grids_tiles: Vec<(i32, i32, macroquad::texture::Texture2D)>,
    // }
    #[warn(TODO)]
    //struct RenderConfig {
    show_tiles: bool,
    show_heights: bool,
    show_owning: bool,
    // }
}

impl RenderContext {
    fn new (event_tx: Sender<driver::Event>) -> RenderContext {
        RenderContext {
            event_tx: event_tx,
            widgets: BTreeMap::new(),
            resources: BTreeMap::new(),
            objects: BTreeMap::new(),
            hero_x: 0.0,
            hero_y: 0.0,
            hf_x: 0.0,
            hf_y: 0.0,
            tile_colors: {
                let f = File::open("tile_colors.ron").expect("unable to open tile_colors.ron");
                from_reader(BufReader::new(f)).expect("unable to deserialize")
            },
            palette: [[0,0,250,255]; 256],
            shift: [0.0, 0.0],
            grids_tiles: Vec::new(),
            show_tiles: true,
            show_heights: true,
            show_owning: true,
        }
    }

    fn event (&mut self, event: state::Event) {
        /* TODO app.event(event) */
        match event {
            /*state::Event::Tiles(tiles) => {
                debug!("RENDER: tiles");
            }*/
            state::Event::Surface(surface) => {
                debug!("RENDER: surface v{} ({}, {})", surface.version(), surface.x(), surface.y());
                if let Some(tileres) = surface.tileres() {
                    for tile in tileres {
                        debug!("RENDER: tile {} {}", tile.id, tile.name);
                        if let Some(color) = self.tile_colors.get(&tile.name) {
                            self.palette[tile.id as usize] = *color;
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
                self.objects.insert(id, (xy,resid));
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
                //FIXME self.shift += Vector(-hero_x,-hero_y);
                self.shift[0] = -self.hero_x;
                self.shift[1] = -self.hero_y;
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
        // HANDLE EVENTS
        //TODO app.handle(...)
        #[cfg(OLD)]{
            let mut should_stop = false;
            let state = &mut self.state;
            self.events_loop.poll_events(|event| {
                state.update(render_tx, &event, &mut should_stop);
            });
            if should_stop {
                return false;
            }
        }

        // UPDATE
        #[cfg(OLD)]
        let delta_s = self.state.delta.tick();

        //TODO app.update(delta_s);
        //TODO camera.rotate(angle);
        //self.angle += delta_s * 0.1;
    }

    fn draw (&self) {
        // RENDER
        //TODO app.render(...)
        //FIXME recalc matrices only if something changed (w,h,angle,zoom)
        //TODO let transform = transform(w,h,camera)
        #[cfg(OLD)]let translate = Matrix3::new(
            1.0,           0.0,           0.0,
            0.0,           1.0,           0.0,
            self.state.shift[0], self.state.shift[1], 1.0,
        );
        #[cfg(OLD)]let rotate = Matrix3::from_angle_z(Rad(self.state.angle));
        #[cfg(OLD)]let scale = Matrix3::from_diagonal(Vector3::new(self.state.zoom / self.state.w as f32, self.state.zoom / self.state.h as f32, 1.0));
        #[cfg(OLD)]let transform = (scale * rotate * translate).into();

        #[cfg(OLD)]if self.state.show_tiles {
            for t in self.state.grids_tiles.iter_mut() {
                t.data.transform = transform;
            }
        }
        #[cfg(OLD)]if self.state.show_owning {
            for t in self.state.grids_owning.iter_mut() {
                t.data.transform = transform;
            }
        }
        #[cfg(OLD)]if self.state.show_heights {
            for t in self.state.grids_heights.iter_mut() {
                t.data.transform = transform;
                t.data.threshold = self.state.threshold as i32;
            }
        }

        #[cfg(OLD)]self.state.encoder.clear(&self.state.main_color, BLACK);

        self.draw_tiles();
        self.draw_owning();
        self.draw_heights();
        self.draw_objects();
        self.draw_gui();
    }

    fn draw_tiles (&self) {
        //TODO self.config.show_tiles: RenderConfig
        if self.show_tiles {
            for &(x, y, texture) in self.grids_tiles.iter() {
                let x = x as f32 * 100.0 + self.shift[0] / 10.0 + screen_width() / 2.0;
                let y = y as f32 * 100.0 + self.shift[1] / 10.0 + screen_height() / 2.0;
                debug!("RENDER: draw texture at ({}, {})", x, y);
                macroquad::texture::draw_texture(texture, x, y, WHITE);
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
        /*
        let mut obj = ObjCol::from_objects(&self.state.objects, self.state.hero_x, self.state.hero_y, self.state.hf_x, self.state.hf_y).bake(self.state.main_color.clone(), &mut self.state.factory, self.state.threshold);
        obj.data.transform = transform;
        obj.data.threshold = 0;
        self.state.encoder.draw(&obj.slice, &self.state.pso_col, &obj.data);
         */
    }

    fn draw_gui (&self) {
        /*
        let (width, height) = self.state.window.get_inner_size().expect("unable to get_inner_size").into();
        let ui = self.state.imgui.frame();
        self.state.imgui_want_capture_mouse = ui.want_capture_mouse();
        self.state.imgui_want_capture_keyboard = ui.want_capture_keyboard();

        let fps = (1.0 / delta_s) as usize;
        //FIXME pass &mut RenderImplState instead
        if !run_ui(&ui, fps, self.state.threshold, &mut self.state.show_tiles, &mut self.state.show_heights, &mut self.state.show_owning, &mut self.state.v1, &self.state.objects, &self.state.resources) {
            return false;
        }
        self.state.imgui_renderer.render(ui, &mut self.state.factory, &mut self.state.encoder).expect("IMGUI Rendering failed");
         */
    }
}
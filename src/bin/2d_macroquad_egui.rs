use macroquad::prelude::*;
use xux::Result;
use xux::client;
use failure::err_msg;
use log::trace;

#[macroquad::main("2d-macroquad-egui")]
async fn main () -> Result<()> {
    #[cfg(feature = "salem")]
        let log_file_name = "xux.salem.log";
    #[cfg(feature = "hafen")]
        let log_file_name = "xux.hafen.log";

    fern::Dispatch::new()
        .level(log::LevelFilter::Error)
        .level_for("xux", log::LevelFilter::Debug)
        .chain(std::io::stdout())
        .chain(
            //fern::log_file(log_file_name)
            std::fs::OpenOptions::new()
                .write(true)
                .create(true)
                .append(false)
                .open(log_file_name)?)
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

    loop {
        clear_background(BLACK);

        // Process keys, mouse etc.

        egui_macroquad::ui(|egui_ctx| {
            egui::Window::new("Окно №1")
                .show(egui_ctx, |ui| {
                    ui.label("Test");
                });
        });

        // Draw things before egui

        egui_macroquad::draw();

        // Draw things after egui

        next_frame().await
    }
}
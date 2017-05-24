#[macro_use]
extern crate log;
extern crate fern;
extern crate xux;

use std::process;

use xux::errors::*;
use xux::ai::Ai;
use xux::ai_decl::AiDecl;
use xux::driver_std::DriverStd;
use xux::render::{Render, RenderKind};
use xux::client::Client;

// TODO
// extern crate nix;
// use nix::sys::socket::setsockopt;
// use nix::sys::socket::SockLevel;
// use nix::sys::socket::SockOpt;
//
// #[derive(Debug,Copy,Clone)]
// struct BindToDevice {
//     dev_name: &'static str
// }
//
// impl BindToDevice {
//     fn new (dev_name: &'static str) -> BindToDevice {
//         BindToDevice{ dev_name: dev_name}
//     }
// }
//
// impl SockOpt for BindToDevice {
//     type Val = &'static str;
//     fn get (&self, fd: RawFd, level: c_int) -> Result<&'static str> { ... }
//     fn set () -> ? { ... }
// }
//
// //char *opt;
// //opt = "eth0";
// //setsockopt(sd, SOL_SOCKET, SO_BINDTODEVICE, opt, 4);
// nix::sys::socket::setsockopt(sock.as_raw_fd, SockLevel::Socket, BindToDevice::new("wlan0"));

// TODO fn run_std_lua() { run::<Std,Lua>() }
// TODO fn run<D,A>(ip: IpAddr, username: String, password: String) where D:Driver,A:Ai {
fn run() -> Result<()> {

    //TODO get "<crate name>.log" file name automatically from cargo (macro?)
    #[cfg(feature = "salem")]
    let log_file_name = "xux.salem.log";
    #[cfg(feature = "hafen")]
    let log_file_name = "xux.hafen.log";

    //TODO prefix logs with timestamp(absolute/relative), file name, line number, function name
    //TODO colorize stdout output: ERROR is RED, WARN is YELLOW etc
    fern::Dispatch::new()
        /*
        .format(|out, message, record| {
            out.finish(
                format_args!("{}[{}][{}] {}",
                    chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                    record.target(),
                    record.level(),
                    message))
        })
        */
        .level(log::LogLevelFilter::Debug)
        .chain(std::io::stdout())
        .chain(
            //fern::log_file(log_file_name)
            std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .append(false)
            .open(log_file_name)
            .chain_err(||"unable to create log file")?)
        .apply().chain_err(||"unable to create log config")?;

    trace!("Starting...");
    debug!("Starting...");
    info!("Starting...");
    warn!("Starting...");
    error!("Starting...");

    // TODO arg parsing with 'clap'
    // TODO handle keyboard interrupt
    // TODO replace all unwraps and expects with normal error handling
    // TODO various formatters for Message and other structs output (full "{:f}", short "{:s}", type only "{:t}")
    // TODO use rustfmt precommit hook
    // TODO add src/bin/mapmerger app to merge sessions (rewrite mapmerger in rust)
    //      (implement when PNGs will be saved in user/char/session subdirs)

    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        info!("wrong argument count");
        info!("usage: {} username password", args[0]);
        return Err("wrong argument count".into());
    }

    let username = args[1].clone();
    let password = args[2].clone();
    #[cfg(feature = "salem")]
    let host = "game.salemthegame.com";
    #[cfg(feature = "hafen")]
    let host = "game.havenandhearth.com";
    let auth_port = 1871;
    let game_port = 1870;

    // run::<DriverMio,AiLua>(ip, username, password);
    //run(host, username, password);
    let (login, cookie) = xux::client::authorize(host, auth_port, username, password).chain_err(||"authorization failed")?;

    let mut ai = AiDecl::new();
    ai.init();

    let mut driver = DriverStd::new(host, game_port).chain_err(||"unable to create driver")?;

    let render = Render::new(RenderKind::TwoD, driver.sender());

    let mut client = Client::new(&mut driver, &mut ai, render);
    client.run(&login, &cookie)
}

fn main () {
    if let Err(ref e) = run() {
        println!("error: {}", e);

        for e in e.iter().skip(1) {
            println!("caused by: {}", e);
        }

        if let Some(backtrace) = e.backtrace() {
            println!("backtrace: {:?}", backtrace);
        }

        process::exit(1);
    }
}

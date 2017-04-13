#[macro_use]
extern crate log;
extern crate fern;
extern crate sac;

use std::process;

use sac::errors::*;
use sac::ai::Ai;
use sac::ai_decl::AiDecl;
use sac::driver_std::DriverStd;
use sac::client::Client;

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
    // let mut log_open_options = std::fs::OpenOptions::new();
    // let log_open_options = log_open_options.create(true).read(true).write(true).truncate(true);
    let logger_config = fern::DispatchConfig {
        format: Box::new(|msg: &str, level: &log::LogLevel, _location: &log::LogLocation| {
            // format!("[{}][{}] {}", time::now().strftime("%Y-%m-%d][%H:%M:%S").unwrap(), level, msg)
            // TODO prefix logs with timestamp(absolute/relative), file name, line number, function name
            format!("[{}] {}", level, msg)
        }),
        output: vec![
            fern::OutputConfig::stdout(), //TODO colorize stdout output: ERROR is RED, WARN is YELLOW etc
            //fern::OutputConfig::file_with_options("log", &log_open_options)
        ],
        level: log::LogLevelFilter::Trace,
    };

    if let Err(e) = fern::init_global_logger(logger_config, log::LogLevelFilter::Trace) {
        panic!("Failed to initialize global logger: {}", e);
    }

    trace!("Starting...");
    debug!("Starting...");
    info!("Starting...");
    warn!("Starting...");
    error!("Starting...");

    // TODO handle keyboard interrupt
    // TODO replace all unwraps and expects with normal error handling
    // TODO various formatters for Message and other structs output (full "{:f}", short "{:s}", type only "{:t}")
    // TODO use rustfmt precommit hook

    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        info!("wrong argument count");
        info!("usage: {} username password", args[0]);
        return Err("wrong argument count".into());
    }

    let username = args[1].clone();
    let password = args[2].clone();
    let host = "game.salemthegame.com";
    let auth_port = 1871;
    let game_port = 1870;

    // run::<DriverMio,AiLua>(ip, username, password);
    //run(host, username, password);
    let (login, cookie) = sac::client::authorize(host, auth_port, username, password).chain_err(||"authorization failed")?;

    let mut ai = AiDecl::new();
    ai.init();
    let mut driver = DriverStd::new(host, game_port).chain_err(||"unable to create driver")?;

    let mut client = Client::new(&mut driver, &mut ai);
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

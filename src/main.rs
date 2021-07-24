use std::process;
use xux::Result;
use xux::client;
use failure::err_msg;
use log::{trace, debug, info, warn, error};

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

    let (login, cookie) = client::authorize(host, auth_port, username, password)?;

    client::run(host, game_port, &login, &cookie)
}

fn main () {
    process::exit(
        match run() {
            Ok(()) => { 0 }
            Err(e) => {
                println!("error \"{}\", cause \"{}\"", e, e.as_fail());
                println!("trace:");
                println!("{}", e.backtrace());
                -1
            }
        }
    );
}

#![allow(incomplete_features)]
#![feature(iter_map_windows)]
#![feature(try_blocks)]
#![feature(duration_millis_float)]
#![feature(lazy_get)]
#![feature(generic_const_exprs)]

use std::{path::PathBuf, time::Duration};

use clap::{crate_authors, crate_name, ArgAction, Parser};
use game::debug_game_configs::DebugGameConfiguration;
use ggez::{
    conf::{FullscreenType, WindowMode},
    event, ContextBuilder, GameResult,
};
use ggez_no_re::{
    logger::{LogLevel, LoggerBuilder},
    util::{self, ResultExtToGameError},
};
use log::debug;
use main_client::MainClient;
use shared::Shared;

mod colors;
mod game;
mod game_client;
mod main_client;
mod main_menu_client;
mod multiplayer;
mod pos;
mod shared;
mod tile;

const LATEST_RELEASE_LINK: &str = "https://github.com/Maurdekye/carcassonne/releases/latest";

fn fullscreen_value_parser(x: &str) -> Result<(usize, usize), &'static str> {
    let parts: Vec<&str> = x.split('x').collect();
    if parts.len() != 2 {
        return Err("Invalid format");
    }
    let width = parts[0].parse::<usize>().map_err(|_| "Invalid width")?;
    let height = parts[1].parse::<usize>().map_err(|_| "Invalid height")?;
    Ok((width, height))
}

fn duration_value_parser(x: &str) -> Result<Duration, &'static str> {
    if let Ok(seconds) = x.parse::<u64>() {
        Ok(Duration::from_secs(seconds))
    } else if let Ok(seconds) = x.parse::<f64>() {
        Ok(Duration::from_secs_f64(seconds))
    } else {
        Err("Invalid duration format")
    }
}

#[derive(Parser, Clone, Debug)]
struct Args {
    /// Start in fullscreen; optionally provide a resolution to run with that res. [default: 1920x1080]
    #[arg(short, long, value_parser = fullscreen_value_parser, default_missing_value = "1920x1080")]
    fullscreen: Option<Option<(usize, usize)>>,

    /// Immediately start a debug game configuration
    #[arg(short = 'c', long)]
    debug_game: Option<DebugGameConfiguration>,

    /// Enable experimental snapping tile placement
    #[arg(short = 'p', long, action = ArgAction::SetTrue)]
    snap_placement: bool,

    /// Ping interval in seconds for multiplayer games.
    #[arg(short = 'g', long, default_value = "5", value_parser = duration_value_parser)]
    ping_interval: Duration,

    /// Enable to save ongoing game progress to this directory [default: saves/]
    #[arg(short = 'v', long, default_missing_value = "saves/")]
    save_games: Option<Option<PathBuf>>,

    /// Enable to save logs to this path [default: logs/]
    #[arg(short = 'o', long, default_missing_value = "logs/")]
    save_logs: Option<Option<PathBuf>>,

    /// Logging level
    #[arg(short = 'e', long, default_value = "info")]
    log_level: LogLevel,

    /// Load a save file
    #[arg(short, long)]
    load: Option<PathBuf>,

    /// Load this save file when starting a multiplayer game
    #[arg(short, long)]
    multiplayer_load: Option<PathBuf>,

    /// Enables debug mode: increases log level to 'trace',
    /// enables saving log files, and enables saving game state
    #[arg(short, long, action = ArgAction::SetTrue)]
    debug: bool,

    /// Path to save persistent data to.
    #[arg(short, long, default_value = "data.json")]
    save_path: PathBuf,
}

fn main() -> GameResult {
    let mut args = Args::parse();
    if args.debug {
        args.save_games = args.save_games.or(Some(Some(PathBuf::from("saves/"))));
        args.save_logs = args.save_logs.or(Some(Some(PathBuf::from("logs/"))));
        args.log_level = LogLevel::Trace;
    }

    LoggerBuilder::new()
        .path_option(args.save_logs.clone().flatten())
        .level(args.log_level)
        .prefix(module_path!())
        .prefix("ggez_no_re")
        .install()
        .to_gameerror()?;

    debug!("Logger initialized");
    debug!("Arguments: {args:#?}");

    let shared = Shared::new(args);

    let window_mode = if let Some(fullscreen_res) = shared.args.fullscreen {
        let (w, h) = fullscreen_res.unwrap();
        WindowMode::default()
            .dimensions(w as f32, h as f32)
            .fullscreen_type(FullscreenType::Desktop)
            .borderless(true)
    } else {
        WindowMode::default()
            .dimensions(1280.0, 720.0)
            .resizable(true)
    };
    debug!("window_mode = {window_mode:?}");

    let (ctx, event_loop) = ContextBuilder::new(crate_name!(), crate_authors!())
        .window_mode(window_mode)
        .build()?;

    let client = MainClient::new(shared);
    debug!("initialized main client");

    event::run(ctx, event_loop, client)
}

#![feature(iter_map_windows)]
#![feature(try_blocks)]
#![feature(duration_millis_float)]
#![feature(lazy_get)]

use std::{path::PathBuf, time::Duration};

use clap::{ArgAction, Parser, ValueEnum};
use ggez::{
    conf::{FullscreenType, WindowMode, WindowSetup},
    event, ContextBuilder, GameResult,
};
use log::debug;
use logger::Logger;
use main_client::MainClient;
use shared::SharedResources;
use util::ResultExt;

mod colors;
mod game;
mod game_client;
mod keycode;
mod line;
mod logger;
mod main_client;
mod main_menu_client;
mod multiplayer;
mod persist;
mod pos;
mod shared;
mod sub_event_handler;
mod tile;
mod ui_manager;
mod util;

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

#[derive(ValueEnum, Clone, Debug)]
enum DebugGameConfiguration {
    MeeplePlacement,
    MultipleSegmentsPerTileScoring,
    MultiplePlayerOwnership,
    RotationTest,
    GroupCoallation,
}

#[derive(ValueEnum, Debug, Clone, Copy)]
enum LogLevel {
    Off,
    Error,
    Warn,
    Info,
    Debug,
    Trace,
    Full,
}

impl From<LogLevel> for log::LevelFilter {
    fn from(value: LogLevel) -> Self {
        use LogLevel::*;
        match value {
            Off => log::LevelFilter::Off,
            Error => log::LevelFilter::Error,
            Warn => log::LevelFilter::Warn,
            Info => log::LevelFilter::Info,
            Debug => log::LevelFilter::Debug,
            Trace | Full => log::LevelFilter::Trace,
        }
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

    let logger = Logger::new(args.clone())?;
    log::set_boxed_logger(Box::new(logger))
        .map(|()| log::set_max_level(args.log_level.into()))
        .to_gameerror()?;

    debug!("Logger initialized");
    debug!("Arguments: {args:#?}");

    let shared = SharedResources::new(args);

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

    let (ctx, event_loop) = ContextBuilder::new("carcassonne", "maurdekye")
        .window_mode(window_mode)
        .window_setup(WindowSetup::default().title("Carcassonne"))
        .build()?;

    let client = MainClient::new(shared);
    debug!("initialized main client");

    event::run(ctx, event_loop, client);
}

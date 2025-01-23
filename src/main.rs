#![feature(iter_map_windows)]

use clap::{ArgAction, Parser};
use ggez::{
    conf::{WindowMode, WindowSetup},
    event, ContextBuilder, GameResult,
};
use main_client::MainClient;

mod game;
mod game_client;
mod main_client;
mod main_menu_client;
mod pos;
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

#[derive(Parser, Clone)]
struct Args {
    /// Start in fullscreen; optionally provide a resolution to run with that res. [default: 1920x1080]
    #[arg(short, long, value_parser = fullscreen_value_parser)]
    fullscreen: Option<Option<(usize, usize)>>,

    /// Enable debug mode
    #[arg(short, long, action = ArgAction::SetTrue)]
    debug: bool,
}

fn main() -> GameResult {
    let args = Args::parse();

    let window_mode = if let Some(fullscreen_res) = args.fullscreen {
        let (w, h) = fullscreen_res.unwrap_or((1920, 1080));
        WindowMode::default()
            .dimensions(w as f32, h as f32)
            .fullscreen_type(ggez::conf::FullscreenType::Desktop)
            .borderless(true)
    } else {
        WindowMode::default()
            .dimensions(1280.0, 720.0)
            .resizable(true)
    };

    let (ctx, event_loop) = ContextBuilder::new("carcassonne", "maurdekye")
        .window_mode(window_mode)
        .window_setup(WindowSetup::default().title("Carcassonne"))
        .build()?;

    let client = MainClient::new(args);

    event::run(ctx, event_loop, client);
}

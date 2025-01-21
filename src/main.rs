#![feature(iter_map_windows)]

use clap::Parser;
use game_client::GameClient;
use ggez::{
    conf::{WindowMode, WindowSetup},
    event, ContextBuilder, GameResult,
};
use main_client::MenuClient;

mod game;
mod game_client;
mod pos;
mod tile;
mod util;

mod main_client {
    use ggez::{
        event::EventHandler,
        glam::{vec2, Vec2},
        graphics::{Canvas, Color, DrawParam, Text},
        GameError,
    };

    pub struct MenuClient {}

    impl MenuClient {
        pub fn new() -> Self {
            Self {}
        }
    }

    impl EventHandler<GameError> for MenuClient {
        fn update(&mut self, _ctx: &mut ggez::Context) -> Result<(), GameError> {
            Ok(())
        }

        fn draw(&mut self, ctx: &mut ggez::Context) -> Result<(), GameError> {
            let mut canvas = Canvas::from_frame(ctx, Color::WHITE);
            let mut menu_text = Text::new("Carcassone");
            menu_text.set_scale(144.0);
            let text_size: Vec2 = menu_text.measure(ctx)?.into();
            let res: Vec2 = ctx.gfx.drawable_size().into();
            canvas.draw(
                &menu_text,
                DrawParam::from((res - text_size) * vec2(0.5, 0.25)),
            );

            canvas.finish(ctx)
        }
    }
}

mod menu_client {}

fn player_count_parser(x: &str) -> Result<usize, &'static str> {
    match x.parse() {
        Ok(n) if (2..=5).contains(&n) => Ok(n),
        _ => Err("Players must be between 2-5"),
    }
}

fn fullscreen_value_parser(x: &str) -> Result<(usize, usize), &'static str> {
    let parts: Vec<&str> = x.split('x').collect();
    if parts.len() != 2 {
        return Err("Invalid format");
    }
    let width = parts[0].parse::<usize>().map_err(|_| "Invalid width")?;
    let height = parts[1].parse::<usize>().map_err(|_| "Invalid height")?;
    Ok((width, height))
}

#[derive(Parser)]
struct Args {
    /// Number of players.
    #[arg(short, long, default_value_t = 2, value_parser = player_count_parser)]
    players: usize,

    /// Start in fullscreen; optionally provide a resolution to run with that res. Default 1080p.
    #[arg(short, long, value_parser = fullscreen_value_parser)]
    fullscreen: Option<Option<(usize, usize)>>,
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
            .dimensions(1200.0, 1000.0)
            .resizable(true)
    };

    let (ctx, event_loop) = ContextBuilder::new("carcassonne", "maurdekye")
        .window_mode(window_mode)
        .window_setup(WindowSetup::default().title("Carcassonne"))
        .build()?;

    // let client = GameClient::new(&ctx, args.players);

    // let mut game = Game::new_with_library(vec![
    //     STARTING_TILE.clone(),
    //     STARTING_TILE.clone(),
    //     STARTING_TILE.clone(),
    //     STARTING_TILE.clone(),
    // ]);
    // game.players.insert(Player::new(Color::RED));
    // game.players.insert(Player::new(Color::BLUE));
    // game.place_tile(STARTING_TILE.clone(), GridPos(0, 0))?;
    // let client = GameClient::new_with_game(&ctx, game);

    let client = MenuClient::new();

    event::run(ctx, event_loop, client);
}

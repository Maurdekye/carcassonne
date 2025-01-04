// use std::io::{stdout, Write};

use ggez::{
    conf::{WindowMode, WindowSetup},
    event::{self, EventHandler},
    graphics::{Canvas, Color, DrawMode, DrawParam, Mesh, Rect},
    input::keyboard::KeyCode,
    mint::Point2,
    Context, ContextBuilder, GameError, GameResult,
};
use tile::{get_tile_library, Tile};

mod tile;
mod util;

const GRID_SIZE: f32 = 0.1;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Pos(i32, i32);

impl Pos {
    fn rect(&self, ctx: &Context) -> Rect {
        let resolution = ctx.gfx.window().inner_size();
        let width = resolution.width as f32 * GRID_SIZE;
        let height = resolution.height as f32 * GRID_SIZE;
        let near_corner = grid_pos_to_screen_pos(*self, ctx);
        Rect::new(near_corner.x, near_corner.y, width, height)
    }
}

struct Game {
    library: Vec<Tile>,
    placed_tiles: Vec<(Pos, Tile)>,
    selected_square: Option<Pos>,
    held_tile: Option<Tile>,
}

impl Game {
    fn new() -> Self {
        Self {
            library: get_tile_library(),
            placed_tiles: Vec::new(),
            selected_square: None,
            held_tile: None,
        }
    }
}

fn screen_pos_to_grid_pos(screen_pos: Point2<f32>, ctx: &Context) -> Pos {
    let res = ctx.gfx.window().inner_size();
    let uv = Point2 {
        x: screen_pos.x / res.width as f32,
        y: screen_pos.y / res.height as f32,
    };
    Pos((uv.x / GRID_SIZE) as i32, (uv.y / GRID_SIZE) as i32)
}

fn grid_pos_to_screen_pos(grid_pos: Pos, ctx: &Context) -> Point2<f32> {
    let res = ctx.gfx.window().inner_size();
    Point2 {
        x: (grid_pos.0 as f32 * GRID_SIZE) * res.width as f32,
        y: (grid_pos.1 as f32 * GRID_SIZE) * res.height as f32,
    }
}

impl EventHandler<GameError> for Game {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        let mouse = ctx.mouse.position();
        let grid_pos = screen_pos_to_grid_pos(mouse, ctx);
        self.selected_square = Some(grid_pos);
        if let Some(tile) = &mut self.held_tile {
            if ctx.keyboard.is_key_just_pressed(KeyCode::R) {
                tile.rotate();
            }

            if ctx.mouse.button_just_pressed(event::MouseButton::Left)
                && !self.placed_tiles.iter().any(|(pos, _)| pos == &grid_pos)
            {
                self.placed_tiles.push((grid_pos, tile.clone()));
                self.held_tile = None;
            }
        } else {
            self.held_tile = self.library.pop();
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas = Canvas::from_frame(ctx, Color::WHITE);

        for (pos, tile) in &self.placed_tiles {
            tile.render(ctx, &mut canvas, pos.rect(ctx))?;
        }

        if let Some(pos) = self.selected_square {
            let rect = pos.rect(ctx);
            let cursor_color = if self.placed_tiles.iter().any(|(p, _)| p == &pos) {
                Color::RED
            } else {
                if let Some(tile) = &self.held_tile {
                    tile.render(ctx, &mut canvas, rect)?;
                }
                Color::GREEN
            };
            canvas.draw(
                &Mesh::new_rectangle(ctx, DrawMode::stroke(2.0), rect, cursor_color)?,
                DrawParam::default(),
            )
        }

        ctx.gfx
            .set_window_title(&format!("Carcassone: {:.2} fps", ctx.time.fps()));

        canvas.finish(ctx)
    }
}

fn main() -> GameResult {
    let (ctx, event_loop) = ContextBuilder::new("carcassone", "maurdekye")
        .window_mode(WindowMode::default().dimensions(800.0, 800.0))
        .window_setup(WindowSetup::default().title("Carcassone"))
        .build()?;
    let game = Game::new();
    event::run(ctx, event_loop, game);
}

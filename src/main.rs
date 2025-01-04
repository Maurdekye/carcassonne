// use std::io::{stdout, Write};

use std::{collections::HashMap, ops::Add};

use ggez::{
    conf::{WindowMode, WindowSetup},
    event::{self, EventHandler},
    graphics::{Canvas, Color, DrawMode, DrawParam, Mesh, Rect},
    input::keyboard::KeyCode,
    mint::Point2,
    Context, ContextBuilder, GameError, GameResult,
};
use tile::{get_tile_library, tile_definitions::STRAIGHT_ROAD, Tile};

mod tile;
mod util;

const GRID_SIZE: f32 = 0.1;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
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

impl Add<Pos> for Pos {
    type Output = Pos;

    fn add(self, rhs: Pos) -> Self::Output {
        Pos(self.0 + rhs.0, self.1 + rhs.1)
    }
}

struct Game {
    library: Vec<Tile>,
    placed_tiles: HashMap<Pos, Tile>,
    selected_square: Option<Pos>,
    held_tile: Option<Tile>,
    last_selected_square: Option<Pos>,
    placement_is_valid: bool,
}

impl Game {
    fn new() -> Self {
        Self {
            library: get_tile_library(),
            placed_tiles: HashMap::from([(Pos(5, 5), STRAIGHT_ROAD.clone())]),
            selected_square: None,
            held_tile: None,
            last_selected_square: None,
            placement_is_valid: false,
        }
    }

    fn reevaluate_selected_square(&mut self) {
        self.placement_is_valid = false;

        let Some(selected_square) = &self.selected_square else {
            return;
        };

        if self.placed_tiles.contains_key(selected_square) {
            return;
        }

        if let Some(held_tile) = &self.held_tile {
            let mut is_adjacent_tile = false;
            use tile::Orientation::*;
            for (orientation, offset) in [North, East, South, West].into_iter().zip([
                Pos(0, 1),
                Pos(1, 0),
                Pos(0, -1),
                Pos(-1, 0),
            ]) {
                let adjacent_pos = *selected_square + offset;
                let Some(adjacent_tile) = self.placed_tiles.get(&adjacent_pos) else {
                    continue;
                };
                is_adjacent_tile = true;
                if held_tile
                    .validate_mounting(adjacent_tile, orientation)
                    .is_none()
                {
                    return;
                }
            }
            if !is_adjacent_tile {
                return;
            }
        } else {
            return;
        }

        self.placement_is_valid = true;
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
        if self.selected_square != self.last_selected_square {
            self.reevaluate_selected_square();
            self.last_selected_square = self.selected_square;
        }

        if let Some(tile) = &mut self.held_tile {
            if ctx.keyboard.is_key_just_pressed(KeyCode::R) {
                tile.rotate();
                self.reevaluate_selected_square();
            }
        }

        if let Some(tile) = &mut self.held_tile {
            if ctx.mouse.button_just_pressed(event::MouseButton::Left) && self.placement_is_valid {
                self.placed_tiles.insert(grid_pos, tile.clone());
                self.held_tile = None;
                self.reevaluate_selected_square();
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
            let cursor_color = if !self.placement_is_valid {
                Color::RED
            } else {
                Color::GREEN
            };
            if !self.placed_tiles.contains_key(&pos) {
                if let Some(tile) = &self.held_tile {
                    tile.render(ctx, &mut canvas, rect)?;
                }
            }
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

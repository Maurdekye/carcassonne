use game::Game;
use ggez::{
    conf::{WindowMode, WindowSetup},
    event::{self, EventHandler},
    graphics::{Canvas, Color, DrawMode, DrawParam, Mesh},
    input::keyboard::KeyCode,
    Context, ContextBuilder, GameError, GameResult,
};
use pos::Pos;
use tile::{tile_definitions::STRAIGHT_ROAD, Orientation, Tile};

mod game;
pub mod pos;
mod tile;
mod util;

const GRID_SIZE: f32 = 0.1;

struct Client {
    held_tile: Option<Tile>,
    selected_square: Option<Pos>,
    last_selected_square: Option<Pos>,
    placement_is_valid: bool,
    game: Game,
}

impl Client {
    fn new() -> Self {
        Self {
            selected_square: None,
            held_tile: None,
            last_selected_square: None,
            placement_is_valid: false,
            game: Game::new(),
        }
    }

    fn reevaluate_selected_square(&mut self) {
        self.placement_is_valid = false;

        let Some(selected_square) = &self.selected_square else {
            return;
        };

        if self.game.placed_tiles.contains_key(selected_square) {
            return;
        }

        if let Some(held_tile) = &self.held_tile {
            let mut is_adjacent_tile = false;
            for (orientation, offset) in Orientation::iter_with_offsets() {
                let adjacent_pos = *selected_square + offset;
                let Some(adjacent_tile) = self.game.placed_tiles.get(&adjacent_pos) else {
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

impl EventHandler<GameError> for Client {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        let mouse = ctx.mouse.position();
        let grid_pos = Pos::from_screen_pos(mouse, ctx);
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

        if self.held_tile.is_some() {
            if ctx.mouse.button_just_pressed(event::MouseButton::Left) && self.placement_is_valid {
                let tile = self.held_tile.take().unwrap();
                self.game.place_tile(tile, grid_pos)?;
                self.reevaluate_selected_square();
            }
        } else {
            self.held_tile = self.game.library.pop();
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas = Canvas::from_frame(ctx, Color::WHITE);

        for (pos, tile) in &self.game.placed_tiles {
            tile.render(ctx, &mut canvas, pos.rect(ctx))?;
        }

        if let Some(pos) = self.selected_square {
            let rect = pos.rect(ctx);
            let cursor_color = if !self.placement_is_valid {
                Color::RED
            } else {
                Color::GREEN
            };
            if !self.game.placed_tiles.contains_key(&pos) {
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
    let mut client = Client::new();
    client.game.place_tile(STRAIGHT_ROAD.clone(), Pos(5, 5))?;
    event::run(ctx, event_loop, client);
}

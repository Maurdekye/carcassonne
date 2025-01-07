use game::{Game, GroupIdentifier};
use ggez::{
    conf::{WindowMode, WindowSetup},
    event::{self, EventHandler},
    glam::{vec2, Vec2},
    graphics::{Canvas, Color, DrawMode, DrawParam, Mesh, Rect},
    input::keyboard::KeyCode,
    Context, ContextBuilder, GameError, GameResult,
};
use pos::GridPos;
use tile::{tile_definitions::STRAIGHT_ROAD, Orientation, Tile};
use util::point_in_polygon;

mod game;
pub mod pos;
mod tile;
mod util;

const GRID_SIZE: f32 = 0.1;

#[derive(Clone, Copy)]
enum PlacementMode {
    Tile,
    Meeple,
}

struct Client {
    held_tile: Option<Tile>,
    selected_square: Option<GridPos>,
    last_selected_square: Option<GridPos>,
    selected_group: Option<GroupIdentifier>,
    placement_is_valid: bool,
    placement_mode: PlacementMode,
    offset: Vec2,
    game: Game,
}

impl Client {
    fn new() -> Self {
        Self {
            selected_square: None,
            held_tile: None,
            last_selected_square: None,
            selected_group: None,
            placement_is_valid: false,
            placement_mode: PlacementMode::Tile,
            offset: Vec2::ZERO,
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

    pub fn from_screen_pos(&self, screen_pos: Vec2, ctx: &Context) -> GridPos {
        let res: Vec2 = ctx.gfx.drawable_size().into();
        (((screen_pos + self.offset) / res) / GRID_SIZE).into()
    }

    pub fn to_screen_pos(&self, pos: GridPos, ctx: &Context) -> Vec2 {
        let res: Vec2 = ctx.gfx.drawable_size().into();
        let pos: Vec2 = pos.into();
        (pos * GRID_SIZE) * res - self.offset
    }

    pub fn grid_pos_rect(&self, pos: &GridPos, ctx: &Context) -> Rect {
        let res: Vec2 = ctx.gfx.drawable_size().into();
        let dims = res * GRID_SIZE;
        let near_corner = self.to_screen_pos(*pos, ctx);
        Rect::new(near_corner.x, near_corner.y, dims.x, dims.y)
    }
}

impl EventHandler<GameError> for Client {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        let mouse: Vec2 = ctx.mouse.position().into();
        let grid_pos = self.from_screen_pos(mouse, ctx);

        if ctx.keyboard.is_key_just_pressed(KeyCode::Space) {
            use PlacementMode::*;
            self.placement_mode = match self.placement_mode {
                Tile => Meeple,
                Meeple => Tile,
            };
        }

        if ctx.mouse.button_pressed(event::MouseButton::Right) {
            self.offset -= Vec2::from(ctx.mouse.delta());
        }

        match self.placement_mode {
            PlacementMode::Tile => {
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
                    if ctx.mouse.button_just_pressed(event::MouseButton::Left)
                        && self.placement_is_valid
                    {
                        let tile = self.held_tile.take().unwrap();
                        self.game.place_tile(tile, grid_pos)?;
                        self.reevaluate_selected_square();
                    }
                } else {
                    self.held_tile = self.game.library.pop();
                }
            }
            PlacementMode::Meeple => {
                let corner = self.to_screen_pos(grid_pos, ctx);
                let subgrid_pos = Vec2::from(mouse) - Vec2::from(corner);
                let subgrid_pos = subgrid_pos / (Vec2::from(ctx.gfx.drawable_size()) * GRID_SIZE);

                'segment_locate: {
                    if let Some(tile) = self.game.placed_tiles.get(&grid_pos) {
                        for (i, segment) in tile.segments.iter().enumerate() {
                            if point_in_polygon(
                                subgrid_pos,
                                &segment
                                    .poly
                                    .iter()
                                    .map(|i| tile.verts[*i])
                                    .collect::<Vec<_>>(),
                            ) {
                                self.selected_group = Some(
                                    *self.game.group_associations.get(&(grid_pos, i)).unwrap(),
                                );
                                break 'segment_locate;
                            }
                        }
                    }
                    self.selected_group = None;
                }
            }
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas = Canvas::from_frame(ctx, Color::WHITE);

        for (pos, tile) in &self.game.placed_tiles {
            tile.render(ctx, &mut canvas, self.grid_pos_rect(pos, ctx))?;
        }

        ctx.gfx
            .set_window_title(&format!("Carcassone: {:.2} fps", ctx.time.fps()));

        match self.placement_mode {
            PlacementMode::Tile => {
                if let Some(pos) = self.selected_square {
                    let rect = self.grid_pos_rect(&pos, ctx);
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
            }
            PlacementMode::Meeple => {
                if let Some(group) = self
                    .selected_group
                    .and_then(|key| self.game.groups.get(key))
                {
                    for (pos, tile, i) in group.segments.iter().filter_map(|(pos, i)| {
                        self.game.placed_tiles.get(pos).map(|tile| (pos, tile, i))
                    }) {
                        let rect = self.grid_pos_rect(pos, ctx);
                        tile.render_segment(
                            *i,
                            ctx,
                            &mut canvas,
                            rect,
                            Some(Color::from_rgb(200, 20, 70)),
                        )?;
                    }

                    for &(pos, orientation) in &group.free_edges {
                        let rect = self.grid_pos_rect(&pos, ctx);
                        let tl = vec2(rect.x, rect.y);
                        let tr = vec2(rect.right(), rect.y);
                        let bl = vec2(rect.x, rect.bottom());
                        let br = vec2(rect.right(), rect.bottom());
                        let line = match orientation {
                            Orientation::North => [tl, tr],
                            Orientation::East => [tr, br],
                            Orientation::South => [br, bl],
                            Orientation::West => [bl, tl],
                        };
                        canvas.draw(
                            &Mesh::new_line(ctx, &line, 2.0, Color::CYAN)?,
                            DrawParam::default(),
                        );
                    }
                }
            }
        }

        canvas.finish(ctx)
    }
}

fn main() -> GameResult {
    let (ctx, event_loop) = ContextBuilder::new("carcassone", "maurdekye")
        .window_mode(WindowMode::default().dimensions(800.0, 800.0))
        .window_setup(WindowSetup::default().title("Carcassone"))
        .build()?;
    let mut client = Client::new();
    client
        .game
        .place_tile(STRAIGHT_ROAD.clone(), GridPos(5, 5))?;
    // client
    //     .game
    //     .place_tile(DEAD_END_ROAD.clone().rotated(), Pos(7, 5))?;
    event::run(ctx, event_loop, client);
}

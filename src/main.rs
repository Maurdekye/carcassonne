use game::{player::Player, Game, GroupIdentifier, PlayerIdentifier, SegmentIdentifier};
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
use util::{point_in_polygon, refit_to_rect};

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
    selected_segment: Option<SegmentIdentifier>,
    selected_group: Option<GroupIdentifier>,
    placement_is_valid: bool,
    placement_mode: PlacementMode,
    active_player: PlayerIdentifier,
    offset: Vec2,
    game: Game,
}

impl Client {
    fn new() -> Self {
        let mut game = Game::new();
        let active_player = game.players.insert(Player::new(Color::GREEN));
        Self {
            selected_square: None,
            held_tile: None,
            last_selected_square: None,
            selected_group: None,
            selected_segment: None,
            placement_is_valid: false,
            placement_mode: PlacementMode::Tile,
            offset: Vec2::ZERO,
            active_player,
            game,
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

    pub fn from_screen_pos(&self, screen_pos: Vec2, ctx: &Context) -> Vec2 {
        let res: Vec2 = ctx.gfx.drawable_size().into();
        ((screen_pos + self.offset) / res) / GRID_SIZE
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

    pub fn draw_meeple(
        &self,
        ctx: &Context,
        canvas: &mut Canvas,
        pos: Vec2,
        color: Color,
    ) -> Result<(), GameError> {
        const MEEPLE_CENTER: Vec2 = vec2(0.5, 0.6);
        const MEEPLE_POINTS: [Vec2; 13] = [
            vec2(0.025, 1.0),
            vec2(0.425, 1.0),
            vec2(0.5, 0.85),
            vec2(0.575, 1.0),
            vec2(0.975, 1.0),
            vec2(0.75, 0.575),
            vec2(1.0, 0.475),
            vec2(1.0, 0.35),
            vec2(0.675, 0.3),
            vec2(0.325, 0.3),
            vec2(0.0, 0.35),
            vec2(0.0, 0.475),
            vec2(0.25, 0.575),
        ];
        const HEAD_POINT: Vec2 = vec2(0.5, 0.3);
        let meeple_points = MEEPLE_POINTS.map(|p| (p - MEEPLE_CENTER) * GRID_SIZE - pos);
        let head_point = (HEAD_POINT - MEEPLE_CENTER) * GRID_SIZE - pos;
        canvas.draw(
            &Mesh::new_polygon(ctx, DrawMode::fill(), &meeple_points, color)?,
            DrawParam::default(),
        );
        canvas.draw(
            &Mesh::new_circle(
                ctx,
                DrawMode::fill(),
                head_point,
                GRID_SIZE * 0.175,
                1.0,
                color,
            )?,
            DrawParam::default(),
        );
        Ok(())
    }
}

impl EventHandler<GameError> for Client {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        let mouse: Vec2 = ctx.mouse.position().into();
        let grid_pos: GridPos = self.from_screen_pos(mouse, ctx).into();

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
                                self.selected_segment = Some((grid_pos, i));
                                break 'segment_locate;
                            }
                        }
                    }
                    self.selected_group = None;
                    self.selected_segment = None;
                }

                if let (Some(seg_ident), Some(group)) = (
                    self.selected_segment,
                    self.selected_group
                        .and_then(|group_ident| self.game.groups.get(group_ident)),
                ) {
                    let player = self.game.players.get(self.active_player).unwrap();
                    if ctx.mouse.button_just_pressed(event::MouseButton::Left) {
                        if !group.meeples.is_empty() && player.meeples > 0 {
                            self.game.place_meeple(seg_ident, self.active_player)?;
                        }
                    }
                }
            }
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas = Canvas::from_frame(ctx, Color::WHITE);

        // draw tiles
        for (pos, tile) in &self.game.placed_tiles {
            tile.render(ctx, &mut canvas, self.grid_pos_rect(pos, ctx))?;
        }

        // draw meeples
        // this is really slow and needs to be optimized with some memoization probably
        for &(seg_ident, player) in self.game.groups.values().flat_map(|group| &group.meeples) {
            let color = self.game.players.get(player).unwrap().color;
            let (pos, seg_index) = seg_ident;
            let tile = self.game.placed_tiles.get(&pos).unwrap();
            let rect = self.grid_pos_rect(&pos, ctx);
            let segment_center = tile.segments[seg_index]
                .poly
                .iter()
                .map(|i| refit_to_rect(tile.verts[*i], rect))
                .reduce(|a, b| a + b)
                .unwrap()
                / tile.segments[seg_index].poly.len() as f32;
            self.draw_meeple(ctx, &mut canvas, segment_center, color)?;
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

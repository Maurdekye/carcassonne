use std::collections::VecDeque;

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

#[derive(Clone)]
enum TurnPhase {
    TilePlacement(Tile),
    MeeplePlacement {
        placed_position: GridPos,
        closed_groups: Vec<GroupIdentifier>,
    },
    EndGame,
}

struct Client {
    selected_square: Option<GridPos>,
    last_selected_square: Option<GridPos>,
    selected_segment: Option<SegmentIdentifier>,
    selected_group: Option<GroupIdentifier>,
    placement_is_valid: bool,
    turn_phase: TurnPhase,
    turn_order: VecDeque<PlayerIdentifier>,
    offset: Vec2,
    game: Game,
}

impl Client {
    fn new() -> Self {
        let mut game = Game::new();
        game.players.insert(Player::new(Color::BLUE));
        game.players.insert(Player::new(Color::YELLOW));
        let first_tile = game.library.pop().unwrap();
        Self {
            selected_square: None,
            last_selected_square: None,
            selected_group: None,
            selected_segment: None,
            placement_is_valid: false,
            turn_phase: TurnPhase::TilePlacement(first_tile),
            offset: Vec2::ZERO,
            turn_order: game.players.keys().collect(),
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

        if let TurnPhase::TilePlacement(held_tile) = &self.turn_phase {
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
        const MEEPLE_SIZE: f32 = 200.0;
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
        let meeple_points =
            MEEPLE_POINTS.map(|p| (p - MEEPLE_CENTER) * MEEPLE_SIZE * GRID_SIZE + pos);
        let head_point = (HEAD_POINT - MEEPLE_CENTER) * MEEPLE_SIZE * GRID_SIZE + pos;
        canvas.draw(
            &Mesh::new_polygon(ctx, DrawMode::fill(), &meeple_points, color)?,
            DrawParam::default(),
        );
        canvas.draw(
            &Mesh::new_circle(
                ctx,
                DrawMode::fill(),
                head_point,
                GRID_SIZE * MEEPLE_SIZE * 0.175,
                1.0,
                color,
            )?,
            DrawParam::default(),
        );
        Ok(())
    }

    fn get_held_tile_mut(&mut self) -> Option<&mut Tile> {
        match &mut self.turn_phase {
            TurnPhase::TilePlacement(tile) => Some(tile),
            _ => None,
        }
    }
}

impl EventHandler<GameError> for Client {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        let mouse: Vec2 = ctx.mouse.position().into();
        let focused_pos: GridPos = self.from_screen_pos(mouse, ctx).into();

        if ctx.mouse.button_pressed(event::MouseButton::Right) {
            self.offset -= Vec2::from(ctx.mouse.delta());
        }

        match &self.turn_phase {
            TurnPhase::TilePlacement(_) => {
                self.selected_square = Some(focused_pos);

                if self.selected_square != self.last_selected_square {
                    self.reevaluate_selected_square();
                    self.last_selected_square = self.selected_square;
                }

                if ctx.mouse.button_just_pressed(event::MouseButton::Left)
                    && self.placement_is_valid
                {
                    let tile = self.get_held_tile_mut().unwrap().clone();
                    let closed_groups = self.game.place_tile(tile, focused_pos)?;
                    self.reevaluate_selected_square();
                    self.turn_phase = TurnPhase::MeeplePlacement {
                        placed_position: focused_pos,
                        closed_groups,
                    };
                }

                if ctx.keyboard.is_key_just_pressed(KeyCode::R) {
                    self.get_held_tile_mut().unwrap().rotate();
                    self.reevaluate_selected_square();
                }
            }
            TurnPhase::MeeplePlacement {
                placed_position,
                closed_groups,
            } => {
                self.selected_group = None;
                self.selected_segment = None;

                if *placed_position == focused_pos {
                    let corner: GridPos = self.to_screen_pos(focused_pos, ctx).into();
                    let subgrid_pos = mouse - Vec2::from(corner);
                    let subgrid_pos =
                        subgrid_pos / (Vec2::from(ctx.gfx.drawable_size()) * GRID_SIZE);

                    'segment_locate: {
                        if let Some(tile) = self.game.placed_tiles.get(&focused_pos) {
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
                                        *self
                                            .game
                                            .group_associations
                                            .get(&(focused_pos, i))
                                            .unwrap(),
                                    );
                                    self.selected_segment = Some((focused_pos, i));
                                    break 'segment_locate;
                                }
                            }
                        }
                    }

                    if ctx.mouse.button_just_pressed(event::MouseButton::Left) {
                        if let (Some(seg_ident), Some(group)) = (
                            self.selected_segment,
                            self.selected_group
                                .and_then(|group_ident| self.game.groups.get(group_ident)),
                        ) {
                            let player_ident = *self.turn_order.front().unwrap();
                            let player = self.game.players.get(player_ident).unwrap();
                            if group.meeples.is_empty() && player.meeples > 0 {
                                // place meeple and advance turn
                                self.game.place_meeple(seg_ident, player_ident)?;

                                for group_ident in closed_groups {
                                    self.game.score_group(*group_ident);
                                }

                                let player_ident = self.turn_order.pop_front().unwrap();
                                self.turn_order.push_back(player_ident);

                                self.turn_phase = match self.game.library.pop() {
                                    Some(tile) => TurnPhase::TilePlacement(tile),
                                    None => TurnPhase::EndGame,
                                }
                            }
                        }
                    }
                }
            }
            TurnPhase::EndGame => {}
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        ctx.gfx
            .set_window_title(&format!("Carcassone: {:.2} fps", ctx.time.fps()));

        let mut canvas = Canvas::from_frame(ctx, Color::WHITE);

        let time = ctx.time.time_since_start().as_secs_f32();

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

        match &self.turn_phase {
            TurnPhase::TilePlacement(tile) => {
                if let Some(pos) = self.selected_square {
                    let rect = self.grid_pos_rect(&pos, ctx);
                    let cursor_color = if !self.placement_is_valid {
                        Color::RED
                    } else {
                        Color::GREEN
                    };
                    if !self.game.placed_tiles.contains_key(&pos) {
                        tile.render(ctx, &mut canvas, rect)?;
                    }
                    canvas.draw(
                        &Mesh::new_rectangle(ctx, DrawMode::stroke(2.0), rect, cursor_color)?,
                        DrawParam::default(),
                    )
                }
            }
            TurnPhase::MeeplePlacement {
                placed_position, ..
            } => {
                let rect = self.grid_pos_rect(&placed_position, ctx);
                canvas.draw(
                    &Mesh::new_rectangle(ctx, DrawMode::stroke(2.0), rect, Color::CYAN)?,
                    DrawParam::default(),
                );

                if let Some((tile_pos, seg_index)) = self.selected_segment {
                    let tile = self.game.placed_tiles.get(&tile_pos).unwrap();
                    let sin_time = time.sin() * 0.1 + 1.0;
                    tile.render_segment(
                        seg_index,
                        ctx,
                        &mut canvas,
                        rect,
                        Some(Color::from_rgb(
                            (200.0 * sin_time) as u8,
                            (20.0 * sin_time) as u8,
                            (70.0 * sin_time) as u8,
                        )),
                    )?;
                }

                // if let Some(group) = self
                //     .selected_group
                //     .and_then(|key| self.game.groups.get(key))
                // {
                //     for (pos, tile, i) in group.segments.iter().filter_map(|(pos, i)| {
                //         self.game.placed_tiles.get(pos).map(|tile| (pos, tile, i))
                //     }) {
                //         let rect = self.grid_pos_rect(pos, ctx);
                //         tile.render_segment(
                //             *i,
                //             ctx,
                //             &mut canvas,
                //             rect,
                //             Some(Color::from_rgb(200, 20, 70)),
                //         )?;
                //     }

                //     for &(pos, orientation) in &group.free_edges {
                //         let rect = self.grid_pos_rect(&pos, ctx);
                //         let tl = vec2(rect.x, rect.y);
                //         let tr = vec2(rect.right(), rect.y);
                //         let bl = vec2(rect.x, rect.bottom());
                //         let br = vec2(rect.right(), rect.bottom());
                //         let line = match orientation {
                //             Orientation::North => [tl, tr],
                //             Orientation::East => [tr, br],
                //             Orientation::South => [br, bl],
                //             Orientation::West => [bl, tl],
                //         };
                //         canvas.draw(
                //             &Mesh::new_line(ctx, &line, 2.0, Color::CYAN)?,
                //             DrawParam::default(),
                //         );
                //     }
                // }
            }
            TurnPhase::EndGame => {}
        }

        // draw current player
        let current_player_ident = *self.turn_order.front().unwrap();
        let current_player = self.game.players.get(current_player_ident).unwrap();

        canvas.draw(
            &Mesh::new_rounded_rectangle(
                ctx,
                DrawMode::fill(),
                Rect::new(10.0, 10.0, 40.0, 40.0),
                5.0,
                current_player.color,
            )?,
            DrawParam::default(),
        );

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

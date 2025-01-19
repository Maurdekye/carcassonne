#![feature(iter_map_windows)]
use std::{collections::VecDeque, sync::Mutex};

use clap::{arg, Parser};
use game::{player::Player, Game, GroupIdentifier, PlayerIdentifier, SegmentIdentifier};
use ggez::{
    conf::{WindowMode, WindowSetup},
    event::{self, EventHandler},
    glam::{vec2, Vec2, Vec2Swizzles},
    graphics::{Canvas, Color, DrawMode, DrawParam, Mesh, Rect, Text},
    input::keyboard::KeyCode,
    Context, ContextBuilder, GameError, GameResult,
};
use pos::GridPos;
use rand::{rngs::StdRng, seq::SliceRandom, thread_rng, SeedableRng};
use tile::{
    tile_definitions::{
        CORNER_CITY_CURVE_ROAD, FOUR_WAY_CROSSROADS, FULL_FORTIFIED_CITY, STARTING_TILE,
    },
    Orientation, Tile,
};
use util::{point_in_polygon, refit_to_rect};

mod game;
pub mod pos;
mod tile;
mod util;

const ZOOM_SPEED: f32 = 1.1;

const SCORE_EFFECT_LIFE: f32 = 2.5;
const SCORE_EFFECT_DISTANCE: f32 = 0.4;
const SCORE_EFFECT_DECCEL: f32 = 15.0;

#[derive(Clone)]
enum TurnPhase {
    TilePlacement(Tile),
    MeeplePlacement {
        placed_position: GridPos,
        closed_groups: Vec<GroupIdentifier>,
    },
    EndGame,
}

#[derive(Debug)]
struct ScoringEffect {
    position: Vec2,
    score: usize,
    color: Color,
    initialized_at: f32,
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
    scale: f32,
    skip_meeple_button: Rect,
    scoring_effects: Vec<ScoringEffect>,
    state_update_lock: Mutex<()>,
    game: Game,
}

impl Client {
    fn new(ctx: &Context, players: usize) -> Self {
        let mut game = Game::new();
        game.library.shuffle(&mut thread_rng());
        for color in [
            Color::RED,
            Color::YELLOW,
            Color::BLUE,
            Color::GREEN,
            Color::BLACK,
        ]
        .into_iter()
        .take(players)
        {
            game.players.insert(Player::new(color));
        }
        Self::new_with_game(ctx, game)
    }

    fn new_with_game(ctx: &Context, mut game: Game) -> Self {
        let first_tile = game.library.pop().unwrap();
        let mut this = Self {
            selected_square: None,
            last_selected_square: None,
            selected_group: None,
            selected_segment: None,
            placement_is_valid: false,
            turn_phase: TurnPhase::TilePlacement(first_tile),
            offset: Vec2::ZERO,
            scale: 1.0,
            turn_order: game.players.keys().collect(),
            skip_meeple_button: Rect::new(0.0, 20.0, 120.0, 40.0),
            scoring_effects: Vec::new(),
            state_update_lock: Mutex::new(()),
            game,
        };
        this.reset_navigation(ctx);
        this
    }

    fn reset_navigation(&mut self, ctx: &Context) {
        self.scale = 0.1;
        self.offset = -Vec2::from(ctx.gfx.drawable_size()) * Vec2::splat(0.5 - self.scale / 2.0);
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

    #[inline]
    pub fn norm(&self, ctx: &Context) -> Vec2 {
        let res: Vec2 = ctx.gfx.drawable_size().into();
        ((res / res.yx()).max(Vec2::ONE) / res) / self.scale
    }

    pub fn to_game_pos(&self, screen_pos: Vec2, ctx: &Context) -> Vec2 {
        (screen_pos + self.offset) * self.norm(ctx)
    }

    pub fn to_screen_pos(&self, game_pos: Vec2, ctx: &Context) -> Vec2 {
        (game_pos / self.norm(ctx)) - self.offset
    }

    pub fn grid_pos_rect(&self, pos: &GridPos, ctx: &Context) -> Rect {
        let res: Vec2 = ctx.gfx.drawable_size().into();
        let dims = (res * self.scale) / (res / res.yx()).max(Vec2::ONE);
        let near_corner = self.to_screen_pos((*pos).into(), ctx);
        Rect::new(near_corner.x, near_corner.y, dims.x, dims.y)
    }

    pub fn draw_meeple(
        &self,
        ctx: &Context,
        canvas: &mut Canvas,
        pos: Vec2,
        color: Color,
        scale: Option<f32>,
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
        let scale = scale.unwrap_or(self.scale) * MEEPLE_SIZE;
        let meeple_points = MEEPLE_POINTS.map(|p| (p - MEEPLE_CENTER) * scale + pos);
        let head_point = (HEAD_POINT - MEEPLE_CENTER) * scale + pos;
        canvas.draw(
            &Mesh::new_polygon(ctx, DrawMode::fill(), &meeple_points, color)?,
            DrawParam::default(),
        );
        canvas.draw(
            &Mesh::new_circle(ctx, DrawMode::fill(), head_point, scale * 0.175, 1.0, color)?,
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

    fn render_player_card(
        &self,
        ctx: &Context,
        canvas: &mut Canvas,
        player_ident: PlayerIdentifier,
        pos: Vec2,
        highlighted: bool,
    ) -> Result<(), GameError> {
        let player = self.game.players.get(player_ident).unwrap();
        let card_rect = Rect {
            x: pos.x,
            y: pos.y,
            w: 160.0,
            h: 60.0,
        };
        canvas.draw(
            &Mesh::new_rounded_rectangle(
                ctx,
                DrawMode::fill(),
                card_rect,
                5.0,
                Color::from_rgb(192, 192, 192),
            )?,
            DrawParam::default(),
        );
        canvas.draw(
            &Text::new(format!("Score: {}", player.score)),
            DrawParam::from(pos + vec2(10.0, 10.0)).color(Color::BLACK),
        );
        for i in 0..player.meeples {
            self.draw_meeple(
                ctx,
                canvas,
                pos + vec2(20.0, 40.0) + vec2(20.0, 0.0) * i as f32,
                player.color,
                Some(0.1),
            )?;
        }
        if highlighted {
            canvas.draw(
                &Mesh::new_rounded_rectangle(
                    ctx,
                    DrawMode::stroke(4.0),
                    card_rect,
                    5.0,
                    player.color,
                )?,
                DrawParam::default(),
            );
        }

        Ok(())
    }

    fn end_turn(&mut self, ctx: &Context, groups_to_close: Vec<GroupIdentifier>) {
        for group_ident in groups_to_close {
            use tile::SegmentType::*;
            let group = self.game.groups.get(group_ident).unwrap();
            match group.gtype {
                City | Road | Monastary => {
                    let scored_meeples = self.game.score_group(group_ident);
                    self.scoring_effects
                        .extend(
                            scored_meeples
                                .into_iter()
                                .map(|score_result| ScoringEffect {
                                    position: score_result.meeple_location,
                                    score: score_result.score,
                                    color: score_result.meeple_color,
                                    initialized_at: ctx.time.time_since_start().as_secs_f32(),
                                }),
                        );
                }
                _ => {}
            }
        }

        let player_ident = self.turn_order.pop_front().unwrap();
        self.turn_order.push_back(player_ident);

        self.turn_phase = match self.game.library.pop() {
            Some(tile) => TurnPhase::TilePlacement(tile),
            None => {
                self.end_game(ctx);
                return;
            }
        }
    }

    fn end_game(&mut self, ctx: &Context) {
        for group_ident in self.game.groups.keys().collect::<Vec<_>>() {
            let group = self.game.groups.get(group_ident).unwrap();
            if !group.meeples.is_empty() {
                let scored_meeples = self.game.score_group(group_ident);
                self.scoring_effects
                    .extend(
                        scored_meeples
                            .into_iter()
                            .map(|score_result| ScoringEffect {
                                position: score_result.meeple_location,
                                score: score_result.score,
                                color: score_result.meeple_color,
                                initialized_at: ctx.time.time_since_start().as_secs_f32(),
                            }),
                    );
            }
        }
        self.turn_phase = TurnPhase::EndGame;
    }
}

impl EventHandler<GameError> for Client {
    fn mouse_wheel_event(&mut self, ctx: &mut Context, _x: f32, y: f32) -> Result<(), GameError> {
        // zooming
        let prev_scale = self.scale;
        self.scale *= ZOOM_SPEED.powf(y);
        self.scale = self.scale.clamp(0.01, 1.0);
        let scale_change = self.scale / prev_scale;
        let mouse: Vec2 = ctx.mouse.position().into();
        self.offset = (self.offset + mouse) * scale_change - mouse;

        Ok(())
    }

    fn update(&mut self, ctx: &mut Context) -> GameResult {
        let mouse: Vec2 = ctx.mouse.position().into();
        let res: Vec2 = ctx.gfx.drawable_size().into();
        let focused_pos: GridPos = self.to_game_pos(mouse, ctx).into();

        // dragging
        if ctx.mouse.button_pressed(event::MouseButton::Right) {
            self.offset -= Vec2::from(ctx.mouse.delta());
        }

        // reset
        if ctx.keyboard.is_key_just_pressed(KeyCode::Space) {
            self.reset_navigation(ctx);
        }

        // update scoring effects
        self.scoring_effects.retain(|effect| {
            ctx.time.time_since_start().as_secs_f32() - effect.initialized_at < SCORE_EFFECT_LIFE
        });

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
                    let tile = self.game.placed_tiles.get(&focused_pos).unwrap();

                    if (0..tile.segments.len())
                        .filter_map(|i| {
                            let (group, _) =
                                self.game.group_and_key_by_seg_ident((focused_pos, i))?;
                            Some(!group.meeples.is_empty())
                        })
                        .all(|x| x)
                    {
                        self.end_turn(ctx, closed_groups.clone());
                    } else {
                        self.turn_phase = TurnPhase::MeeplePlacement {
                            placed_position: focused_pos,
                            closed_groups,
                        };
                    }
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

                self.skip_meeple_button.x = res.x - self.skip_meeple_button.w - 20.0;

                let player_ident = *self.turn_order.front().unwrap();
                let player = self.game.players.get(player_ident).unwrap();
                if player.meeples == 0
                    || (self.skip_meeple_button.contains(mouse)
                        && ctx.mouse.button_just_pressed(event::MouseButton::Left))
                {
                    self.end_turn(ctx, closed_groups.clone());
                } else if *placed_position == focused_pos {
                    let subgrid_pos = self.to_game_pos(mouse, ctx) - Vec2::from(focused_pos);
                    'segment_locate: {
                        if let Some(tile) = self.game.placed_tiles.get(&focused_pos) {
                            for (i, _) in tile.segments.iter().enumerate() {
                                let (group, group_ident) = self
                                    .game
                                    .group_and_key_by_seg_ident((*placed_position, i))
                                    .unwrap();
                                if group.gtype.placeable()
                                    && group.meeples.is_empty()
                                    && point_in_polygon(
                                        subgrid_pos,
                                        &tile.segment_polygon(i).collect::<Vec<_>>(),
                                    )
                                {
                                    let _lock = self.state_update_lock.lock().unwrap();
                                    self.selected_group = Some(group_ident);
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
                                self.end_turn(ctx, closed_groups.clone());
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
        let lock = self.state_update_lock.lock().unwrap();

        ctx.gfx
            .set_window_title(&format!("Carcassone: {:.2} fps", ctx.time.fps()));
        let mouse: Vec2 = ctx.mouse.position().into();

        let mut canvas = Canvas::from_frame(ctx, Color::WHITE);

        let time = ctx.time.time_since_start().as_secs_f32();
        let sin_time = time.sin() * 0.1 + 1.0;
        let origin_rect = self.grid_pos_rect(&GridPos(0, 0), ctx);

        // draw tiles
        for (pos, tile) in &self.game.placed_tiles {
            tile.render(ctx, &mut canvas, self.grid_pos_rect(pos, ctx))?;
        }

        // draw meeples
        for &(seg_ident, player) in self.game.groups.values().flat_map(|group| &group.meeples) {
            let color = self.game.players.get(player).unwrap().color;
            let (pos, seg_index) = seg_ident;
            let tile = self.game.placed_tiles.get(&pos).unwrap();
            let rect = self.grid_pos_rect(&pos, ctx);
            let segment_meeple_spot = refit_to_rect(tile.segments[seg_index].meeple_spot, rect);
            self.draw_meeple(ctx, &mut canvas, segment_meeple_spot, color, None)?;
        }

        // draw score effects
        for effect in &self.scoring_effects {
            let lifetime = (time - effect.initialized_at) / SCORE_EFFECT_LIFE;
            let alpha = (1.0 - lifetime).max(0.0) * 255.0;
            let y_shift = ((-((lifetime * SCORE_EFFECT_DECCEL) / SCORE_EFFECT_LIFE)).exp() - 1.0)
                * SCORE_EFFECT_DISTANCE;
            let pos = self.to_screen_pos(effect.position + y_shift * Vec2::Y, ctx);
            let mut color = effect.color;
            color.a = alpha;
            let mut text = Text::new(format!(" +{} ", effect.score));
            text.set_scale(20.0);
            let size: Vec2 = text.measure(ctx)?.into();
            canvas.draw(&text, DrawParam::from(pos - size / 2.0).color(color));
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
                let rect = self.grid_pos_rect(placed_position, ctx);
                canvas.draw(
                    &Mesh::new_rectangle(ctx, DrawMode::stroke(2.0), rect, Color::CYAN)?,
                    DrawParam::default(),
                );

                let on_ui = self.skip_meeple_button.contains(mouse);

                if !on_ui {
                    if let Some(group_ident) = self.selected_group {
                        let outline = self.game.get_group_outline(group_ident);
                        for polyline in outline.iter().map(|polyline| {
                            polyline
                                .iter()
                                .map(|vert| refit_to_rect(*vert, origin_rect))
                                .collect::<Vec<_>>()
                        }) {
                            canvas.draw(
                                &Mesh::new_line(
                                    ctx,
                                    &polyline,
                                    2.0,
                                    Color::from_rgb(
                                        (200.0 * sin_time) as u8,
                                        (20.0 * sin_time) as u8,
                                        (70.0 * sin_time) as u8,
                                    ),
                                )?,
                                DrawParam::default(),
                            );
                        }
                    }
                }

                // draw skip meeples button
                canvas.draw(
                    &Mesh::new_rounded_rectangle(
                        ctx,
                        DrawMode::fill(),
                        self.skip_meeple_button,
                        4.0,
                        Color::from_rgb(0, 128, 192),
                    )?,
                    DrawParam::default(),
                );
                let Rect { x, y, .. } = self.skip_meeple_button;
                canvas.draw(
                    &Text::new("Skip meeples"),
                    DrawParam::from(vec2(x, y) + vec2(10.0, 10.0)).color(Color::BLACK),
                );
            }
            TurnPhase::EndGame => {}
        }

        // draw ui

        let current_player_ident = *self.turn_order.front().unwrap();
        if ctx.keyboard.is_key_pressed(KeyCode::Tab) {
            for (i, &player_ident) in self.turn_order.iter().enumerate() {
                self.render_player_card(
                    ctx,
                    &mut canvas,
                    player_ident,
                    vec2(20.0, 20.0) + vec2(0.0, 80.0) * i as f32,
                    player_ident == current_player_ident,
                )?;
            }
        } else {
            self.render_player_card(
                ctx,
                &mut canvas,
                current_player_ident,
                vec2(20.0, 20.0),
                false,
            )?;
        }

        drop(lock);
        canvas.finish(ctx)
    }
}

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

    let mut client = Client::new(&ctx, args.players);
    client
        .game
        .place_tile(STARTING_TILE.clone(), GridPos(0, 0))?;

    event::run(ctx, event_loop, client);
}

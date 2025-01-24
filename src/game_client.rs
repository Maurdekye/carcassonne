use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;
use std::sync::mpsc::{channel, Receiver, Sender};

use crate::game::{
    player::Player, Game, GroupIdentifier, PlayerIdentifier, ScoringResult, SegmentIdentifier,
};
use crate::main_client::MainEvent;
use crate::pos::GridPos;
use crate::sub_event_handler::SubEventHandler;
use crate::tile::{tile_definitions::STARTING_TILE, Tile};
use crate::ui_manager::{Button, ButtonBounds, ButtonState, UIManager};
use crate::util::{point_in_polygon, refit_to_rect, DrawableWihParamsExt, TextExt};

use ggez::input::keyboard::KeyCode;
use ggez::input::mouse::{set_cursor_type, CursorIcon};
use ggez::{
    event::{self, EventHandler},
    glam::{vec2, Vec2, Vec2Swizzles},
    graphics::{Canvas, Color, DrawMode, DrawParam, Mesh, Rect, Text},
    Context, GameError, GameResult,
};
use pause_screen_subclient::PauseScreenSubclient;
use rand::{seq::SliceRandom, thread_rng};

mod pause_screen_subclient;

const ZOOM_SPEED: f32 = 1.1;

const SCORE_EFFECT_LIFE: f32 = 2.5;
const SCORE_EFFECT_DISTANCE: f32 = 0.4;
const SCORE_EFFECT_DECCEL: f32 = 15.0;

const END_GAME_SCORE_DELAY: f32 = 3.0;
const END_GAME_SCORE_INTERVAL: f32 = 1.75;

pub const NUM_PLAYERS: usize = 5;
pub const PLAYER_COLORS: [Color; NUM_PLAYERS] = [
    Color::RED,
    Color::YELLOW,
    Color::BLUE,
    Color::GREEN,
    Color::BLACK,
];

#[derive(Debug, Clone)]
enum GameEvent {
    MainEvent(MainEvent),
    SkipMeeples,
    ClosePauseMenu,
    EndGame,
    ResetCamera,
    Undo,
}

#[allow(clippy::large_enum_variant)]
#[derive(Clone)]
enum TurnPhase {
    TilePlacement {
        tile: Tile,
    },
    MeeplePlacement {
        placed_position: GridPos,
        closed_groups: Vec<GroupIdentifier>,
    },
    EndGame {
        next_tick: Option<f32>,
    },
}

#[derive(Debug)]
struct ScoringEffect {
    position: Vec2,
    score: usize,
    color: Color,
    initialized_at: f32,
}

impl ScoringEffect {
    fn from_scoring_result(ctx: &Context, score_result: ScoringResult) -> Self {
        ScoringEffect {
            position: score_result.meeple_location,
            score: score_result.score,
            color: score_result.meeple_color,
            initialized_at: ctx.time.time_since_start().as_secs_f32(),
        }
    }
}

#[derive(Clone)]
pub struct GameState {
    game: Game,
    turn_phase: TurnPhase,
    turn_order: VecDeque<PlayerIdentifier>,
}

pub struct GameClient {
    parent_channel: Sender<MainEvent>,
    event_sender: Sender<GameEvent>,
    event_receiver: Receiver<GameEvent>,
    pause_menu: Option<PauseScreenSubclient>,
    selected_square: Option<GridPos>,
    last_selected_square: Option<GridPos>,
    selected_segment_and_group: Option<(SegmentIdentifier, GroupIdentifier)>,
    placement_is_valid: bool,
    offset: Vec2,
    scale: f32,
    scoring_effects: Vec<ScoringEffect>,
    history: Vec<GameState>,
    skip_meeples_button: Rc<RefCell<Button<GameEvent>>>,
    return_to_main_menu_button: Rc<RefCell<Button<GameEvent>>>,
    state: GameState,
    ui: UIManager<GameEvent>,
}

impl GameClient {
    pub fn new(ctx: &Context, players: Vec<Color>, parent_channel: Sender<MainEvent>) -> Self {
        let mut game = Game::new();
        game.library.shuffle(&mut thread_rng());
        for color in players {
            game.players.insert(Player::new(color));
        }
        game.place_tile(STARTING_TILE.clone(), GridPos(0, 0))
            .unwrap();
        Self::new_with_game(ctx, game, parent_channel)
    }

    pub fn new_with_game(ctx: &Context, mut game: Game, parent_channel: Sender<MainEvent>) -> Self {
        let first_tile = game.draw_placeable_tile().unwrap();
        let (event_sender, event_receiver) = channel();
        let ui_sender = event_sender.clone();
        let (ui, [skip_meeples_button, return_to_main_menu_button]) = UIManager::new_and_rc_buttons(
            ui_sender,
            [
                Button::new_with_styling(
                    ButtonBounds {
                        relative: Rect::new(1.0, 0.0, 0.0, 0.0),
                        absolute: Rect::new(-180.0, 20.0, 160.0, 40.0),
                    },
                    Text::new("Skip meeples"),
                    DrawParam::default(),
                    Color::from_rgb(0, 128, 192),
                    GameEvent::SkipMeeples,
                ),
                Button::new(
                    ButtonBounds {
                        relative: Rect::new(1.0, 0.0, 0.0, 0.0),
                        absolute: Rect::new(-260.0, 20.0, 240.0, 40.0),
                    },
                    Text::new("Return to Main Menu"),
                    GameEvent::MainEvent(MainEvent::ReturnToMainMenu),
                ),
            ],
        );
        let mut this = Self {
            parent_channel,
            event_sender,
            event_receiver,
            pause_menu: None,
            selected_square: None,
            last_selected_square: None,
            selected_segment_and_group: None,
            placement_is_valid: false,
            offset: Vec2::ZERO,
            scale: 1.0,
            scoring_effects: Vec::new(),
            history: Vec::new(),
            state: GameState {
                turn_phase: TurnPhase::TilePlacement { tile: first_tile },
                turn_order: game.players.keys().collect(),
                game,
            },
            skip_meeples_button,
            return_to_main_menu_button,
            ui,
        };
        this.reset_camera(ctx);
        this
    }

    fn push_history(&mut self) {
        self.history.push(self.state.clone());
    }

    fn pop_history(&mut self) {
        if let Some(state) = self.history.pop() {
            self.state = state;
        }
    }

    fn reset_camera(&mut self, ctx: &Context) {
        self.scale = 0.1;
        self.offset = -Vec2::from(ctx.gfx.drawable_size()) * Vec2::splat(0.5 - self.scale / 2.0);
    }

    fn reevaluate_selected_square(&mut self) {
        self.placement_is_valid = false;

        let Some(selected_square) = &self.selected_square else {
            return;
        };

        if self.state.game.placed_tiles.contains_key(selected_square) {
            return;
        }

        if let TurnPhase::TilePlacement { tile: held_tile } = &self.state.turn_phase {
            self.placement_is_valid = self
                .state
                .game
                .is_valid_tile_position(held_tile, *selected_square);
        }
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
        ctx: &Context,
        canvas: &mut Canvas,
        pos: Vec2,
        color: Color,
        scale: f32,
    ) -> GameResult<()> {
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
        let scale = scale * MEEPLE_SIZE;
        let meeple_points = MEEPLE_POINTS.map(|p| (p - MEEPLE_CENTER) * scale + pos);
        let head_point = (HEAD_POINT - MEEPLE_CENTER) * scale + pos;
        Mesh::new_polygon(ctx, DrawMode::fill(), &meeple_points, color)?.draw(canvas);
        Mesh::new_circle(ctx, DrawMode::fill(), head_point, scale * 0.175, 1.0, color)?
            .draw(canvas);
        Ok(())
    }

    fn get_held_tile_mut(&mut self) -> Option<&mut Tile> {
        match &mut self.state.turn_phase {
            TurnPhase::TilePlacement { tile } => Some(tile),
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
        let player = self.state.game.players.get(player_ident).unwrap();
        let card_rect = Rect {
            x: pos.x,
            y: pos.y,
            w: 160.0,
            h: 60.0,
        };
        Mesh::new_rounded_rectangle(
            ctx,
            DrawMode::fill(),
            card_rect,
            5.0,
            Color::from_rgb(192, 192, 192),
        )?
        .draw(canvas);
        Text::new(format!("Score: {}", player.score))
            .pos(pos + vec2(10.0, 10.0))
            .color(Color::BLACK)
            .draw(canvas);
        for i in 0..player.meeples {
            GameClient::draw_meeple(
                ctx,
                canvas,
                pos + vec2(20.0, 40.0) + vec2(20.0, 0.0) * i as f32,
                player.color,
                0.1,
            )?;
        }
        if highlighted {
            Mesh::new_rounded_rectangle(ctx, DrawMode::stroke(4.0), card_rect, 5.0, player.color)?
                .draw(canvas);
        }

        Ok(())
    }

    fn end_turn(&mut self, ctx: &Context, groups_to_close: Vec<GroupIdentifier>) {
        for group_ident in groups_to_close {
            use crate::tile::SegmentType::*;
            let group = self.state.game.groups.get(group_ident).unwrap();
            match group.gtype {
                City | Road | Monastary => {
                    let scored_meeples = self.state.game.score_group(group_ident);
                    self.scoring_effects.extend(
                        scored_meeples.into_iter().map(|score_result| {
                            ScoringEffect::from_scoring_result(ctx, score_result)
                        }),
                    );
                }
                _ => {}
            }
        }

        let player_ident = self.state.turn_order.pop_front().unwrap();
        self.state.turn_order.push_back(player_ident);

        match self.state.game.draw_placeable_tile() {
            Some(tile) => self.state.turn_phase = TurnPhase::TilePlacement { tile },
            None => self.end_game(ctx),
        }
    }

    fn end_game(&mut self, ctx: &Context) {
        self.state.turn_phase = TurnPhase::EndGame {
            next_tick: Some(ctx.time.time_since_start().as_secs_f32() + END_GAME_SCORE_DELAY),
        };
    }

    fn handle_event(&mut self, ctx: &mut Context, event: GameEvent) -> Result<(), GameError> {
        match event {
            GameEvent::MainEvent(event) => self.parent_channel.send(event).unwrap(),
            GameEvent::SkipMeeples => {
                if let TurnPhase::MeeplePlacement { closed_groups, .. } = &self.state.turn_phase {
                    let groups_to_close = closed_groups.clone();
                    self.push_history();
                    self.end_turn(ctx, groups_to_close);
                }
            }
            GameEvent::ClosePauseMenu => self.pause_menu = None,
            GameEvent::EndGame => {
                self.pause_menu = None;
                self.push_history();
                self.end_game(ctx)
            }
            GameEvent::ResetCamera => {
                self.pause_menu = None;
                self.reset_camera(ctx)
            }
            GameEvent::Undo => {
                self.pause_menu = None;
                self.pop_history();
                self.reevaluate_selected_square();
            }
        }
        Ok(())
    }

    fn place_tile(&mut self, ctx: &mut Context, focused_pos: GridPos) -> Result<(), GameError> {
        self.push_history();

        let tile = self.get_held_tile_mut().unwrap().clone();
        let closed_groups = self.state.game.place_tile(tile, focused_pos)?;
        self.reevaluate_selected_square();

        let tile = self.state.game.placed_tiles.get(&focused_pos).unwrap();
        let player_ident = *self.state.turn_order.front().unwrap();
        let player = self.state.game.players.get(player_ident).unwrap();

        if player.meeples == 0
            || (0..tile.segments.len())
                .filter_map(|i| {
                    let (group, _) = self
                        .state
                        .game
                        .group_and_key_by_seg_ident((focused_pos, i))?;
                    Some(!group.meeples.is_empty())
                })
                .all(|x| x)
        {
            self.end_turn(ctx, closed_groups.clone());
        } else {
            self.state.turn_phase = TurnPhase::MeeplePlacement {
                placed_position: focused_pos,
                closed_groups,
            };
        }
        Ok(())
    }

    fn get_selected_segment(
        &self,
        focused_pos: GridPos,
        placed_position: &GridPos,
        subgrid_pos: Vec2,
    ) -> Option<(SegmentIdentifier, GroupIdentifier)> {
        if let Some(tile) = self.state.game.placed_tiles.get(&focused_pos) {
            for (i, _) in tile.segments.iter().enumerate() {
                let (group, group_ident) = self
                    .state
                    .game
                    .group_and_key_by_seg_ident((*placed_position, i))
                    .unwrap();
                if group.gtype.placeable()
                    && group.meeples.is_empty()
                    && point_in_polygon(subgrid_pos, &tile.segment_polygon(i).collect::<Vec<_>>())
                {
                    return Some(((focused_pos, i), group_ident));
                }
            }
        }
        None
    }
}

impl EventHandler<GameError> for GameClient {
    fn mouse_wheel_event(&mut self, ctx: &mut Context, _x: f32, y: f32) -> Result<(), GameError> {
        if self.pause_menu.is_some() {
            return Ok(());
        }

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
        // process events
        while let Ok(event) = self.event_receiver.try_recv() {
            self.handle_event(ctx, event)?;
        }

        // update pause menu
        if let Some(pause_menu) = &mut self.pause_menu {
            self.selected_square = None;
            pause_menu.update(ctx)?;
            return Ok(());
        }

        let mut on_clickable = false;
        let mouse: Vec2 = ctx.mouse.position().into();
        let focused_pos: GridPos = self.to_game_pos(mouse, ctx).into();
        let is_endgame = matches!(self.state.turn_phase, TurnPhase::EndGame { .. });
        let has_history = !self.history.is_empty();

        self.skip_meeples_button.borrow_mut().state = ButtonState::invisible_if(!matches!(
            self.state.turn_phase,
            TurnPhase::MeeplePlacement { .. }
        ));
        self.return_to_main_menu_button.borrow_mut().state = ButtonState::invisible_if(!matches!(
            self.state.turn_phase,
            TurnPhase::EndGame { next_tick: None }
        ));

        // update ui
        self.ui.update(ctx);

        // dragging
        if ctx.mouse.button_pressed(event::MouseButton::Right) {
            self.offset -= Vec2::from(ctx.mouse.delta());
        }

        // open pause menu
        if ctx.keyboard.is_key_just_pressed(KeyCode::Escape) {
            self.pause_menu = Some(PauseScreenSubclient::new(
                self.event_sender.clone(),
                is_endgame,
                has_history,
            ));
        }

        // update scoring effects
        self.scoring_effects.retain(|effect| {
            ctx.time.time_since_start().as_secs_f32() - effect.initialized_at < SCORE_EFFECT_LIFE
        });

        match &self.state.turn_phase {
            TurnPhase::TilePlacement { .. } => {
                self.selected_square = Some(focused_pos);

                // update selected square validity
                if self.selected_square != self.last_selected_square {
                    self.reevaluate_selected_square();
                    self.last_selected_square = self.selected_square;
                }

                if ctx.mouse.button_just_pressed(event::MouseButton::Left)
                    && self.placement_is_valid
                {
                    self.place_tile(ctx, focused_pos)?;
                }

                if ctx.keyboard.is_key_just_pressed(KeyCode::R) {
                    self.get_held_tile_mut().unwrap().rotate();
                    self.reevaluate_selected_square();
                }

                on_clickable = self.placement_is_valid;
            }
            TurnPhase::MeeplePlacement {
                placed_position,
                closed_groups,
            } => {
                self.selected_segment_and_group = None;

                if *placed_position == focused_pos {
                    let subgrid_pos = self.to_game_pos(mouse, ctx) - Vec2::from(focused_pos);
                    self.selected_segment_and_group =
                        self.get_selected_segment(focused_pos, placed_position, subgrid_pos);

                    on_clickable = self.selected_segment_and_group.is_some();

                    if ctx.mouse.button_just_pressed(event::MouseButton::Left) {
                        if let Some((seg_ident, Some(group))) =
                            self.selected_segment_and_group
                                .map(|(seg_ident, group_ident)| {
                                    (seg_ident, self.state.game.groups.get(group_ident))
                                })
                        {
                            let player_ident = *self.state.turn_order.front().unwrap();
                            let player = self.state.game.players.get(player_ident).unwrap();
                            if group.meeples.is_empty() && player.meeples > 0 {
                                let closed_groups = closed_groups.clone();
                                self.push_history();

                                // place meeple and advance turn
                                self.state.game.place_meeple(seg_ident, player_ident)?;
                                self.selected_segment_and_group = None;
                                self.end_turn(ctx, closed_groups);
                            }
                        }
                    }
                }
            }
            TurnPhase::EndGame { next_tick } => {
                if let Some(next_tick) = next_tick {
                    if ctx.time.time_since_start().as_secs_f32() > *next_tick {
                        let next_tick = 'group_score: {
                            let Some((group_ident, _)) = self
                                .state
                                .game
                                .groups
                                .iter()
                                .find(|(_, group)| !group.meeples.is_empty())
                            else {
                                break 'group_score None;
                            };
                            let scored_meeples = self.state.game.score_group(group_ident);
                            self.scoring_effects.extend(scored_meeples.into_iter().map(
                                |score_result| {
                                    ScoringEffect::from_scoring_result(ctx, score_result)
                                },
                            ));

                            Some(
                                ctx.time.time_since_start().as_secs_f32() + END_GAME_SCORE_INTERVAL,
                            )
                        };
                        self.state.turn_phase = TurnPhase::EndGame { next_tick };
                    }
                }
            }
        }

        if on_clickable {
            set_cursor_type(ctx, CursorIcon::Hand);
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let res: Vec2 = ctx.gfx.drawable_size().into();

        let mut canvas = Canvas::from_frame(ctx, Color::WHITE);

        let time = ctx.time.time_since_start().as_secs_f32();
        let sin_time = time.sin() * 0.1 + 1.0;
        let origin_rect = self.grid_pos_rect(&GridPos(0, 0), ctx);

        // draw tiles
        for (pos, tile) in &self.state.game.placed_tiles {
            tile.render(ctx, &mut canvas, self.grid_pos_rect(pos, ctx))?;
        }

        // draw meeples
        for &(seg_ident, player) in self
            .state
            .game
            .groups
            .values()
            .flat_map(|group| &group.meeples)
        {
            let color = self.state.game.players.get(player).unwrap().color;
            let (pos, seg_index) = seg_ident;
            let tile = self.state.game.placed_tiles.get(&pos).unwrap();
            let rect = self.grid_pos_rect(&pos, ctx);
            let segment_meeple_spot = refit_to_rect(tile.segments[seg_index].meeple_spot, rect);
            GameClient::draw_meeple(ctx, &mut canvas, segment_meeple_spot, color, self.scale)?;
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
            Text::new(format!(" +{} ", effect.score))
                .size(20.0)
                .centered_on(ctx, pos)?
                .color(color)
                .draw(&mut canvas);
        }

        match &self.state.turn_phase {
            TurnPhase::TilePlacement { tile } => {
                if let Some(pos) = self.selected_square {
                    let rect = self.grid_pos_rect(&pos, ctx);
                    let cursor_color = if !self.placement_is_valid {
                        Color::RED
                    } else {
                        Color::GREEN
                    };
                    if !self.state.game.placed_tiles.contains_key(&pos) {
                        tile.render(ctx, &mut canvas, rect)?;
                    }
                    Mesh::new_rectangle(ctx, DrawMode::stroke(2.0), rect, cursor_color)?
                        .draw(&mut canvas);
                }
            }
            TurnPhase::MeeplePlacement {
                placed_position, ..
            } => {
                let rect = self.grid_pos_rect(placed_position, ctx);
                Mesh::new_rectangle(ctx, DrawMode::stroke(2.0), rect, Color::CYAN)?
                    .draw(&mut canvas);

                if !self.ui.on_ui {
                    'draw_outline: {
                        if let Some((_, group_ident)) = self.selected_segment_and_group {
                            let Some(outline) = self.state.game.get_group_outline(group_ident)
                            else {
                                break 'draw_outline;
                            };
                            for polyline in outline.iter().map(|polyline| {
                                polyline
                                    .iter()
                                    .map(|vert| refit_to_rect(*vert, origin_rect))
                                    .collect::<Vec<_>>()
                            }) {
                                Mesh::new_line(
                                    ctx,
                                    &polyline,
                                    2.0,
                                    Color::from_rgb(
                                        (200.0 * sin_time) as u8,
                                        (20.0 * sin_time) as u8,
                                        (70.0 * sin_time) as u8,
                                    ),
                                )?
                                .draw(&mut canvas);
                            }
                        }
                    }
                }
            }
            TurnPhase::EndGame { .. } => {}
        }

        // draw ui
        self.ui.draw(ctx, &mut canvas)?;

        let current_player_ident = *self.state.turn_order.front().unwrap();
        let is_endgame = matches!(self.state.turn_phase, TurnPhase::EndGame { .. });
        if ctx.keyboard.is_key_pressed(KeyCode::Tab) || is_endgame {
            // draw player cards
            for (i, &player_ident) in self.state.turn_order.iter().enumerate() {
                self.render_player_card(
                    ctx,
                    &mut canvas,
                    player_ident,
                    vec2(20.0, 20.0) + vec2(0.0, 80.0) * i as f32,
                    player_ident == current_player_ident && !is_endgame,
                )?;
            }

            // draw remaining tile count
            if !is_endgame {
                let tile_count_rect = Rect::new(200.0, 20.0, 60.0, 60.0);
                Mesh::new_rounded_rectangle(
                    ctx,
                    DrawMode::fill(),
                    tile_count_rect,
                    5.0,
                    Color::from_rgb(192, 173, 138),
                )?
                .draw(&mut canvas);
                Text::new(format!("{}", self.state.game.library.len()))
                    .size(24.0)
                    .centered_on(ctx, tile_count_rect.center().into())?
                    .draw(&mut canvas);
            }
        } else {
            // draw card of current player
            self.render_player_card(
                ctx,
                &mut canvas,
                current_player_ident,
                vec2(20.0, 20.0),
                false,
            )?;
        }

        // draw player color outline
        if !is_endgame {
            Mesh::new_rectangle(
                ctx,
                DrawMode::stroke(8.0),
                Rect::new(0.0, 0.0, res.x, res.y),
                self.state
                    .game
                    .players
                    .get(current_player_ident)
                    .unwrap()
                    .color,
            )?
            .draw(&mut canvas);
        }

        // draw pause menu
        if let Some(pause_menu) = &mut self.pause_menu {
            pause_menu.draw(ctx, &mut canvas)?;
        }

        canvas.finish(ctx)
    }
}

use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;
use std::sync::mpsc::{channel, Receiver, Sender};

use crate::game::player::PlayerType;
use crate::game::ShapeDetails;
use crate::game::{
    player::Player, Game, GroupIdentifier, PlayerIdentifier, ScoringResult, SegmentIdentifier,
};
use crate::line::LineExt;
use crate::main_client::MainEvent;
use crate::multiplayer::transport::message::server::User;
use crate::multiplayer::transport::message::GameMessage;
use crate::pos::GridPos;
use crate::sub_event_handler::SubEventHandler;
use crate::tile::{tile_definitions::STARTING_TILE, Tile};
use crate::ui_manager::{Button, ButtonBounds, ButtonState, UIManager};
use crate::util::{
    point_in_polygon, refit_to_rect, AnchorPoint, ContextExt, DrawableWihParamsExt, TextExt,
};
use crate::Args;

use ggez::input::keyboard::{KeyCode, KeyMods};
use ggez::input::mouse::{set_cursor_type, CursorIcon};
use ggez::{
    event,
    glam::{vec2, Vec2, Vec2Swizzles},
    graphics::{Canvas, Color, DrawMode, DrawParam, Mesh, Rect, Text},
    Context, GameError, GameResult,
};
use pause_screen_subclient::PauseScreenSubclient;
use rand::{seq::SliceRandom, thread_rng};

mod pause_screen_subclient;

const ZOOM_SPEED: f32 = 1.1;
const MOVE_SPEED: f32 = 45.0;
const MOVE_ACCEL: f32 = 0.1;
const SPRINT_MOD: f32 = 2.0;

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
    InspectGroups,
}

#[allow(clippy::large_enum_variant)]
#[derive(Clone)]
enum TurnPhase {
    TilePlacement {
        tile: Tile,
        placeable_positions: Vec<GridPos>,
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

struct GroupInspection {
    selected_group: Option<GroupIdentifier>,
}

#[derive(Clone)]
pub struct GameState {
    game: Game,
    turn_phase: TurnPhase,
    turn_order: VecDeque<PlayerIdentifier>,
}

pub struct GameClient {
    parent_channel: Sender<MainEvent>,
    action_channel: Option<Sender<GameMessage>>,
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
    inspecting_groups: Option<GroupInspection>,
    ui: UIManager<GameEvent, GameEvent>,
    camera_movement: Vec2,
    args: Args,
}

impl GameClient {
    pub fn new(
        ctx: &Context,
        args: Args,
        players: Vec<Color>,
        parent_channel: Sender<MainEvent>,
    ) -> Self {
        let mut game = Game::new();
        game.library.shuffle(&mut thread_rng());
        for color in players {
            game.players.insert(Player::new(color));
        }
        game.place_tile(STARTING_TILE.clone(), GridPos(0, 0))
            .unwrap();
        Self::new_with_game(ctx, args, game, parent_channel)
    }

    pub fn new_with_game(
        ctx: &Context,
        args: Args,
        game: Game,
        parent_channel: Sender<MainEvent>,
    ) -> Self {
        GameClient::new_inner(ctx, args, game, parent_channel, None)
    }

    pub fn new_inner(
        ctx: &Context,
        args: Args,
        mut game: Game,
        parent_channel: Sender<MainEvent>,
        action_channel: Option<Sender<GameMessage>>,
    ) -> Self {
        let (first_tile, placeable_positions) = game.draw_placeable_tile().unwrap();
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
            action_channel,
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
                turn_phase: TurnPhase::TilePlacement {
                    tile: first_tile,
                    placeable_positions,
                },
                turn_order: game.players.keys().collect(),
                game,
            },
            inspecting_groups: None,
            skip_meeples_button,
            return_to_main_menu_button,
            ui,
            camera_movement: Vec2::ZERO,
            args,
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
        self.camera_movement = Vec2::ZERO;
    }

    fn reevaluate_selected_square(&mut self) {
        self.reevaluate_selected_square_inner(true);
    }

    fn reevaluate_selected_square_counterclockwise(&mut self) {
        self.reevaluate_selected_square_inner(false);
    }

    fn reevaluate_selected_square_inner(&mut self, clockwise: bool) {
        self.placement_is_valid = false;

        let Some(selected_square) = &self.selected_square else {
            return;
        };

        if self.state.game.placed_tiles.contains_key(selected_square) {
            return;
        }

        if let TurnPhase::TilePlacement {
            tile: held_tile, ..
        } = &mut self.state.turn_phase
        {
            if self.args.snap_placement {
                while !self
                    .state
                    .game
                    .is_valid_tile_position(held_tile, *selected_square)
                {
                    if clockwise {
                        held_tile.rotate_clockwise();
                    } else {
                        held_tile.rotate_counterclockwise();
                    }
                }
                self.placement_is_valid = true;
            } else {
                self.placement_is_valid = self
                    .state
                    .game
                    .is_valid_tile_position(held_tile, *selected_square);
            }
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
            TurnPhase::TilePlacement { tile, .. } => Some(tile),
            _ => None,
        }
    }

    pub fn can_play(&self) -> bool {
        self.get_current_player() == self.state.game.local_player
    }

    fn draw_player_card(
        &self,
        ctx: &Context,
        canvas: &mut Canvas,
        player_ident: PlayerIdentifier,
        pos: Vec2,
        highlighted: bool,
    ) -> Result<f32, GameError> {
        let player = self.state.game.players.get(player_ident).unwrap();
        let mut card_rect = Rect {
            x: pos.x,
            y: pos.y,
            w: 160.0,
            h: 60.0,
        };
        let mut content_origin = vec2(10.0, 10.0);
        let display_name = match player.ptype {
            PlayerType::Local => None,
            PlayerType::MultiplayerHost => Some("Host".to_string()),
            PlayerType::MultiplayerClient { address, .. } => Some(address.to_string()),
        };
        if display_name.is_some() {
            card_rect.h += 20.0;
            card_rect.w += 60.0;
            content_origin.y += 20.0;
        }
        Mesh::new_rounded_rectangle(
            ctx,
            DrawMode::fill(),
            card_rect,
            5.0,
            Color::from_rgb(192, 192, 192),
        )?
        .draw(canvas);
        if let Some(display_name) = display_name {
            Text::new(display_name)
                .pos(pos + content_origin + vec2(0.0, -20.0))
                .color(Color::BLACK)
                .draw(canvas);
            if let PlayerType::MultiplayerClient {
                latency: Some(latency),
                ..
            } = player.ptype
            {
                Text::new(format!("{}ms", latency.as_millis()))
                    .anchored_by(
                        ctx,
                        pos + vec2(card_rect.w, 0.0) + vec2(-10.0, 10.0),
                        AnchorPoint::NorthEast,
                    )?
                    .color(Color::BLACK)
                    .draw(canvas);
            }
        }
        Text::new(format!("Score: {}", player.score))
            .pos(pos + content_origin)
            .color(Color::BLACK)
            .draw(canvas);
        for i in 0..player.meeples {
            GameClient::draw_meeple(
                ctx,
                canvas,
                pos + content_origin + vec2(10.0, 30.0) + vec2(20.0, 0.0) * i as f32,
                player.color,
                0.1,
            )?;
        }
        if highlighted {
            Mesh::new_rounded_rectangle(ctx, DrawMode::stroke(4.0), card_rect, 5.0, player.color)?
                .draw(canvas);
        }

        Ok(card_rect.h)
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
            Some((tile, placeable_positions)) => {
                self.state.turn_phase = TurnPhase::TilePlacement {
                    tile,
                    placeable_positions,
                }
            }
            None => self.end_game(ctx),
        }
    }

    fn end_game(&mut self, ctx: &Context) {
        self.state.turn_phase = TurnPhase::EndGame {
            next_tick: Some(ctx.time.time_since_start().as_secs_f32() + END_GAME_SCORE_DELAY),
        };
    }

    fn skip_meeples(&mut self, ctx: &Context) {
        if let TurnPhase::MeeplePlacement { closed_groups, .. } = &self.state.turn_phase {
            let groups_to_close = closed_groups.clone();
            self.push_history();
            self.end_turn(ctx, groups_to_close);
        }
    }

    fn end_game_immediately(&mut self, ctx: &Context) {
        self.push_history();
        self.end_game(ctx);
    }

    fn handle_event(&mut self, ctx: &mut Context, event: GameEvent) -> Result<(), GameError> {
        match event {
            GameEvent::MainEvent(event) => self.parent_channel.send(event).unwrap(),
            GameEvent::SkipMeeples => {
                self.skip_meeples(ctx);
                self.broadcast_action(GameMessage::SkipMeeples);
            }
            GameEvent::ClosePauseMenu => self.pause_menu = None,
            GameEvent::EndGame => {
                self.pause_menu = None;
                self.end_game_immediately(ctx);
                self.broadcast_action(GameMessage::EndGame);
            }
            GameEvent::ResetCamera => {
                self.pause_menu = None;
                self.reset_camera(ctx)
            }
            GameEvent::Undo => {
                if self.can_play() {
                    self.pause_menu = None;
                    self.pop_history();
                    self.reevaluate_selected_square();
                    self.broadcast_action(GameMessage::Undo);
                }
            }
            GameEvent::InspectGroups => {
                self.pause_menu = None;
                self.inspecting_groups = Some(GroupInspection {
                    selected_group: None,
                });
            }
        }
        Ok(())
    }

    pub fn handle_message(&mut self, ctx: &mut Context, message: GameMessage) -> GameResult<()> {
        match message {
            GameMessage::PlaceTile {
                selected_square,
                rotation,
            } => {
                if let TurnPhase::TilePlacement { tile, .. } = &mut self.state.turn_phase {
                    while tile.rotation != rotation {
                        tile.rotate_clockwise();
                    }
                    self.place_tile(ctx, selected_square)?;
                }
            }
            GameMessage::PlaceMeeple { seg_ident } => {
                if let TurnPhase::MeeplePlacement { closed_groups, .. } = &mut self.state.turn_phase
                {
                    let closed_groups = closed_groups.clone();
                    let player_ident = self.state.turn_order.front().unwrap();
                    self.place_meeple(ctx, closed_groups, seg_ident, *player_ident)?;
                }
            }
            GameMessage::SkipMeeples => self.skip_meeples(ctx),
            GameMessage::EndGame => self.end_game_immediately(ctx),
            GameMessage::Undo => {
                self.pop_history();
                self.reevaluate_selected_square();
            }
        }
        Ok(())
    }

    pub fn update_pings(&mut self, users: Vec<User>) -> GameResult<()> {
        for player in self.state.game.players.values_mut() {
            if let Some(new_latency) = users.iter().find_map(|user| {
                user.client_info.as_ref().and_then(|client_info| {
                    (PlayerType::from(Some(client_info.ip)) == player.ptype)
                        .then_some(client_info.latency)
                })
            }) {
                if let PlayerType::MultiplayerClient { latency, .. } = &mut player.ptype {
                    *latency = new_latency;
                }
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

    fn place_meeple(
        &mut self,
        ctx: &Context,
        closed_groups: Vec<GroupIdentifier>,
        seg_ident: SegmentIdentifier,
        player_ident: PlayerIdentifier,
    ) -> Result<(), GameError> {
        self.push_history();

        self.state.game.place_meeple(seg_ident, player_ident)?;
        self.selected_segment_and_group = None;
        self.end_turn(ctx, closed_groups);
        Ok(())
    }

    fn broadcast_action(&mut self, message: GameMessage) {
        if let Some(action_channel) = &mut self.action_channel {
            let _ = action_channel.send(message);
        }
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

    pub fn get_current_player(&self) -> PlayerType {
        self.state
            .game
            .players
            .get(*self.state.turn_order.front().unwrap())
            .unwrap()
            .ptype
    }
}

impl SubEventHandler<GameError> for GameClient {
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
        self.ui.update(ctx)?;
        while let Ok(event) = self.event_receiver.try_recv() {
            self.handle_event(ctx, event)?;
        }

        let has_history = !self.history.is_empty();
        let can_play = self.can_play();
        let is_endgame = matches!(self.state.turn_phase, TurnPhase::EndGame { .. });

        // update pause menu
        if let Some(pause_menu) = &mut self.pause_menu {
            self.selected_square = None;
            pause_menu.can_end_game.set(is_endgame);
            pause_menu.can_undo.set(can_play && has_history);
            pause_menu.update(ctx)?;
            return Ok(());
        }

        let mut on_clickable = false;
        let mouse: Vec2 = ctx.mouse.position().into();
        let focused_pos: GridPos = self.to_game_pos(mouse, ctx).into();

        self.skip_meeples_button.borrow_mut().state = ButtonState::invisible_if(
            !matches!(self.state.turn_phase, TurnPhase::MeeplePlacement { .. }) || !can_play,
        );
        self.return_to_main_menu_button.borrow_mut().state = ButtonState::invisible_if(!matches!(
            self.state.turn_phase,
            TurnPhase::EndGame { next_tick: None }
        ));

        // dragging
        if ctx.mouse.button_pressed(event::MouseButton::Right) {
            self.offset -= Vec2::from(ctx.mouse.delta());
        }

        // keyboard based movement
        {
            let mut movement_vector: Vec2 = [
                (KeyCode::W, vec2(0.0, -1.0)),
                (KeyCode::Up, vec2(0.0, -1.0)),
                (KeyCode::D, vec2(1.0, 0.0)),
                (KeyCode::Right, vec2(1.0, 0.0)),
                (KeyCode::S, vec2(0.0, 1.0)),
                (KeyCode::Down, vec2(0.0, 1.0)),
                (KeyCode::A, vec2(-1.0, 0.0)),
                (KeyCode::Left, vec2(-1.0, 0.0)),
            ]
            .into_iter()
            .map(|(code, dir)| {
                if ctx.keyboard.is_key_pressed(code) {
                    dir
                } else {
                    Vec2::ZERO
                }
            })
            .sum();
            if movement_vector != Vec2::ZERO {
                movement_vector = movement_vector.normalize();
            }
            movement_vector = movement_vector * self.scale * MOVE_SPEED;
            if ctx.keyboard.is_mod_active(KeyMods::SHIFT) {
                movement_vector *= SPRINT_MOD;
            }
            self.camera_movement =
                self.camera_movement * (1.0 - MOVE_ACCEL) + movement_vector * MOVE_ACCEL;
            self.offset += self.camera_movement;
        }

        // open pause menu / close group inspection ui
        if ctx.keyboard.is_key_just_pressed(KeyCode::Escape) {
            if self.inspecting_groups.is_some() {
                self.inspecting_groups = None;
            } else {
                self.pause_menu = Some(PauseScreenSubclient::new(
                    self.event_sender.clone(),
                    is_endgame,
                    has_history && can_play,
                ));
            }
        }

        // update scoring effects
        self.scoring_effects.retain(|effect| {
            ctx.time.time_since_start().as_secs_f32() - effect.initialized_at < SCORE_EFFECT_LIFE
        });

        let gameboard_pos = self.to_game_pos(mouse, ctx);
        let subgrid_pos = gameboard_pos - Vec2::from(focused_pos);
        if let Some(group_inspection) = &mut self.inspecting_groups {
            group_inspection.selected_group = self
                .state
                .game
                .placed_tiles
                .get(&focused_pos)
                .and_then(|tile| {
                    (0..tile.segments.len()).find(|seg_index| {
                        tile.segments[*seg_index].stype.placeable() && {
                            let segment_poly: Vec<_> = tile.segment_polygon(*seg_index).collect();
                            point_in_polygon(subgrid_pos, &segment_poly)
                        }
                    })
                })
                .and_then(|i| self.state.game.group_associations.get(&(focused_pos, i)))
                .cloned();
        } else {
            'game_play: {
                match &self.state.turn_phase {
                    TurnPhase::TilePlacement {
                        placeable_positions,
                        ..
                    } => {
                        // determine tile placement location
                        if !can_play {
                            self.selected_square = None;
                            break 'game_play;
                        }

                        if self.args.snap_placement {
                            self.selected_square = placeable_positions
                                .iter()
                                .cloned()
                                .map(|pos| {
                                    (
                                        pos,
                                        Vec2::from(pos)
                                            .distance_squared(gameboard_pos - vec2(0.5, 0.5)),
                                    )
                                })
                                .min_by(|(_, a), (_, b)| a.total_cmp(b))
                                .map(|(pos, _)| pos);
                        } else {
                            self.selected_square = Some(focused_pos);
                        }

                        // update selected square validity
                        if self.selected_square != self.last_selected_square {
                            self.reevaluate_selected_square();
                            self.last_selected_square = self.selected_square;
                        }

                        // place tile
                        if ctx.mouse.button_just_pressed(event::MouseButton::Left)
                            && self.placement_is_valid
                        {
                            if let Some(selected_square) = self.selected_square {
                                let rotation = self.get_held_tile_mut().unwrap().rotation;
                                self.place_tile(ctx, selected_square)?;
                                self.broadcast_action(GameMessage::PlaceTile {
                                    selected_square,
                                    rotation,
                                });
                            }
                        }

                        // rotate tile
                        if ctx.keyboard.is_key_just_pressed(KeyCode::R) {
                            self.get_held_tile_mut().unwrap().rotate_clockwise();
                            self.reevaluate_selected_square();
                        }

                        // rotate tile counterclockwise (dont tell anyone it's actually just three clockwise rotations)
                        if ctx.keyboard.is_key_just_pressed(KeyCode::E) {
                            let tile = self.get_held_tile_mut().unwrap();
                            tile.rotate_counterclockwise();
                            self.reevaluate_selected_square_counterclockwise();
                        }

                        on_clickable = self.placement_is_valid;
                    }
                    TurnPhase::MeeplePlacement {
                        placed_position,
                        closed_groups,
                    } => {
                        self.selected_segment_and_group = None;

                        if !can_play {
                            break 'game_play;
                        }

                        if *placed_position == focused_pos {
                            let subgrid_pos =
                                self.to_game_pos(mouse, ctx) - Vec2::from(focused_pos);
                            self.selected_segment_and_group = self.get_selected_segment(
                                focused_pos,
                                placed_position,
                                subgrid_pos,
                            );

                            on_clickable = self.selected_segment_and_group.is_some();

                            if ctx.mouse.button_just_pressed(event::MouseButton::Left) {
                                if let Some((seg_ident, Some(group))) = self
                                    .selected_segment_and_group
                                    .map(|(seg_ident, group_ident)| {
                                        (seg_ident, self.state.game.groups.get(group_ident))
                                    })
                                {
                                    let player_ident = *self.state.turn_order.front().unwrap();
                                    let player = self.state.game.players.get(player_ident).unwrap();
                                    if group.meeples.is_empty() && player.meeples > 0 {
                                        self.place_meeple(
                                            ctx,
                                            closed_groups.clone(),
                                            seg_ident,
                                            player_ident,
                                        )?;
                                        self.broadcast_action(GameMessage::PlaceMeeple {
                                            seg_ident,
                                        });
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
                                        ctx.time.time_since_start().as_secs_f32()
                                            + END_GAME_SCORE_INTERVAL,
                                    )
                                };
                                self.state.turn_phase = TurnPhase::EndGame { next_tick };
                            }
                        }
                    }
                }
            }
        }

        if on_clickable {
            set_cursor_type(ctx, CursorIcon::Hand);
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context, canvas: &mut Canvas) -> GameResult {
        let res = ctx.res();

        let time = ctx.time.time_since_start().as_secs_f32();
        let sin_time = time.sin() * 0.1 + 1.0;
        let origin_rect = self.grid_pos_rect(&GridPos(0, 0), ctx);

        // draw tiles
        for (pos, tile) in &self.state.game.placed_tiles {
            tile.render(ctx, canvas, self.grid_pos_rect(pos, ctx))?;
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
            let norm = self.norm(ctx);
            let meeple_scale = 0.001 / norm.x.max(norm.y);
            GameClient::draw_meeple(
                ctx,
                canvas,
                segment_meeple_spot,
                color,
                meeple_scale,
            )?;
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
                .draw(canvas);
        }

        if let Some(group_inspection) = &self.inspecting_groups {
            // draw group inspection ui
            Text::new("Press Esc to return")
                .size(32.0)
                .anchored_by(
                    ctx,
                    res * vec2(0.5, 0.0) + vec2(0.0, 10.0),
                    AnchorPoint::NorthCenter,
                )?
                .color(Color::BLACK)
                .draw(canvas);

            let _: Option<()> =
                try {
                    let group_ident = group_inspection.selected_group?;
                    let score_details = self
                        .state
                        .game
                        .get_group_scoring_details(group_ident)?
                        .clone();
                    let shape_details = self.state.game.get_group_shape_details(group_ident)?;

                    // draw colored outline
                    let refit_line = |line: &Vec<Vec2>| -> Vec<Vec2> {
                        line.iter()
                            .map(|vert| refit_to_rect(*vert, origin_rect))
                            .collect()
                    };
                    for line in shape_details.outline.iter().map(refit_line) {
                        Mesh::new_line(ctx, &line, 4.0, Color::BLACK)
                            .ok()?
                            .draw(canvas);
                    }
                    if score_details.owners.len() < 2 {
                        let color = score_details
                            .owners
                            .first()
                            .map(|(_, c)| c)
                            .cloned()
                            .unwrap_or(Color::from_rgb(100, 100, 100));
                        for line in shape_details.outline.iter().map(refit_line) {
                            Mesh::new_line(ctx, &line, 3.0, color).ok()?.draw(canvas);
                        }
                    } else {
                        const DOTTED_LINE_LENGTH: f32 = 25.0;
                        for line in shape_details.outline.iter().map(refit_line) {
                            for (line_seg, color) in
                                score_details.owners.iter().enumerate().flat_map(
                                    |(i, (_, color))| {
                                        line.offset_subsections(
                                            DOTTED_LINE_LENGTH
                                                * (score_details.owners.len() - 1) as f32,
                                            DOTTED_LINE_LENGTH,
                                            DOTTED_LINE_LENGTH * i as f32,
                                        )
                                        .map(move |line_seg| (line_seg, color))
                                    },
                                )
                            {
                                Mesh::new_line(ctx, &line_seg, 3.0, *color)
                                    .ok()?
                                    .draw(canvas);
                            }
                        }
                    }

                    // draw info box
                    let anchor_point = refit_to_rect(shape_details.popup_location, origin_rect)
                        - vec2(90.0, 120.0);
                    let group = self.state.game.groups.get(group_ident)?;
                    let infobox_rect = Rect::new(anchor_point.x, anchor_point.y, 180.0, 62.0);
                    Mesh::new_rounded_rectangle(
                        ctx,
                        DrawMode::fill(),
                        infobox_rect,
                        5.0,
                        group.gtype.color(),
                    )
                    .ok()?
                    .draw(canvas);
                    Mesh::new_rounded_rectangle(
                        ctx,
                        DrawMode::stroke(5.0),
                        infobox_rect,
                        5.0,
                        Color::from_rgb(100, 100, 100),
                    )
                    .ok()?
                    .draw(canvas);
                    Text::new(format!(
                        "\
{}
{} Point{}
{}",
                        group.gtype.name(),
                        score_details.score,
                        if score_details.score == 1 { "" } else { "s" },
                        if score_details.owners.is_empty() {
                            "Unclaimed"
                        } else {
                            "Owners:"
                        }
                    ))
                    .anchored_by(ctx, anchor_point + vec2(8.0, 8.0), AnchorPoint::NorthWest)
                    .ok()?
                    .color(Color::BLACK)
                    .draw(canvas);

                    let meeple_point = anchor_point + vec2(77.0, 47.0);
                    for (i, (_, color)) in score_details.owners.iter().enumerate() {
                        GameClient::draw_meeple(
                            ctx,
                            canvas,
                            meeple_point + vec2(20.0, 0.0) * i as f32,
                            *color,
                            0.075,
                        )
                        .ok()?;
                    }
                };

            // finish ui
            Mesh::new_rectangle(
                ctx,
                DrawMode::stroke(6.0),
                Rect::new(0.0, 0.0, res.x, res.y),
                Color::CYAN,
            )?
            .draw(canvas);
        } else {
            match &self.state.turn_phase {
                TurnPhase::TilePlacement { tile, .. } => {
                    // draw tile placement ui
                    if let Some(pos) = self.selected_square {
                        let rect = self.grid_pos_rect(&pos, ctx);
                        let cursor_color = if !self.placement_is_valid {
                            Color::RED
                        } else {
                            Color::GREEN
                        };
                        if !self.state.game.placed_tiles.contains_key(&pos) {
                            tile.render(ctx, canvas, rect)?;
                        }
                        Mesh::new_rectangle(ctx, DrawMode::stroke(2.0), rect, cursor_color)?
                            .draw(canvas);
                    }
                }
                TurnPhase::MeeplePlacement {
                    placed_position, ..
                } => {
                    // draw meeple placement ui
                    let rect = self.grid_pos_rect(placed_position, ctx);
                    Mesh::new_rectangle(ctx, DrawMode::stroke(2.0), rect, Color::CYAN)?
                        .draw(canvas);

                    if !self.ui.on_ui {
                        'draw_outline: {
                            if let Some((_, group_ident)) = self.selected_segment_and_group {
                                let Some(ShapeDetails { outline, .. }) =
                                    self.state.game.get_group_shape_details(group_ident)
                                else {
                                    break 'draw_outline;
                                };
                                for line in outline.iter().map(|line| {
                                    line.iter()
                                        .map(|vert| refit_to_rect(*vert, origin_rect))
                                        .collect::<Vec<_>>()
                                }) {
                                    Mesh::new_line(
                                        ctx,
                                        &line,
                                        2.0,
                                        Color::from_rgb(
                                            (200.0 * sin_time) as u8,
                                            (20.0 * sin_time) as u8,
                                            (70.0 * sin_time) as u8,
                                        ),
                                    )?
                                    .draw(canvas);
                                }
                            }
                        }
                    }
                }
                TurnPhase::EndGame { .. } => {}
            }
        }

        // draw ui
        self.ui.draw(ctx, canvas)?;

        let current_player_ident = *self.state.turn_order.front().unwrap();
        let is_endgame = matches!(self.state.turn_phase, TurnPhase::EndGame { .. });
        if ctx.keyboard.is_key_pressed(KeyCode::Tab) || is_endgame {
            // draw player cards
            let mut card_location = vec2(20.0, 20.0);
            for &player_ident in &self.state.turn_order {
                let height = self.draw_player_card(
                    ctx,
                    canvas,
                    player_ident,
                    card_location,
                    player_ident == current_player_ident && !is_endgame,
                )?;
                card_location.y += height + 20.0;
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
                .draw(canvas);
                Text::new(format!("{}", self.state.game.library.len()))
                    .size(24.0)
                    .centered_on(ctx, tile_count_rect.center().into())?
                    .draw(canvas);
            }
        } else {
            // draw card of current player
            self.draw_player_card(ctx, canvas, current_player_ident, vec2(20.0, 20.0), false)?;
        }

        // draw player color outline if it is your turn
        if !is_endgame && self.can_play() && self.inspecting_groups.is_none() {
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
            .draw(canvas);
        }

        // draw pause menu
        if let Some(pause_menu) = &mut self.pause_menu {
            pause_menu.draw(ctx, canvas)?;
        }

        Ok(())
    }
}

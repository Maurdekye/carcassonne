use std::cell::RefCell;
use std::collections::VecDeque;
use std::fs::{create_dir_all, File};
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::time::SystemTime;

use crate::colors::PANEL_COLOR;
use crate::game::player::{ConnectionState, PlayerType};
use crate::game::ShapeDetails;
use crate::game::{
    player::Player, Game, GroupIdentifier, PlayerIdentifier, ScoringResult, SegmentIdentifier,
};
use crate::keybinds::Keybinds;
use crate::line::LineExt;
use crate::main_client::MainEvent;
use crate::multiplayer::transport::message::server::User;
use crate::multiplayer::transport::message::{GameMessage, TilePreview};
use crate::pos::GridPos;
use crate::sub_event_handler::SubEventHandler;
use crate::tile::{tile_definitions::STARTING_TILE, Tile};
use crate::ui_manager::{Bounds, Button, UIElement, UIElementState, UIManager};
use crate::util::{
    point_in_polygon, refit_to_rect, AnchorPoint, ContextExt, DrawableWihParamsExt, MinByF32Key,
    ResultExt, SystemTimeExt, TextExt,
};
use crate::{game_client, SharedResources};

use ggez::input::mouse::{set_cursor_type, CursorIcon};
use ggez::{
    glam::{vec2, Vec2, Vec2Swizzles},
    graphics::{Canvas, Color, DrawMode, DrawParam, Mesh, Rect, Text},
    Context, GameError, GameResult,
};
use log::{debug, trace};
use pause_screen_subclient::PauseScreenSubclient;
use rand::{seq::SliceRandom, thread_rng};
use serde::{Deserialize, Serialize};

mod pause_screen_subclient;

const ZOOM_SPEED: f32 = 1.1;
const MOVE_SPEED: f32 = 45.0;
const MOVE_ACCEL: f32 = 0.1;
const SPRINT_MOD: f32 = 2.0;

const KEYBOARD_ZOOM_SPEED: f32 = 0.65;
const KEYBOARD_ZOOM_ACEL: f32 = 0.1;

const SCORE_EFFECT_LIFE: f32 = 2.5;
const SCORE_EFFECT_DISTANCE: f32 = 0.4;
const SCORE_EFFECT_DECCEL: f32 = 15.0;

const END_GAME_SCORE_DELAY: f32 = 3.0;
const END_GAME_SCORE_INTERVAL: f32 = 1.75;

const MEEPLE_SIZE: f32 = 0.001;

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
    ReturnToLobby,
}

#[derive(Clone, Debug)]
pub enum GameAction {
    Message(GameMessage),
    ReturnToLobby,
}

#[allow(clippy::large_enum_variant)]
#[derive(Clone, Serialize, Deserialize)]
enum TurnPhase {
    TilePlacement {
        tile: Tile,
        placeable_positions: Vec<GridPos>,
        preview_location: Option<GridPos>,
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

#[derive(Clone, Serialize, Deserialize)]
pub struct GameState {
    pub game: Game,
    turn_phase: TurnPhase,
    turn_order: VecDeque<PlayerIdentifier>,
}

impl std::fmt::Debug for GameState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        #[allow(unused)]
        #[derive(Debug)]
        struct Game<'a> {
            tiles: usize,
            players: Vec<&'a Player>,
        }
        #[derive(Debug)]
        enum TurnPhase {
            MeeplePlacement,
            TilePlacement,
            EndGame,
        }
        f.debug_struct("GameState")
            .field(
                "game",
                &Game {
                    tiles: self.game.placed_tiles.len(),
                    players: self
                        .turn_order
                        .iter()
                        .map(|p| self.game.players.get(*p).unwrap())
                        .collect(),
                },
            )
            .field(
                "turn_phase",
                &match &self.turn_phase {
                    game_client::TurnPhase::TilePlacement { .. } => TurnPhase::TilePlacement,
                    game_client::TurnPhase::MeeplePlacement { .. } => TurnPhase::MeeplePlacement,
                    game_client::TurnPhase::EndGame { .. } => TurnPhase::EndGame,
                },
            )
            .finish()
    }
}

#[derive(PartialEq, Eq, Debug)]
enum PlacementValidity {
    Invalid,
    ValidWithDifferentRotation,
    Valid,
}

struct GridSelectionInfo {
    gameboard_pos: Vec2,
    focused_pos: GridPos,
    subgrid_pos: Vec2,
}

pub struct GameClient {
    parent_channel: Sender<MainEvent>,
    action_channel: Option<Sender<GameAction>>,
    event_sender: Sender<GameEvent>,
    event_receiver: Receiver<GameEvent>,
    pause_menu: Option<PauseScreenSubclient>,
    selected_square: Option<GridPos>,
    selected_segment_and_group: Option<(SegmentIdentifier, GroupIdentifier)>,
    placement_is_valid: bool,
    offset: Vec2,
    scale: f32,
    scoring_effects: Vec<ScoringEffect>,
    history: Vec<GameState>,
    skip_meeples_button: Rc<RefCell<Button<GameEvent>>>,
    return_to_main_menu_button: Rc<RefCell<Button<GameEvent>>>,
    pub state: GameState,
    inspecting_groups: Option<GroupInspection>,
    ui: UIManager<GameEvent, GameEvent>,
    camera_movement: Vec2,
    camera_zoom: f32,
    creation_time: SystemTime,
    shared: SharedResources,
    keybinds: Keybinds,
}

impl GameClient {
    pub fn new(
        ctx: &Context,
        shared: SharedResources,
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
        Self::new_with_game(ctx, shared, game, parent_channel)
    }

    pub fn new_with_game(
        ctx: &Context,
        args: SharedResources,
        game: Game,
        parent_channel: Sender<MainEvent>,
    ) -> Self {
        GameClient::new_with_game_and_action_channel(ctx, args, game, parent_channel, None)
    }

    pub fn new_with_game_and_action_channel(
        ctx: &Context,
        shared: SharedResources,
        mut game: Game,
        parent_channel: Sender<MainEvent>,
        action_channel: Option<Sender<GameAction>>,
    ) -> Self {
        let (first_tile, placeable_positions) = game.draw_placeable_tile().unwrap();
        GameClient::new_from_state(
            ctx,
            shared,
            GameState {
                turn_phase: TurnPhase::TilePlacement {
                    tile: first_tile,
                    placeable_positions,
                    preview_location: None,
                },
                turn_order: game.players.keys().collect(),
                game,
            },
            parent_channel,
            action_channel,
        )
    }

    pub fn new_from_state(
        ctx: &Context,
        shared: SharedResources,
        state: GameState,
        parent_channel: Sender<MainEvent>,
        action_channel: Option<Sender<GameAction>>,
    ) -> Self {
        let (event_sender, event_receiver) = channel();
        let is_local = matches!(state.game.local_player, PlayerType::Local);
        let keybinds = shared.persistent.borrow().keybinds.clone();
        let ui_sender = event_sender.clone();
        let (
            ui,
            [UIElement::Button(skip_meeples_button), UIElement::Button(return_to_main_menu_button)],
        ) = UIManager::new_and_rc_elements(
            ui_sender,
            [
                UIElement::Button(Button::new_with_styling(
                    Bounds {
                        relative: Rect::new(1.0, 0.0, 0.0, 0.0),
                        absolute: Rect::new(-220.0, 20.0, 200.0, 40.0),
                    },
                    Text::new(format!("Skip meeples ({})", keybinds.skip_meeples)),
                    DrawParam::default(),
                    Color::from_rgb(0, 128, 192),
                    GameEvent::SkipMeeples,
                )),
                UIElement::Button(Button::new(
                    Bounds {
                        relative: Rect::new(1.0, 0.0, 0.0, 0.0),
                        absolute: Rect::new(-260.0, 20.0, 240.0, 40.0),
                    },
                    Text::new(if is_local {
                        "Return to Main Menu"
                    } else {
                        "Return to Lobby"
                    }),
                    if is_local {
                        GameEvent::MainEvent(MainEvent::MainMenu)
                    } else {
                        GameEvent::ReturnToLobby
                    },
                )),
            ],
        )
        else {
            panic!()
        };
        let mut this = Self {
            parent_channel,
            action_channel,
            event_sender,
            event_receiver,
            pause_menu: None,
            selected_square: None,
            selected_segment_and_group: None,
            placement_is_valid: false,
            offset: Vec2::ZERO,
            scale: 1.0,
            scoring_effects: Vec::new(),
            history: Vec::new(),
            state,
            inspecting_groups: None,
            skip_meeples_button,
            return_to_main_menu_button,
            ui,
            camera_movement: Vec2::ZERO,
            camera_zoom: 0.0,
            creation_time: SystemTime::now(),
            shared,
            keybinds,
        };
        this.reset_camera(ctx);
        this
    }

    pub fn load(
        ctx: &Context,
        args: SharedResources,
        parent_channel: Sender<MainEvent>,
        action_channel: Option<Sender<GameAction>>,
        path: PathBuf,
    ) -> GameResult<Self> {
        let file = File::open(path)?;
        let mut state: GameState = bincode::deserialize_from(file).to_gameerror()?;
        for (_, player) in &mut state.game.players {
            player.ptype = PlayerType::Local;
        }
        state.game.local_player = PlayerType::Local;
        Ok(Self::new_from_state(
            ctx,
            args,
            state,
            parent_channel,
            action_channel,
        ))
    }

    fn save(&self, mut path: PathBuf) -> GameResult<()> {
        path.push(self.creation_time.strftime("%Y-%m-%d_%H-%M-%S"));
        let _ = create_dir_all(&path);
        path.push(SystemTime::now().strftime("%Y-%m-%d_%H-%M-%S%.3f.save"));
        debug!("saving game state to {}", path.display());
        let mut file = File::create(path)?;
        bincode::serialize_into(&mut file, &self.state).to_gameerror()
    }

    fn push_history(&mut self) -> GameResult<()> {
        if let Some(Some(base_path)) = &self.shared.args.save_games {
            self.save(base_path.clone())?;
        }
        self.history.push(self.state.clone());
        Ok(())
    }

    fn pop_history(&mut self) {
        if let Some(state) = self.history.pop() {
            debug!("pop history");
            self.state = state;
        }
    }

    fn reset_camera(&mut self, ctx: &Context) {
        debug!("camera reset");
        self.scale = 0.1;
        self.offset = -Vec2::from(ctx.gfx.drawable_size()) * Vec2::splat(0.5 - self.scale / 2.0);
        self.camera_movement = Vec2::ZERO;
        self.camera_zoom = 0.0;
    }

    fn reevaluate_selected_square(&mut self) {
        self.reevaluate_selected_square_inner(true);
    }

    fn reevaluate_selected_square_counterclockwise(&mut self) {
        self.reevaluate_selected_square_inner(false);
    }

    fn is_placement_valid(&self, pos: GridPos) -> PlacementValidity {
        use PlacementValidity::*;
        if self.state.game.placed_tiles.contains_key(&pos) {
            return Invalid;
        }

        let TurnPhase::TilePlacement {
            tile: held_tile, ..
        } = &self.state.turn_phase
        else {
            return Invalid;
        };

        if self.state.game.is_valid_tile_position(held_tile, pos) {
            Valid
        } else {
            ValidWithDifferentRotation
        }
    }

    fn reevaluate_selected_square_inner(&mut self, clockwise: bool) {
        use PlacementValidity::*;
        self.placement_is_valid = false;

        let Some(selected_square) = self.selected_square else {
            return;
        };

        let mut placement_validity = self.is_placement_valid(selected_square);
        if self.shared.args.snap_placement {
            for _ in 0..4 {
                if placement_validity != ValidWithDifferentRotation {
                    break;
                } else if clockwise {
                    self.get_held_tile_mut().unwrap().rotate_clockwise();
                } else {
                    self.get_held_tile_mut().unwrap().rotate_counterclockwise();
                }
                placement_validity = self.is_placement_valid(selected_square);
            }
        }

        self.placement_is_valid = placement_validity == Valid
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

    fn is_endgame(&self) -> bool {
        matches!(self.state.turn_phase, TurnPhase::EndGame { .. })
    }

    pub fn can_play(&self) -> bool {
        self.get_current_player_type() == &self.state.game.local_player && !self.is_endgame()
    }

    fn draw_player_card(
        &self,
        ctx: &Context,
        canvas: &mut Canvas,
        player_ident: PlayerIdentifier,
        pos: Vec2,
        highlighted: bool,
    ) -> Result<Rect, GameError> {
        let player = self.state.game.players.get(player_ident).unwrap();
        let mut card_rect = Rect {
            x: pos.x,
            y: pos.y,
            w: 160.0,
            h: 60.0,
        };
        let mut content_origin = vec2(10.0, 10.0);
        let display_name = match &player.ptype {
            PlayerType::Local => None,
            PlayerType::MultiplayerHost { username }
            | PlayerType::MultiplayerClient { username, .. } => Some(username.clone()),
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
            match player.ptype {
                PlayerType::MultiplayerClient {
                    connection_state:
                        ConnectionState::Connected {
                            latency: Some(latency),
                        },
                    ..
                } => {
                    Text::new(format!("{}ms", latency.as_millis()))
                        .anchored_by(
                            ctx,
                            pos + vec2(card_rect.w, 0.0) + vec2(-10.0, 10.0),
                            AnchorPoint::NorthEast,
                        )?
                        .color(Color::BLACK)
                        .draw(canvas);
                }
                PlayerType::MultiplayerClient {
                    connection_state: ConnectionState::Disconnected,
                    ..
                } => {
                    Text::new("Disconnected")
                        .anchored_by(
                            ctx,
                            pos + vec2(card_rect.w, 0.0) + vec2(-10.0, 10.0),
                            AnchorPoint::NorthEast,
                        )?
                        .color(Color::from_rgb(128, 0, 0))
                        .draw(canvas);
                }
                _ => (),
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

        Ok(card_rect)
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
                    preview_location: None,
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

    fn skip_meeples(&mut self, ctx: &Context) -> GameResult<()> {
        debug!("skipping meeple placement");
        if let TurnPhase::MeeplePlacement { closed_groups, .. } = &self.state.turn_phase {
            let groups_to_close = closed_groups.clone();
            self.push_history()?;
            self.end_turn(ctx, groups_to_close);
        }
        Ok(())
    }

    fn end_game_immediately(&mut self, ctx: &Context) -> GameResult<()> {
        self.push_history()?;
        self.end_game(ctx);
        Ok(())
    }

    fn handle_event(&mut self, ctx: &mut Context, event: GameEvent) -> Result<(), GameError> {
        trace!("event = {event:?}");
        match event {
            GameEvent::MainEvent(event) => self.parent_channel.send(event).unwrap(),
            GameEvent::SkipMeeples => {
                self.skip_meeples(ctx)?;
                self.broadcast_action(GameMessage::SkipMeeples);
            }
            GameEvent::ClosePauseMenu => self.pause_menu = None,
            GameEvent::EndGame => {
                self.pause_menu = None;
                self.end_game_immediately(ctx)?;
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
            GameEvent::ReturnToLobby => {
                if let Some(action_channel) = &mut self.action_channel {
                    let _ = action_channel.send(GameAction::ReturnToLobby);
                }
            }
        }
        Ok(())
    }

    pub fn handle_message(&mut self, ctx: &mut Context, message: GameMessage) -> GameResult<()> {
        trace!("received {message:?}");
        match message {
            GameMessage::PlaceTile {
                selected_square,
                rotation,
            } => {
                if let TurnPhase::TilePlacement { tile, .. } = &mut self.state.turn_phase {
                    tile.rotate_to(rotation);
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
            GameMessage::SkipMeeples => self.skip_meeples(ctx)?,
            GameMessage::EndGame => self.end_game_immediately(ctx)?,
            GameMessage::Undo => {
                self.pop_history();
                self.reevaluate_selected_square();
            }
            GameMessage::PreviewTile(tile_preview) => {
                if let TurnPhase::TilePlacement {
                    preview_location,
                    tile,
                    ..
                } = &mut self.state.turn_phase
                {
                    if let Some(TilePreview {
                        selected_square,
                        rotation,
                    }) = tile_preview
                    {
                        *preview_location = Some(selected_square);
                        tile.rotate_to(rotation);
                    } else {
                        *preview_location = None;
                    }
                }
            }
        }
        Ok(())
    }

    pub fn update_pings(&mut self, users: Vec<User>) -> GameResult<()> {
        for player in self.state.game.players.values_mut() {
            let user_data = users.iter().find_map(|user| {
                user.client_info.as_ref().and_then(|client_info| {
                    player
                        .ptype
                        .matches_address(Some(client_info.ip))
                        .then_some((client_info.latency, user.username.clone()))
                })
            });

            if let PlayerType::MultiplayerClient {
                username,
                connection_state,
                ..
            } = &mut player.ptype
            {
                if let Some((new_latency, new_username)) = user_data {
                    *connection_state = ConnectionState::Connected {
                        latency: new_latency,
                    };
                    *username = new_username;
                } else {
                    *connection_state = ConnectionState::Disconnected;
                }
            }
        }
        Ok(())
    }

    fn place_tile(&mut self, ctx: &mut Context, focused_pos: GridPos) -> Result<(), GameError> {
        debug!("place_tile at {focused_pos:?}");
        self.push_history()?;

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
        debug!("player {player_ident:?} placing meeple at {seg_ident:?}");
        self.push_history()?;

        self.state.game.place_meeple(seg_ident, player_ident)?;
        self.selected_segment_and_group = None;
        self.end_turn(ctx, closed_groups);
        Ok(())
    }

    fn broadcast_action(&mut self, message: GameMessage) {
        trace!("sending {message:?}");
        if let Some(action_channel) = &mut self.action_channel {
            let _ = action_channel.send(GameAction::Message(message));
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

    fn get_current_player(&self) -> PlayerIdentifier {
        *self.state.turn_order.front().unwrap()
    }

    pub fn get_current_player_type(&self) -> &PlayerType {
        &self
            .state
            .game
            .players
            .get(self.get_current_player())
            .unwrap()
            .ptype
    }

    fn update_preview(&mut self) {
        if let TurnPhase::TilePlacement {
            tile: Tile { rotation, .. },
            ..
        } = self.state.turn_phase
        {
            self.broadcast_action(GameMessage::PreviewTile(self.selected_square.map(
                |selected_square| TilePreview {
                    selected_square,
                    rotation,
                },
            )));
        }
    }

    fn grid_selection_info(&self, ctx: &Context) -> GridSelectionInfo {
        let gameboard_pos = self.to_game_pos(ctx.mouse.position().into(), ctx);
        let focused_pos = GridPos::from(gameboard_pos);
        let subgrid_pos = gameboard_pos - Vec2::from(focused_pos);
        GridSelectionInfo {
            gameboard_pos,
            focused_pos,
            subgrid_pos,
        }
    }

    fn origin_rect(&self, ctx: &Context) -> Rect {
        self.grid_pos_rect(&GridPos(0, 0), ctx)
    }

    fn draw_group_inspection_ui(
        &mut self,
        ctx: &mut Context,
        canvas: &mut Canvas,
    ) -> Result<(), GameError> {
        if let Some(group_inspection) = &self.inspecting_groups {
            let origin_rect = self.origin_rect(ctx);
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
        }
        Ok(())
    }

    fn draw_turn_phase(&mut self, ctx: &mut Context, canvas: &mut Canvas) -> GameResult<()> {
        match &self.state.turn_phase {
            TurnPhase::TilePlacement { .. } => {
                if let Some(pos) = self.selected_square {
                    self.draw_held_tile_at_pos(ctx, canvas, pos)?;
                }
            }
            TurnPhase::MeeplePlacement {
                placed_position, ..
            } => {
                let rect = self.grid_pos_rect(placed_position, ctx);
                let time = ctx.time.time_since_start().as_secs_f32();
                let sin_time = time.sin() * 0.1 + 1.0;
                Mesh::new_rectangle(ctx, DrawMode::stroke(2.0), rect, Color::CYAN)?.draw(canvas);

                if self.ui.cursor_override.is_none() {
                    'draw_outline: {
                        if let Some((_, group_ident)) = self.selected_segment_and_group {
                            let origin_rect = self.origin_rect(ctx);
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
        Ok(())
    }

    fn draw_held_tile_at_pos(
        &mut self,
        ctx: &mut Context,
        canvas: &mut Canvas,
        pos: GridPos,
    ) -> Result<(), GameError> {
        let TurnPhase::TilePlacement { tile, .. } = &self.state.turn_phase else {
            return Ok(());
        };
        let rect = self.grid_pos_rect(&pos, ctx);
        let cursor_color = if self.is_placement_valid(pos) != PlacementValidity::Valid {
            Color::RED
        } else {
            Color::GREEN
        };
        if !self.state.game.placed_tiles.contains_key(&pos) {
            tile.render(ctx, canvas, rect)?;
        }
        Mesh::new_rectangle(ctx, DrawMode::stroke(2.0), rect, cursor_color)?.draw(canvas);
        Ok(())
    }

    fn draw_game_details(
        &mut self,
        ctx: &mut Context,
        canvas: &mut Canvas,
    ) -> Result<(), GameError> {
        let is_endgame = matches!(self.state.turn_phase, TurnPhase::EndGame { .. });
        let current_player_ident = self.get_current_player();
        if self.keybinds.detailed_view.pressed(ctx) || is_endgame {
            // draw player cards
            let mut card_location = vec2(20.0, 20.0);
            let mut cards_right_extent: f32 = 0.0;
            for &player_ident in &self.state.turn_order {
                let rect = self.draw_player_card(
                    ctx,
                    canvas,
                    player_ident,
                    card_location,
                    player_ident == current_player_ident && !is_endgame,
                )?;
                card_location.y += rect.h + 20.0;
                cards_right_extent = cards_right_extent.max(rect.right());
            }

            if !is_endgame {
                // draw remaining tile count
                let tile_count_rect = Rect::new(cards_right_extent + 20.0, 20.0, 60.0, 60.0);
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

                // draw controls cheatsheet
                let res = ctx.res();
                let cheatsheet_text = Text::new(
                    "Left mouse - Place tile / meeple    Right mouse - Drag    R - Rotate",
                )
                .size(16.0);
                let bounds: Vec2 = cheatsheet_text.measure(ctx)?.into();
                let width = bounds.x + 12.0;
                Mesh::new_rounded_rectangle(
                    ctx,
                    DrawMode::fill(),
                    Bounds {
                        relative: Rect::new(0.5, 1.0, 0.0, 0.0),
                        absolute: Rect::new(-width / 2.0, -30.0, width, 24.0),
                    }
                    .corrected_bounds(res),
                    5.0,
                    PANEL_COLOR,
                )?
                .draw(canvas);
                cheatsheet_text
                    .anchored_by(
                        ctx,
                        res * vec2(0.5, 1.0) + vec2(0.0, -10.0),
                        AnchorPoint::SouthCenter,
                    )?
                    .color(Color::WHITE)
                    .draw(canvas);
            }
        } else {
            // draw card of current player
            self.draw_player_card(ctx, canvas, current_player_ident, vec2(20.0, 20.0), false)?;
        }
        Ok(())
    }

    fn draw_meeples(&mut self, ctx: &mut Context, canvas: &mut Canvas) -> Result<(), GameError> {
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
            let meeple_scale = MEEPLE_SIZE / norm.x.max(norm.y);
            GameClient::draw_meeple(ctx, canvas, segment_meeple_spot, color, meeple_scale)?;
        }
        Ok(())
    }

    fn draw_scoring_effects(
        &mut self,
        ctx: &mut Context,
        canvas: &mut Canvas,
    ) -> Result<(), GameError> {
        let time = ctx.time.time_since_start().as_secs_f32();
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
        Ok(())
    }

    fn keyboard_movement_update(&mut self, ctx: &mut Context) -> GameResult<()> {
        // sliding
        let mut movement_vector: Vec2 = [
            (&self.keybinds.move_up, vec2(0.0, -1.0)),
            (&self.keybinds.move_up_alternate, vec2(0.0, -1.0)),
            (&self.keybinds.move_right, vec2(1.0, 0.0)),
            (&self.keybinds.move_right_alternate, vec2(1.0, 0.0)),
            (&self.keybinds.move_down, vec2(0.0, 1.0)),
            (&self.keybinds.move_down_alternate, vec2(0.0, 1.0)),
            (&self.keybinds.move_left, vec2(-1.0, 0.0)),
            (&self.keybinds.move_left_alternate, vec2(-1.0, 0.0)),
        ]
        .into_iter()
        .flat_map(|(binding, dir)| binding.pressed(ctx).then_some(dir))
        .sum();
        if movement_vector != Vec2::ZERO {
            movement_vector = movement_vector.normalize();
        }
        movement_vector = movement_vector * self.scale * MOVE_SPEED;
        if self.keybinds.move_faster.pressed(ctx) {
            movement_vector *= SPRINT_MOD;
        }
        self.camera_movement =
            self.camera_movement * (1.0 - MOVE_ACCEL) + movement_vector * MOVE_ACCEL;
        self.offset += self.camera_movement;

        // zooming
        let zoom_factor = match (
            self.keybinds.zoom_in.pressed(ctx),
            self.keybinds.zoom_out.pressed(ctx),
        ) {
            (true, false) => -KEYBOARD_ZOOM_SPEED,
            (false, true) => KEYBOARD_ZOOM_SPEED,
            _ => 0.0,
        };
        self.camera_zoom =
            self.camera_zoom * (1.0 - KEYBOARD_ZOOM_ACEL) + zoom_factor * KEYBOARD_ZOOM_ACEL;
        self.zoom(ctx, self.camera_zoom)
    }

    fn pause_menu_activation_update(&mut self, ctx: &mut Context) {
        if self.keybinds.pause.just_pressed(ctx) {
            self.pause_menu = Some(PauseScreenSubclient::new(
                self.shared.clone(),
                self.event_sender.clone(),
                self.is_endgame(),
                !self.history.is_empty() && self.can_play(),
            ));
        }
    }

    fn turn_phase_update(&mut self, ctx: &mut Context, on_clickable: &mut bool) -> GameResult<()> {
        let can_play = self.can_play();
        let GridSelectionInfo {
            gameboard_pos,
            focused_pos,
            subgrid_pos,
        } = self.grid_selection_info(ctx);

        match &self.state.turn_phase {
            TurnPhase::TilePlacement {
                placeable_positions,
                ..
            } => {
                // determine tile placement location
                if !can_play {
                    self.set_selected_square(None);
                    return Ok(());
                }

                if self.shared.args.snap_placement {
                    let gameboard_pos = gameboard_pos - vec2(0.5, 0.5);
                    self.set_selected_square(
                        placeable_positions
                            .iter()
                            .min_by_f32_key(|pos| Vec2::from(**pos).distance_squared(gameboard_pos))
                            .cloned(),
                    );
                } else {
                    self.set_selected_square(Some(focused_pos));
                }

                // rotate tile
                if self.keybinds.rotate_clockwise.just_pressed(ctx) {
                    self.get_held_tile_mut().unwrap().rotate_clockwise();
                    self.reevaluate_selected_square();
                    self.update_preview();
                }

                // rotate tile counterclockwise (dont tell anyone it's actually just three clockwise rotations)
                if self.keybinds.rotate_counterclockwise.just_pressed(ctx) {
                    self.get_held_tile_mut().unwrap().rotate_counterclockwise();
                    self.reevaluate_selected_square_counterclockwise();
                    self.update_preview();
                }

                // place tile
                if self.keybinds.place_tile.just_pressed(ctx) && self.placement_is_valid {
                    if let Some(selected_square) = self.selected_square {
                        let rotation = self.get_held_tile_mut().unwrap().rotation;
                        self.place_tile(ctx, selected_square)?;
                        self.broadcast_action(GameMessage::PlaceTile {
                            selected_square,
                            rotation,
                        });
                    }
                }

                *on_clickable = self.placement_is_valid;
            }
            TurnPhase::MeeplePlacement {
                placed_position,
                closed_groups,
            } => {
                self.selected_segment_and_group = None;

                if !can_play {
                    return Ok(());
                }

                if *placed_position == focused_pos {
                    self.selected_segment_and_group =
                        self.get_selected_segment(focused_pos, placed_position, subgrid_pos);

                    *on_clickable = self.selected_segment_and_group.is_some();

                    if self.keybinds.place_meeple.just_pressed(ctx) {
                        if let Some((seg_ident, Some(group))) =
                            self.selected_segment_and_group
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
                                self.broadcast_action(GameMessage::PlaceMeeple { seg_ident });
                            }
                        }
                    }
                }

                if self.keybinds.skip_meeples.just_pressed(ctx) {
                    self.skip_meeples(ctx)?;
                    self.broadcast_action(GameMessage::SkipMeeples);
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
        Ok(())
    }

    fn group_inspection_update(&mut self, ctx: &mut Context) {
        let GridSelectionInfo {
            focused_pos,
            subgrid_pos,
            ..
        } = self.grid_selection_info(ctx);

        self.inspecting_groups = (self.keybinds.detailed_view.pressed(ctx) || self.is_endgame())
            .then(|| GroupInspection {
                selected_group: self
                    .state
                    .game
                    .placed_tiles
                    .get(&focused_pos)
                    .and_then(|tile| {
                        (0..tile.segments.len()).find(|seg_index| {
                            tile.segments[*seg_index].stype.placeable() && {
                                let segment_poly: Vec<_> =
                                    tile.segment_polygon(*seg_index).collect();
                                point_in_polygon(subgrid_pos, &segment_poly)
                            }
                        })
                    })
                    .and_then(|i| self.state.game.group_associations.get(&(focused_pos, i)))
                    .cloned(),
            });
    }

    fn draw_player_color_outline(
        &mut self,
        ctx: &mut Context,
        canvas: &mut Canvas,
    ) -> Result<(), GameError> {
        let current_player_ident = self.get_current_player();
        let res = ctx.res();
        if self.can_play() {
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
        };
        Ok(())
    }

    fn pause_menu_update(&mut self, ctx: &mut Context) -> Result<bool, GameError> {
        let is_endgame = self.is_endgame();
        let can_play = self.can_play();
        let pause_menu_open = if let Some(pause_menu) = &mut self.pause_menu {
            pause_menu.can_end_game.set(is_endgame);
            pause_menu
                .can_undo
                .set(can_play && !self.history.is_empty());
            pause_menu.update(ctx)?;
            self.set_selected_square(None);
            true
        } else {
            false
        };
        Ok(pause_menu_open)
    }

    fn set_selected_square(&mut self, new_selected_square: Option<GridPos>) {
        if self.selected_square != new_selected_square {
            self.selected_square = new_selected_square;
            self.reevaluate_selected_square();
            self.update_preview();
        }
    }

    fn zoom(&mut self, ctx: &Context, factor: f32) -> GameResult<()> {
        if self.pause_menu.is_some() {
            return Ok(());
        }

        // zooming
        let prev_scale = self.scale;
        self.scale *= ZOOM_SPEED.powf(factor);
        self.scale = self.scale.clamp(0.01, 1.0);
        let scale_change = self.scale / prev_scale;
        let mouse: Vec2 = ctx.mouse.position().into();
        self.offset = (self.offset + mouse) * scale_change - mouse;

        Ok(())
    }
}

impl SubEventHandler<GameError> for GameClient {
    fn mouse_wheel_event(&mut self, ctx: &mut Context, _x: f32, y: f32) -> Result<(), GameError> {
        self.zoom(ctx, y)
    }

    fn update(&mut self, ctx: &mut Context) -> GameResult {
        self.ui.update(ctx)?;
        while let Ok(event) = self.event_receiver.try_recv() {
            self.handle_event(ctx, event)?;
        }

        let mut on_clickable = false;

        if self.pause_menu_update(ctx)? {
            return Ok(());
        }

        // update button states
        self.skip_meeples_button.borrow_mut().state = UIElementState::invisible_if(
            !matches!(self.state.turn_phase, TurnPhase::MeeplePlacement { .. }) || !self.can_play(),
        );
        self.return_to_main_menu_button.borrow_mut().state =
            UIElementState::invisible_if(!matches!(
                self.state.turn_phase,
                TurnPhase::EndGame { next_tick: None }
            ));

        // dragging
        if self.keybinds.drag_camera.pressed(ctx) {
            self.offset -= Vec2::from(ctx.mouse.delta());
        }

        self.keyboard_movement_update(ctx)?;

        self.pause_menu_activation_update(ctx);

        // update scoring effects
        self.scoring_effects.retain(|effect| {
            ctx.time.time_since_start().as_secs_f32() - effect.initialized_at < SCORE_EFFECT_LIFE
        });

        self.group_inspection_update(ctx);

        if self.inspecting_groups.is_none() || self.is_endgame() {
            self.turn_phase_update(ctx, &mut on_clickable)?;
        } else {
            self.set_selected_square(None);
        }

        if on_clickable {
            set_cursor_type(ctx, CursorIcon::Pointer);
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context, canvas: &mut Canvas) -> GameResult {
        for (pos, tile) in &self.state.game.placed_tiles {
            tile.render(ctx, canvas, self.grid_pos_rect(pos, ctx))?;
        }

        self.draw_meeples(ctx, canvas)?;

        self.draw_scoring_effects(ctx, canvas)?;

        if self.inspecting_groups.is_none() {
            self.draw_turn_phase(ctx, canvas)?;
        }

        // draw tile preview
        if let TurnPhase::TilePlacement {
            preview_location: Some(pos),
            ..
        } = self.state.turn_phase
        {
            self.draw_held_tile_at_pos(ctx, canvas, pos)?;
        }

        self.draw_group_inspection_ui(ctx, canvas)?;

        self.ui.draw(ctx, canvas)?;

        self.draw_game_details(ctx, canvas)?;

        self.draw_player_color_outline(ctx, canvas)?;

        if let Some(pause_menu) = &mut self.pause_menu {
            pause_menu.draw(ctx, canvas)?;
        }

        Ok(())
    }
}

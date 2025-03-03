use std::{
    net::IpAddr,
    sync::mpsc::{channel, Receiver, Sender},
};

use ggez::{graphics::Canvas, Context, GameError};
use lobby_client::{LobbyClient, LobbyEvent};
use message::server::User;

use crate::{
    game::player::PlayerType,
    game_client::{
        GameAction, GameClient, GameClientConfiguration, GameExpansions, GameState,
        PlayerConfiguration,
    },
    main_client::MainEvent,
    Shared,
};
use ggez_no_re::sub_event_handler::SubEventHandler;

pub mod host_client;
pub mod join_client;
mod lobby_client;
pub mod message;
pub mod multiplayer_menu;

enum MultiplayerPhase<T> {
    Lobby(LobbyClient<T>),
    Game {
        game: GameClient,
        action_channel: Receiver<GameAction>,
    },
}

impl<T> MultiplayerPhase<T> {
    pub fn new_game(
        ctx: &Context,
        shared: Shared,
        parent_channel: Sender<MainEvent>,
        users: Vec<User>,
        seed: u64,
        local_user: Option<IpAddr>,
        local_username: String,
        expansions: GameExpansions,
    ) -> MultiplayerPhase<T> {
        let (action_sender, action_channel) = channel();
        let local_player = PlayerType::from_details(local_username, local_user);
        let players = users
            .iter()
            .map(|user| {
                let address = user.client_info.as_ref().map(|info| info.ip);
                (
                    user.color.unwrap(),
                    PlayerType::from_details(user.username.clone(), address),
                )
            })
            .collect();
        MultiplayerPhase::Game {
            game: GameClient::new(
                ctx,
                shared,
                parent_channel,
                Some(action_sender),
                GameClientConfiguration {
                    seed,
                    expansions,
                    players: PlayerConfiguration::Multiplayer {
                        local_player,
                        players,
                    },
                },
            ),
            action_channel,
        }
    }

    pub fn new_from_state(
        ctx: &Context,
        shared: Shared,
        parent_channel: Sender<MainEvent>,
        mut state: GameState,
        local_user: Option<IpAddr>,
        local_username: String,
    ) -> MultiplayerPhase<T> {
        state.game.local_player = PlayerType::from_details(local_username, local_user);
        let (action_sender, action_channel) = channel();
        MultiplayerPhase::Game {
            game: GameClient::new_from_state(
                ctx,
                shared,
                state,
                parent_channel,
                Some(action_sender),
            ),
            action_channel,
        }
    }
}

impl<T> SubEventHandler for MultiplayerPhase<T>
where
    T: From<LobbyEvent>,
{
    fn mouse_wheel_event(&mut self, ctx: &mut Context, x: f32, y: f32) -> Result<(), GameError> {
        use MultiplayerPhase::*;
        match self {
            Lobby(lobby_client) => lobby_client.mouse_wheel_event(ctx, x, y),
            Game { game, .. } => game.mouse_wheel_event(ctx, x, y),
        }
    }

    fn update(&mut self, ctx: &mut Context) -> Result<(), GameError> {
        use MultiplayerPhase::*;
        match self {
            Lobby(lobby_client) => lobby_client.update(ctx),
            Game { game, .. } => game.update(ctx),
        }
    }

    fn draw(&mut self, ctx: &mut Context, canvas: &mut Canvas) -> Result<(), GameError> {
        use MultiplayerPhase::*;
        match self {
            Lobby(lobby_client) => lobby_client.draw(ctx, canvas),
            Game { game, .. } => game.draw(ctx, canvas),
        }
    }
}

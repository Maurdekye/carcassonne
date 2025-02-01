use std::{
    net::IpAddr,
    sync::mpsc::{channel, Receiver, Sender},
};

use ggez::{graphics::Canvas, Context, GameError};
use lobby_client::{LobbyClient, LobbyEvent};
use rand::{rngs::StdRng, seq::SliceRandom, SeedableRng};
use transport::message::{server::User, GameMessage};

use crate::{
    game::{player::Player, Game},
    game_client::GameClient,
    main_client::MainEvent,
    pos::GridPos,
    sub_event_handler::SubEventHandler,
    tile::{tile_definitions::STARTING_TILE, Tile},
    Args,
};

pub mod host_client;
pub mod join_client;
mod lobby_client;
pub mod transport;

enum MultiplayerPhase<T> {
    Lobby(LobbyClient<T>),
    Game {
        game: GameClient,
        action_channel: Receiver<GameMessage>,
    },
}

impl<T> MultiplayerPhase<T> {
    pub fn new_game(
        ctx: &Context,
        args: Args,
        parent_channel: Sender<MainEvent>,
        users: Vec<User>,
        seed: u64,
        local_user: Option<IpAddr>,
    ) -> MultiplayerPhase<T> {
        let mut library = Tile::default_library();
        library.shuffle(&mut StdRng::seed_from_u64(seed));
        let mut game = Game::new_inner(library, local_user.into());
        for user in users {
            game.players.insert(Player::new_inner(
                user.color.unwrap(),
                user.client_info.as_ref().map(|info| info.ip).into(),
            ));
        }
        let (action_sender, action_channel) = channel();
        game.place_tile(STARTING_TILE.clone(), GridPos(0, 0))
            .unwrap();
        MultiplayerPhase::Game {
            game: GameClient::new_inner(ctx, args, game, parent_channel, Some(action_sender)),
            action_channel,
        }
    }
}

impl<T> SubEventHandler<GameError> for MultiplayerPhase<T>
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

use std::time::Duration;

use ggez::{graphics::Canvas, Context, GameError};
use lobby_client::LobbyClient;

use crate::{game_client::GameClient, sub_event_handler::SubEventHandler};

mod lobby_client;
pub mod host_client;
pub mod join_client;
pub mod transport;

const PING_FREQUENCY: Duration = Duration::from_secs(2);

enum MultiplayerPhase {
    Lobby(LobbyClient),
    Game(GameClient),
}

impl SubEventHandler<GameError> for MultiplayerPhase {
    fn update(&mut self, ctx: &mut Context) -> Result<(), GameError> {
        use MultiplayerPhase::*;
        match self {
            Lobby(lobby_client) => lobby_client.update(ctx),
            Game(game_client) => game_client.update(ctx),
        }
    }

    fn draw(&mut self, ctx: &mut Context, canvas: &mut Canvas) -> Result<(), GameError> {
        use MultiplayerPhase::*;
        match self {
            Lobby(lobby_client) => lobby_client.draw(ctx, canvas),
            Game(game_client) => game_client.draw(ctx, canvas),
        }
    }
}
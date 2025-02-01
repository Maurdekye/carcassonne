use ggez::graphics::Color;
use serde::{Deserialize, Serialize};

use super::GameMessage;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ClientMessage {
    Ping,
    Pong,
    Lobby(LobbyMessage),
    Game(GameMessage),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum LobbyMessage {
    ChooseColor(Option<Color>),
}

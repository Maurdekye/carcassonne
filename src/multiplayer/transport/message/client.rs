use ggez::graphics::Color;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ClientMessage {
    Ping,
    Pong,
    Lobby(LobbyMessage),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum LobbyMessage {
    ChooseColor(Option<Color>),
}

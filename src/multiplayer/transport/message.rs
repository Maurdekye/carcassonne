use ggez::graphics::Color;
use serde::{Deserialize, Serialize};

use std::time::Duration;

use std::net::IpAddr;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ClientInfo {
    pub ip: IpAddr,
    pub latency: Option<Duration>
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct User {
    pub client_info: Option<ClientInfo>,
    pub color: Option<Color>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LobbyState {
    pub users: Vec<User>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum LobbyMessage {
    ChooseColor(Color),
    LobbyState(LobbyState),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Message {
    Ping,
    Pong,
    Lobby(LobbyMessage),
}

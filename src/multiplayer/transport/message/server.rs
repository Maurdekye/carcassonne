use ggez::graphics::Color;
use serde::{Deserialize, Serialize};

use std::time::Duration;

use std::net::IpAddr;

use crate::game::player::PlayerType;

use super::GameMessage;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ServerMessage {
    Ping,
    Pong,
    Lobby(LobbyMessage),
    Game { message: GameMessage, user: PlayerType },
    StartGame { game_seed: u64 },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum LobbyMessage {
    LobbyState(LobbyState),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LobbyState {
    pub users: Vec<User>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ClientInfo {
    pub ip: IpAddr,
    pub latency: Option<Duration>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct User {
    pub client_info: Option<ClientInfo>,
    pub color: Option<Color>,
}

impl User {
    pub fn ip(&self) -> Option<&IpAddr> {
        self.client_info.as_ref().map(|info| &info.ip)
    }
}

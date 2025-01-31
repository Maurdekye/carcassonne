use ggez::graphics::Color;
use serde::{Deserialize, Serialize};

use std::time::Duration;

use std::net::IpAddr;

pub mod client;
pub mod server;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Message {
    Client(client::ClientMessage),
    Server(server::ServerMessage),
}

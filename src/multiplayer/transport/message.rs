use std::net::SocketAddr;

use client::ClientMessage;
use serde::{Deserialize, Serialize};
use server::ServerMessage;

use crate::game::SegmentIdentifier;
use crate::pos::GridPos;

pub mod client;
pub mod server;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Message {
    Client(ClientMessage),
    Server(ServerMessage),
    YourSocket(SocketAddr),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum GameMessage {
    PlaceTile {
        selected_square: GridPos,
        rotation: usize,
    },
    PreviewTile(Option<TilePreview>),
    PlaceMeeple {
        seg_ident: SegmentIdentifier,
    },
    SkipMeeples,
    EndGame,
    Undo,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TilePreview {
    pub selected_square: GridPos,
    pub rotation: usize,
}

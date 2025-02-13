use client::ClientMessage;
use ggez_no_re::transport::ClientServerMessage;
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
}

impl ClientServerMessage for Message {
    type ClientMessage = ClientMessage;
    type ServerMessage = ServerMessage;
}

impl From<ClientMessage> for Message {
    fn from(value: ClientMessage) -> Self {
        Message::Client(value)
    }
}

impl From<ServerMessage> for Message {
    fn from(value: ServerMessage) -> Self {
        Message::Server(value)
    }
}

impl TryFrom<Message> for ClientMessage {
    type Error = ();

    fn try_from(value: Message) -> Result<Self, Self::Error> {
        match value {
            Message::Client(client_message) => Ok(client_message),
            _ => Err(()),
        }
    }
}

impl TryFrom<Message> for ServerMessage {
    type Error = ();

    fn try_from(value: Message) -> Result<Self, Self::Error> {
        match value {
            Message::Server(server_message) => Ok(server_message),
            _ => Err(()),
        }
    }
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

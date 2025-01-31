use std::{
    cell::LazyCell,
    net::SocketAddr,
    sync::mpsc::{channel, Receiver, Sender},
    time::{Duration, Instant},
};

use ggez::{
    glam::{vec2, Vec2},
    graphics::{Color, Text},
    Context, GameError, GameResult,
};

use crate::{
    game_client::GameClient,
    main_client::MainEvent,
    multiplayer::transport::message::{LobbyMessage, Message},
    sub_event_handler::SubEventHandler,
    ui_manager::UIManager,
    util::{AnchorPoint, ContextExt, TextExt},
    Args,
};

use super::{
    lobby_client::LobbyClient,
    transport::{MessageClient, MessageTransporter, NetworkEvent},
    MultiplayerPhase, PING_FREQUENCY,
};

#[derive(Clone)]
enum UIEvent {}

enum MultiplayerJoinMenuEvent {
    MainEvent(MainEvent),
    NetworkEvent(NetworkEvent),
    UIEvent(UIEvent),
}

impl From<UIEvent> for MultiplayerJoinMenuEvent {
    fn from(value: UIEvent) -> Self {
        MultiplayerJoinMenuEvent::UIEvent(value)
    }
}

impl From<NetworkEvent> for MultiplayerJoinMenuEvent {
    fn from(value: NetworkEvent) -> Self {
        MultiplayerJoinMenuEvent::NetworkEvent(value)
    }
}

pub struct MultiplayerJoinMenuClient {
    parent_channel: Sender<MainEvent>,
    _event_sender: Sender<MultiplayerJoinMenuEvent>,
    event_receiver: Receiver<MultiplayerJoinMenuEvent>,
    args: Args,
    ui: UIManager<UIEvent, MultiplayerJoinMenuEvent>,
    _message_client: MessageClient,
    connection: Option<MessageTransporter>,
    last_ping: Instant,
    latency: Option<Duration>,
    phase: Option<MultiplayerPhase>,
}

impl MultiplayerJoinMenuClient {
    pub fn new(parent_channel: Sender<MainEvent>, args: Args, socket: SocketAddr) -> Self {
        let (event_sender, event_receiver) = channel();
        let ui = UIManager::new(event_sender.clone(), []);
        let message_client = MessageClient::start(event_sender.clone(), socket);
        MultiplayerJoinMenuClient {
            parent_channel,
            _event_sender: event_sender,
            event_receiver,
            ui,
            args,
            _message_client: message_client,
            connection: None,
            last_ping: Instant::now(),
            latency: None,
            phase: None,
        }
    }
    fn handle_event(
        &mut self,
        _ctx: &mut Context,
        event: MultiplayerJoinMenuEvent,
    ) -> GameResult<()> {
        match event {
            MultiplayerJoinMenuEvent::MainEvent(main_event) => {
                self.parent_channel.send(main_event).unwrap()
            }
            MultiplayerJoinMenuEvent::NetworkEvent(network_event) => match network_event {
                NetworkEvent::Connect(transport) => {
                    println!("Connected!");
                    self.connection = Some(transport);
                    self.phase = Some(MultiplayerPhase::Lobby(LobbyClient::new(Vec::new())));
                }
                NetworkEvent::Message(server_message) => {
                    dbg!(&server_message);
                    let mut server = LazyCell::new(|| self.connection.as_mut().unwrap());
                    match server_message {
                        Message::Pong => {
                            let now = Instant::now();
                            self.latency = Some(now - self.last_ping);
                        }
                        Message::Ping => {
                            LazyCell::force_mut(&mut server).blind_send(&Message::Pong)
                        }
                        Message::Lobby(lobby_message) => {
                            if let Some(MultiplayerPhase::Lobby(lobby)) = &mut self.phase {
                                lobby.handle_message(lobby_message)?;
                            }
                        }
                        _ => {}
                    }
                }
                NetworkEvent::Disconnect => {
                    println!("Disconnected.");
                    self.connection = None;
                    self.phase = None;
                }
            },
            MultiplayerJoinMenuEvent::UIEvent(_uievent) => {}
        }
        Ok(())
    }
}

impl SubEventHandler<GameError> for MultiplayerJoinMenuClient {
    fn update(&mut self, ctx: &mut ggez::Context) -> Result<(), GameError> {
        self.ui.update(ctx)?;
        while let Ok(event) = self.event_receiver.try_recv() {
            self.handle_event(ctx, event)?;
        }

        if let Some(phase) = &mut self.phase {
            phase.update(ctx)?;
        }

        if let Some(connection) = &mut self.connection {
            let now = Instant::now();
            if now - self.last_ping > PING_FREQUENCY {
                connection.blind_send(&Message::Ping);
                self.last_ping = now;
            }
        }

        Ok(())
    }

    fn draw(
        &mut self,
        ctx: &mut ggez::Context,
        canvas: &mut ggez::graphics::Canvas,
    ) -> Result<(), GameError> {
        match &mut self.phase {
            None => {
                Text::new(format!(
                    "Connecting to {}:{}...",
                    self.args.ip.unwrap(),
                    self.args.port
                ))
                .size(36.0)
                .anchored_by(
                    ctx,
                    ctx.res() * vec2(0.5, 0.0) + vec2(0.0, 30.0),
                    AnchorPoint::NorthCenter,
                )?
                .color(Color::BLACK)
                .draw(canvas);
            }
            Some(MultiplayerPhase::Lobby(lobby)) => {
                Text::new("Connected")
                    .size(36.0)
                    .anchored_by(
                        ctx,
                        ctx.res() * vec2(0.5, 0.0) + vec2(0.0, 30.0),
                        AnchorPoint::NorthCenter,
                    )?
                    .color(Color::BLACK)
                    .draw(canvas);

                lobby.draw(ctx, canvas)?;
            }
            Some(MultiplayerPhase::Game(game)) => {
                game.draw(ctx, canvas)?;
            }
        }

        if let Some(latency) = self.latency {
            Text::new(format!("{} ms", latency.as_millis()))
                .size(16.0)
                .anchored_by(ctx, vec2(10.0, 10.0), AnchorPoint::NorthWest)?
                .color(Color::from_rgb(160, 160, 160))
                .draw(canvas);
        }

        self.ui.draw(ctx, canvas)
    }
}

impl Drop for MultiplayerJoinMenuClient {
    fn drop(&mut self) {
        if let Some(mut connection) = self.connection.take() {
            let _ = connection.shutdown();
        }
    }
}

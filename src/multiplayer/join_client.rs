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
    multiplayer::transport::message::{client::ClientMessage, server::ServerMessage, Message},
    sub_event_handler::SubEventHandler,
    ui_manager::UIManager,
    util::{AnchorPoint, ContextExt, TextExt},
    Args,
};

use super::{
    lobby_client::{LobbyClient, LobbyEvent},
    transport::{
        message::client::{self, LobbyMessage}, ClientNetworkEvent, ClientsideTransport, MessageClient, MessageTransporter, NetworkEvent
    },
    MultiplayerPhase, PING_FREQUENCY,
};

#[derive(Clone)]
enum UIEvent {}

enum JoinEvent {
    MainEvent(MainEvent),
    NetworkEvent(ClientNetworkEvent),
    UIEvent(UIEvent),
    LobbyEvent(LobbyEvent),
}

impl From<UIEvent> for JoinEvent {
    fn from(value: UIEvent) -> Self {
        JoinEvent::UIEvent(value)
    }
}

impl From<ClientNetworkEvent> for JoinEvent {
    fn from(value: ClientNetworkEvent) -> Self {
        JoinEvent::NetworkEvent(value)
    }
}

impl From<LobbyEvent> for JoinEvent {
    fn from(value: LobbyEvent) -> Self {
        JoinEvent::LobbyEvent(value)
    }
}

pub struct JoinClient {
    parent_channel: Sender<MainEvent>,
    event_sender: Sender<JoinEvent>,
    event_receiver: Receiver<JoinEvent>,
    args: Args,
    ui: UIManager<UIEvent, JoinEvent>,
    _message_client: MessageClient,
    connection: Option<ClientsideTransport>,
    last_ping: Instant,
    latency: Option<Duration>,
    phase: Option<MultiplayerPhase<JoinEvent>>,
}

impl JoinClient {
    pub fn new(parent_channel: Sender<MainEvent>, args: Args, socket: SocketAddr) -> Self {
        let (event_sender, event_receiver) = channel();
        let ui = UIManager::new(event_sender.clone(), []);
        let message_client = MessageClient::start(event_sender.clone(), socket);
        JoinClient {
            parent_channel,
            event_sender,
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
    fn handle_event(&mut self, _ctx: &mut Context, event: JoinEvent) -> GameResult<()> {
        match event {
            JoinEvent::MainEvent(main_event) => self.parent_channel.send(main_event).unwrap(),
            JoinEvent::NetworkEvent(network_event) => match network_event {
                NetworkEvent::Connect(transport) => {
                    println!("Connected!");
                    self.phase = Some(MultiplayerPhase::Lobby(LobbyClient::new(
                        Vec::new(),
                        Some(dbg!(transport.local_addr().unwrap().ip())),
                        self.event_sender.clone(),
                    )));
                    self.connection = Some(transport);
                }
                NetworkEvent::Message(server_message) => {
                    dbg!(&server_message);
                    let mut server = LazyCell::new(|| self.connection.as_mut().unwrap());
                    match server_message {
                        ServerMessage::Pong => {
                            let now = Instant::now();
                            self.latency = Some(now - self.last_ping);
                        }
                        ServerMessage::Ping => {
                            LazyCell::force_mut(&mut server).blind_send(ClientMessage::Pong)
                        }
                        ServerMessage::Lobby(lobby_message) => {
                            if let Some(MultiplayerPhase::Lobby(lobby)) = &mut self.phase {
                                lobby.handle_message(lobby_message)?;
                            }
                        }
                    }
                }
                NetworkEvent::Disconnect => {
                    println!("Disconnected.");
                    self.connection = None;
                    self.latency = None;
                    self.phase = None;
                }
            },
            JoinEvent::UIEvent(_uievent) => {}
            JoinEvent::LobbyEvent(LobbyEvent::ChooseColor(color)) => {
                if let Some(connection) = &mut self.connection {
                    connection
                        .blind_send(ClientMessage::Lobby(client::LobbyMessage::ChooseColor(color)));
                }
            }
        }
        Ok(())
    }
}

impl SubEventHandler<GameError> for JoinClient {
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
                connection.blind_send(ClientMessage::Ping);
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

impl Drop for JoinClient {
    fn drop(&mut self) {
        if let Some(mut connection) = self.connection.take() {
            let _ = connection.shutdown();
        }
    }
}

use std::{
    cell::LazyCell,
    net::{IpAddr, SocketAddr},
    sync::mpsc::{channel, Receiver, Sender},
    time::{Duration, Instant},
};

use ggez::{
    glam::vec2,
    graphics::{Color, Text},
    Context, GameError, GameResult,
};

use crate::{
    main_client::MainEvent,
    multiplayer::transport::message::{
        client::ClientMessage,
        server::{self, LobbyState, ServerMessage},
    },
    sub_event_handler::SubEventHandler,
    ui_manager::UIManager,
    util::{AnchorPoint, ContextExt, TextExt},
    Args,
};

use super::{
    lobby_client::{LobbyClient, LobbyEvent},
    transport::{
        message::{
            client::{self},
            server::User,
        },
        ClientNetworkEvent, ClientsideTransport, MessageClient, NetworkEvent,
    },
    MultiplayerPhase,
};

#[derive(Clone)]
enum UIEvent {}

#[allow(clippy::enum_variant_names)]
enum JoinEvent {
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
    connection: Option<(ClientsideTransport, IpAddr)>,
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

    fn start_game(&mut self, ctx: &Context, users: Vec<User>, seed: u64) {
        self.phase = Some(MultiplayerPhase::new_game(
            ctx,
            self.args.clone(),
            self.parent_channel.clone(),
            users,
            seed,
            Some(self.connection.as_ref().unwrap().1),
        ));
    }

    fn handle_event(&mut self, ctx: &mut Context, event: JoinEvent) -> GameResult<()> {
        match event {
            JoinEvent::NetworkEvent(network_event) => match network_event {
                NetworkEvent::Connect {
                    transport,
                    my_socket_addr,
                } => {
                    println!("connected");
                    self.phase = Some(MultiplayerPhase::Lobby(LobbyClient::new(
                        Vec::new(),
                        Some(my_socket_addr.ip()),
                        self.event_sender.clone(),
                    )));
                    self.connection = Some((transport, my_socket_addr.ip()));
                }
                NetworkEvent::Message(server_message) => {
                    println!("{server_message:?}");
                    let mut server = LazyCell::new(|| &mut self.connection.as_mut().unwrap().0);
                    match server_message {
                        ServerMessage::Pong => {
                            let now = Instant::now();
                            self.latency = Some(now - self.last_ping);
                        }
                        ServerMessage::Ping => {
                            LazyCell::force_mut(&mut server).blind_send(ClientMessage::Pong)
                        }
                        ServerMessage::Lobby(lobby_message) => {
                            match (&mut self.phase, lobby_message) {
                                (Some(MultiplayerPhase::Lobby(lobby)), lobby_message) => {
                                    lobby.handle_message(lobby_message)?;
                                }
                                (
                                    Some(MultiplayerPhase::Game { game, .. }),
                                    server::LobbyMessage::LobbyState(LobbyState { users }),
                                ) => {
                                    game.update_pings(users)?;
                                }
                                _ => {}
                            }
                        }
                        ServerMessage::StartGame { game_seed } => {
                            if let Some(MultiplayerPhase::Lobby(lobby)) = &self.phase {
                                self.start_game(ctx, lobby.users.clone(), game_seed);
                            }
                        }
                        ServerMessage::Game { message, user } => {
                            if let Some(MultiplayerPhase::Game { game, .. }) = &mut self.phase {
                                if game.get_current_player() == user {
                                    game.handle_message(ctx, message)?;
                                }
                            }
                        }
                    }
                }
                NetworkEvent::Disconnect => {
                    println!("disconnected");
                    self.connection = None;
                    self.latency = None;
                    self.phase = None;
                }
            },
            JoinEvent::UIEvent(_uievent) => {}
            JoinEvent::LobbyEvent(LobbyEvent::ChooseColor(color)) => {
                if let Some((connection, _)) = &mut self.connection {
                    connection.blind_send(ClientMessage::Lobby(client::LobbyMessage::ChooseColor(
                        color,
                    )));
                }
            }
        }
        Ok(())
    }
}

impl SubEventHandler<GameError> for JoinClient {
    fn mouse_wheel_event(&mut self, ctx: &mut Context, x: f32, y: f32) -> Result<(), GameError> {
        if let Some(phase) = &mut self.phase {
            phase.mouse_wheel_event(ctx, x, y)?;
        }
        Ok(())
    }

    fn update(&mut self, ctx: &mut ggez::Context) -> Result<(), GameError> {
        self.ui.update(ctx)?;
        while let Ok(event) = self.event_receiver.try_recv() {
            self.handle_event(ctx, event)?;
        }

        if let Some(phase) = &mut self.phase {
            phase.update(ctx)?;
            if let (MultiplayerPhase::Game { action_channel, .. }, Some((connection, _))) =
                (phase, &mut self.connection)
            {
                while let Ok(message) = action_channel.try_recv() {
                    connection.blind_send(ClientMessage::Game(message));
                }
            }
        }

        if let Some((connection, _)) = &mut self.connection {
            let now = Instant::now();
            if now - self.last_ping > self.args.ping_interval {
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
            Some(MultiplayerPhase::Game { game, .. }) => {
                game.draw(ctx, canvas)?;
            }
        }

        if let Some(latency) = self.latency {
            Text::new(format!("{} ms", latency.as_millis()))
                .size(16.0)
                .anchored_by(ctx, vec2(2.0, 2.0), AnchorPoint::NorthWest)?
                .color(Color::from_rgb(160, 160, 160))
                .draw(canvas);
        }

        self.ui.draw(ctx, canvas)
    }
}

impl Drop for JoinClient {
    fn drop(&mut self) {
        if let Some((mut connection, _)) = self.connection.take() {
            let _ = connection.shutdown();
        }
    }
}

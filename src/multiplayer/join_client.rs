use std::{
    cell::{LazyCell, RefCell},
    net::{IpAddr, SocketAddr},
    rc::Rc,
    sync::mpsc::{channel, Receiver, Sender},
    time::{Duration, Instant},
};

use ggez::{
    glam::vec2,
    graphics::{Canvas, Color, Rect, Text},
    Context, GameError, GameResult,
};
use log::{debug, info, trace};

use crate::{
    main_client::MainEvent,
    multiplayer::transport::message::{
        client::ClientMessage,
        server::{self, LobbyState, ServerMessage},
    },
    sub_event_handler::SubEventHandler,
    ui_manager::{Bounds, Button, UIElement, UIElementState, UIManager},
    util::{AnchorPoint, ContextExt, TextExt},
    SharedResources,
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

#[derive(Clone, Debug)]
enum UIEvent {
    MainEvent(MainEvent),
}

#[allow(clippy::enum_variant_names)]
#[derive(Debug)]
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
    shared: SharedResources,
    ui: UIManager<UIEvent, JoinEvent>,
    _message_client: MessageClient,
    connection: Option<(ClientsideTransport, IpAddr)>,
    last_ping: Instant,
    latency: Option<Duration>,
    phase: Option<MultiplayerPhase<JoinEvent>>,
    socket: SocketAddr,
    back_button: Rc<RefCell<Button<UIEvent>>>,
}

impl JoinClient {
    pub fn new(parent_channel: Sender<MainEvent>, shared: SharedResources, socket: SocketAddr) -> Self {
        let (event_sender, event_receiver) = channel();
        let (ui, [UIElement::Button(back_button)]) = UIManager::new_and_rc_elements(
            event_sender.clone(),
            [UIElement::Button(Button::new(
                Bounds::absolute(Rect::new(30.0, 30.0, 120.0, 40.0)),
                Text::new("Cancel"),
                UIEvent::MainEvent(MainEvent::MultiplayerMenu),
            ))],
        ) else {
            panic!()
        };
        let message_client = MessageClient::start(event_sender.clone(), socket);
        JoinClient {
            parent_channel,
            event_sender,
            event_receiver,
            ui,
            shared,
            _message_client: message_client,
            connection: None,
            last_ping: Instant::now(),
            latency: None,
            phase: None,
            socket,
            back_button,
        }
    }

    fn start_game(&mut self, ctx: &Context, users: Vec<User>, seed: u64) {
        info!("Game start!");
        self.phase = Some(MultiplayerPhase::new_game(
            ctx,
            self.shared.clone(),
            self.parent_channel.clone(),
            users,
            seed,
            Some(self.connection.as_ref().unwrap().1),
        ));
    }

    fn handle_event(&mut self, ctx: &mut Context, event: JoinEvent) -> GameResult<()> {
        trace!("event = {event:?}");
        match event {
            JoinEvent::NetworkEvent(network_event) => match network_event {
                NetworkEvent::Connect {
                    transport,
                    my_socket_addr,
                } => {
                    debug!("disconnected");
                    self.phase = Some(MultiplayerPhase::Lobby(LobbyClient::new(
                        Vec::new(),
                        Some(my_socket_addr.ip()),
                        self.event_sender.clone(),
                    )));
                    self.connection = Some((transport, my_socket_addr.ip()));
                }
                NetworkEvent::Message(server_message) => {
                    debug!("received {server_message:?}");
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
                                if game.get_current_player_type() == user {
                                    game.handle_message(ctx, message)?;
                                }
                            }
                        }
                    }
                }
                NetworkEvent::Disconnect => {
                    debug!("disconnected");
                    self.connection = None;
                    self.latency = None;
                    self.phase = None;
                }
            },
            JoinEvent::UIEvent(ui_event) => match ui_event {
                UIEvent::MainEvent(main_event) => self.parent_channel.send(main_event).unwrap(),
            },
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

    fn update(&mut self, ctx: &mut Context) -> Result<(), GameError> {
        self.ui.update(ctx)?;
        while let Ok(event) = self.event_receiver.try_recv() {
            self.handle_event(ctx, event)?;
        }

        {
            let mut back_button = self.back_button.borrow_mut();
            match self.phase {
                None => back_button.text = Text::new("Cancel"),
                Some(MultiplayerPhase::Lobby(_)) => back_button.text = Text::new("Leave"),
                _ => {}
            }
            back_button.state = UIElementState::invisible_if(matches!(
                self.phase,
                Some(MultiplayerPhase::Game { .. })
            ));
        }

        if let Some(phase) = &mut self.phase {
            phase.update(ctx)?;
            if let (MultiplayerPhase::Game { action_channel, .. }, Some((connection, _))) =
                (phase, &mut self.connection)
            {
                while let Ok(message) = action_channel.try_recv() {
                    debug!("sending {message:?}");
                    connection.blind_send(ClientMessage::Game(message));
                }
            }
        }

        if let Some((connection, _)) = &mut self.connection {
            let now = Instant::now();
            if now - self.last_ping > self.shared.args.ping_interval {
                connection.blind_send(ClientMessage::Ping);
                self.last_ping = now;
            }
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context, canvas: &mut Canvas) -> Result<(), GameError> {
        match &mut self.phase {
            None => {
                Text::new(format!("Connecting to {}...", self.socket))
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

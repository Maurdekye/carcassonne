use std::{
    cell::RefCell,
    collections::HashMap,
    net::IpAddr,
    rc::Rc,
    sync::mpsc::{channel, Receiver, Sender},
    time::Instant,
};

use ggez::{
    glam::vec2,
    graphics::{Canvas, Color, Rect, Text},
    Context, GameError, GameResult,
};

use crate::{
    game::player::PlayerType,
    main_client::MainEvent,
    multiplayer::transport::message::client::{self, ClientMessage},
    sub_event_handler::SubEventHandler,
    ui_manager::{Button, ButtonBounds, ButtonState, UIManager},
    util::{AnchorPoint, ContextExt, TextExt},
    Args,
};

use super::{
    lobby_client::{LobbyClient, LobbyEvent},
    transport::{
        message::server::{self, ClientInfo, LobbyState, ServerMessage, User},
        MessageServer, NetworkEvent, ServerNetworkEvent, ServersideTransport,
    },
    MultiplayerPhase,
};

#[derive(Clone)]
enum UIEvent {
    MainEvent(MainEvent),
    StartGame,
}

#[allow(clippy::enum_variant_names)]
enum HostEvent {
    UIEvent(UIEvent),
    NetworkEvent {
        src_addr: IpAddr,
        event: ServerNetworkEvent,
    },
    LobbyEvent(LobbyEvent),
}

impl From<(IpAddr, ServerNetworkEvent)> for HostEvent {
    fn from((src_addr, event): (IpAddr, ServerNetworkEvent)) -> Self {
        HostEvent::NetworkEvent { src_addr, event }
    }
}

impl From<UIEvent> for HostEvent {
    fn from(value: UIEvent) -> Self {
        HostEvent::UIEvent(value)
    }
}

impl From<LobbyEvent> for HostEvent {
    fn from(value: LobbyEvent) -> Self {
        HostEvent::LobbyEvent(value)
    }
}

struct HostClientInfo {
    transport: ServersideTransport,
    last_ping: Instant,
}

struct HostUser {
    user: User,
    client_info: Option<HostClientInfo>,
}

#[derive(PartialEq, Eq, std::hash::Hash)]
pub enum IpOrHost {
    Host,
    Ip(IpAddr),
}

pub struct HostClient {
    args: Args,
    parent_channel: Sender<MainEvent>,
    _event_sender: Sender<HostEvent>,
    event_receiver: Receiver<HostEvent>,
    ui: UIManager<UIEvent, HostEvent>,
    _message_server: MessageServer,
    users: HashMap<IpOrHost, HostUser>,
    phase: MultiplayerPhase<HostEvent>,
    start_game_button: Rc<RefCell<Button<UIEvent>>>,
}

impl HostClient {
    pub fn new(parent_channel: Sender<MainEvent>, args: Args) -> HostClient {
        let (event_sender, event_receiver) = channel();
        let ui_sender = event_sender.clone();
        let (ui, [_, start_game_button]) = UIManager::new_and_rc_buttons(
            ui_sender,
            [
                Button::new(
                    ButtonBounds::absolute(Rect::new(30.0, 30.0, 120.0, 40.0)),
                    Text::new("Back"),
                    UIEvent::MainEvent(MainEvent::ReturnToMainMenu),
                ),
                Button::new(
                    ButtonBounds {
                        relative: Rect::new(1.0, 1.0, 0.0, 0.0),
                        absolute: Rect::new(-260.0, -60.0, 240.0, 40.0),
                    },
                    Text::new("Start Game").size(32.0),
                    UIEvent::StartGame,
                ),
            ],
        );
        start_game_button.borrow_mut().state = ButtonState::Disabled;
        let message_server = MessageServer::start(event_sender.clone(), args.port);
        let mut this = HostClient {
            args,
            parent_channel,
            ui,
            _message_server: message_server,
            users: HashMap::from([(
                IpOrHost::Host,
                HostUser {
                    client_info: None,
                    user: User {
                        client_info: None,
                        color: None,
                    },
                },
            )]),
            phase: MultiplayerPhase::Lobby(LobbyClient::new(
                Vec::new(),
                None,
                event_sender.clone(),
            )),
            _event_sender: event_sender,
            event_receiver,
            start_game_button,
        };
        this.update_lobby_clients();
        this
    }

    fn update_lobby_clients(&mut self) {
        let users: Vec<_> = self.users.values().map(|user| user.user.clone()).collect();
        match &mut self.phase {
            MultiplayerPhase::Lobby(lobby) => {
                self.start_game_button.borrow_mut().state = ButtonState::disabled_if(
                    users.len() < 2 || users.iter().any(|user| user.color.is_none()),
                );
                let message = server::LobbyMessage::LobbyState(LobbyState { users });
                for client in self.users.values_mut() {
                    if let Some(host_client_info) = &mut client.client_info {
                        host_client_info
                            .transport
                            .blind_send(ServerMessage::Lobby(message.clone()));
                    }
                }
                let _ = lobby.handle_message(message);
            }
            MultiplayerPhase::Game { game, .. } => {
                let _ = game.update_pings(users);
            }
        }
    }

    fn add_client(&mut self, transport: ServersideTransport, client_info: ClientInfo) {
        self.users.insert(
            IpOrHost::Ip(client_info.ip),
            HostUser {
                client_info: Some(HostClientInfo {
                    transport,
                    last_ping: Instant::now(),
                }),
                user: User {
                    color: None,
                    client_info: Some(client_info),
                },
            },
        );
        self.update_lobby_clients();
    }

    fn handle_event(&mut self, ctx: &mut Context, event: HostEvent) -> GameResult<()> {
        match event {
            HostEvent::NetworkEvent { src_addr, event } => match event {
                NetworkEvent::Connect { transport, .. } => {
                    println!("[{src_addr:?}] connected");
                    self.add_client(
                        transport,
                        ClientInfo {
                            ip: src_addr,
                            latency: None,
                        },
                    );
                }
                NetworkEvent::Message(client_message) => {
                    println!("[{src_addr:?}] {client_message:?}");
                    let host_client = self.users.get_mut(&IpOrHost::Ip(src_addr)).unwrap();
                    let host_client_info = host_client.client_info.as_mut().unwrap();
                    match client_message {
                        ClientMessage::Ping => {
                            host_client_info.transport.blind_send(ServerMessage::Pong);
                        }
                        ClientMessage::Pong => {
                            let last_ping = host_client_info.last_ping;
                            let client_info = host_client.user.client_info.as_mut().unwrap();
                            client_info.latency = Some(Instant::now() - last_ping);
                            self.update_lobby_clients();
                        }
                        ClientMessage::Lobby(client::LobbyMessage::ChooseColor(color)) => {
                            if let MultiplayerPhase::Lobby(_) = self.phase {
                                host_client.user.color = color;
                                self.update_lobby_clients();
                            }
                        }
                        ClientMessage::Game(message) => {
                            if let MultiplayerPhase::Game { game, .. } = &mut self.phase {
                                let source_player = PlayerType::from(Some(src_addr));
                                if game.get_current_player_type() == source_player {
                                    for client_info in self
                                        .users
                                        .values_mut()
                                        .filter(|user| {
                                            Some(src_addr)
                                                != user.user.client_info.as_ref().map(|i| i.ip)
                                        })
                                        .filter_map(|user| user.client_info.as_mut())
                                    {
                                        client_info.transport.blind_send(ServerMessage::Game {
                                            message: message.clone(),
                                            user: source_player,
                                        })
                                    }
                                    game.handle_message(ctx, message)?;
                                }
                            }
                        }
                    }
                }
                NetworkEvent::Disconnect => {
                    println!("[{src_addr:?}] disconnected");
                    self.users.remove(&IpOrHost::Ip(src_addr));
                    self.update_lobby_clients();
                }
            },
            HostEvent::UIEvent(uievent) => match uievent {
                UIEvent::MainEvent(main_event) => self.parent_channel.send(main_event).unwrap(),
                UIEvent::StartGame => {
                    if self.users.values().all(|user| user.user.color.is_some()) {
                        self.start_game(ctx);
                    }
                }
            },
            HostEvent::LobbyEvent(LobbyEvent::ChooseColor(color)) => {
                let me = self
                    .users
                    .values_mut()
                    .find(|user| user.client_info.is_none())
                    .unwrap();
                me.user.color = color;
                self.update_lobby_clients();
            }
        }
        Ok(())
    }

    fn start_game(&mut self, ctx: &Context) {
        println!("Game Start!");
        let game_seed = rand::random();
        for client in self
            .users
            .values_mut()
            .filter_map(|user| user.client_info.as_mut())
        {
            client
                .transport
                .blind_send(ServerMessage::StartGame { game_seed });
        }
        self.phase = MultiplayerPhase::new_game(
            ctx,
            self.args.clone(),
            self.parent_channel.clone(),
            self.users.values().map(|user| user.user.clone()).collect(),
            game_seed,
            None,
        );
    }
}

impl SubEventHandler<GameError> for HostClient {
    fn mouse_wheel_event(&mut self, ctx: &mut Context, x: f32, y: f32) -> Result<(), GameError> {
        self.phase.mouse_wheel_event(ctx, x, y)
    }

    fn update(&mut self, ctx: &mut Context) -> Result<(), GameError> {
        if let MultiplayerPhase::Lobby(_) = &self.phase {
            self.ui.update(ctx)?;
        }

        while let Ok(event) = self.event_receiver.try_recv() {
            self.handle_event(ctx, event)?;
        }

        self.phase.update(ctx)?;
        if let MultiplayerPhase::Game { action_channel, .. } = &mut self.phase {
            for message in action_channel.try_iter() {
                for client_info in self
                    .users
                    .values_mut()
                    .filter_map(|user| user.client_info.as_mut())
                {
                    client_info.transport.blind_send(ServerMessage::Game {
                        message: message.clone(),
                        user: PlayerType::MultiplayerHost,
                    });
                }
            }
        }

        let now = Instant::now();
        let mut updated_ping = false;
        for client in self
            .users
            .values_mut()
            .filter_map(|user| user.client_info.as_mut())
        {
            if now - client.last_ping > self.args.ping_interval {
                client.last_ping = now;
                client.transport.blind_send(ServerMessage::Ping);
                updated_ping = true;
            }
        }
        if updated_ping {
            self.update_lobby_clients();
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context, canvas: &mut Canvas) -> Result<(), GameError> {
        if let MultiplayerPhase::Lobby(_) = self.phase {
            Text::new(format!("Hosting on port {}", self.args.port))
                .size(36.0)
                .anchored_by(
                    ctx,
                    ctx.res() * vec2(0.5, 0.0) + vec2(0.0, 30.0),
                    AnchorPoint::NorthCenter,
                )?
                .color(Color::BLACK)
                .draw(canvas);
        }

        self.phase.draw(ctx, canvas)?;

        if let MultiplayerPhase::Lobby(_) = &self.phase {
            self.ui.draw(ctx, canvas)?;
        }

        Ok(())
    }
}

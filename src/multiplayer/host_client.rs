use std::{
    cell::RefCell,
    collections::HashMap,
    error::Error,
    fs::File,
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
use log::{debug, info, trace};

use crate::{
    game::player::PlayerType,
    game_client::{GameAction, GameState},
    game_expansions_selector::GameExpansionsSelector,
    main_client::MainEvent,
    multiplayer::{
        lobby_client::{LobbyClient, LobbyEvent},
        message::{
            client::{self, ClientMessage},
            server::{self, ClientInfo, LobbyState, ServerMessage, User},
        },
        MultiplayerPhase,
    },
    util::{AnchorPoint, ContextExt, TextExt},
    Shared,
};

use ggez_no_re::{
    sub_event_handler::SubEventHandler,
    transport::{MessageServer, NetworkEvent, ServerNetworkEvent, ServersideTransport},
    ui_manager::{Bounds, button::Button, UIElement, UIElementState, UIManager},
};

use super::message::Message;

#[derive(Clone, Debug)]
enum UIEvent {
    MainEvent(MainEvent),
    StartGame,
}

#[allow(clippy::enum_variant_names)]
#[derive(Debug)]
enum HostEvent {
    UIEvent(UIEvent),
    NetworkEvent {
        src_addr: IpAddr,
        event: ServerNetworkEvent<Message>,
    },
    LobbyEvent(LobbyEvent),
}

impl From<(IpAddr, ServerNetworkEvent<Message>)> for HostEvent {
    fn from((src_addr, event): (IpAddr, ServerNetworkEvent<Message>)) -> Self {
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
    shared: Shared,
    parent_channel: Sender<MainEvent>,
    event_sender: Sender<HostEvent>,
    event_receiver: Receiver<HostEvent>,
    ui: UIManager<UIEvent, HostEvent>,
    expansions_selector: GameExpansionsSelector,
    _message_server: MessageServer,
    users: HashMap<IpOrHost, HostUser>,
    phase: MultiplayerPhase<HostEvent>,
    start_game_button: Rc<RefCell<Button<UIEvent>>>,
    port: u16,
    username: String,
}

impl HostClient {
    pub fn new(
        parent_channel: Sender<MainEvent>,
        shared: Shared,
        username: String,
        port: u16,
    ) -> HostClient {
        let (event_sender, event_receiver) = channel();
        let ui_sender = event_sender.clone();
        let (ui, [_, UIElement::Button(start_game_button)]) = UIManager::new_and_rc_elements(
            ui_sender,
            [
                UIElement::Button(Button::new(
                    Bounds::absolute(Rect::new(30.0, 30.0, 120.0, 40.0)),
                    Text::new("Back"),
                    UIEvent::MainEvent(MainEvent::MultiplayerMenu),
                )),
                UIElement::Button(Button::new(
                    Bounds {
                        relative: Rect::new(1.0, 1.0, 0.0, 0.0),
                        absolute: Rect::new(-260.0, -60.0, 240.0, 40.0),
                    },
                    Text::new("Start Game").size(32.0),
                    UIEvent::StartGame,
                )),
            ],
        ) else {
            panic!()
        };
        let expansions_selector = GameExpansionsSelector::new(Bounds {
            relative: Rect::new(0.6, 0.3, 0.0, 0.0),
            absolute: Rect::new(0.0, 100.0, 0.0, 0.0),
        });
        start_game_button.borrow_mut().state = UIElementState::Disabled;
        let message_server = MessageServer::start::<Message>(event_sender.clone(), port);
        let mut this = HostClient {
            parent_channel,
            ui,
            expansions_selector,
            _message_server: message_server,
            users: HashMap::from([(
                IpOrHost::Host,
                HostUser {
                    client_info: None,
                    user: User {
                        client_info: None,
                        color: None,
                        username: username.clone(),
                    },
                },
            )]),
            phase: MultiplayerPhase::Lobby(LobbyClient::new(
                Vec::new(),
                None,
                shared.clone(),
                event_sender.clone(),
            )),
            shared,
            event_sender,
            event_receiver,
            start_game_button,
            port,
            username,
        };
        this.update_lobby_clients();
        this
    }

    fn broadcast(&mut self, message: ServerMessage) {
        self.broadcast_filter(message, |_| true);
    }

    fn broadcast_filter(&mut self, message: ServerMessage, filter: impl Fn(IpAddr) -> bool) {
        trace!("message = {message:?}");
        for client_info in self
            .users
            .values_mut()
            .filter(|user| {
                user.user
                    .client_info
                    .as_ref()
                    .is_some_and(|i| (filter)(i.ip))
            })
            .filter_map(|user| user.client_info.as_mut())
        {
            client_info.transport.blind_send::<Message>(message.clone())
        }
    }

    fn update_lobby_clients(&mut self) {
        let users: Vec<_> = self.users.values().map(|user| user.user.clone()).collect();
        let message = server::LobbyMessage::LobbyState(LobbyState {
            users: users.clone(),
        });
        match &mut self.phase {
            MultiplayerPhase::Lobby(lobby) => {
                self.start_game_button.borrow_mut().state = UIElementState::disabled_if(
                    users.len() < 2 || users.iter().any(|user| user.color.is_none()),
                );
                let _ = lobby.handle_message(message.clone());
            }
            MultiplayerPhase::Game { game, .. } => {
                let _ = game.update_pings(users);
            }
        }
        self.broadcast(ServerMessage::Lobby(message.clone()));
    }

    fn add_client(&mut self, mut transport: ServersideTransport, client_info: ClientInfo) {
        if let MultiplayerPhase::Game { game, .. } = &self.phase {
            transport.blind_send::<Message>(ServerMessage::GameState(game.state.clone().into()));
        }
        let username = client_info.ip.to_string();
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
                    username,
                },
            },
        );
        self.update_lobby_clients();
    }

    fn handle_event(&mut self, ctx: &mut Context, event: HostEvent) -> GameResult<()> {
        trace!("event = {event:?}");
        match event {
            HostEvent::NetworkEvent { src_addr, event } => match event {
                NetworkEvent::Connect { transport, .. } => {
                    debug!("[{src_addr:?}] connected");
                    self.add_client(
                        transport,
                        ClientInfo {
                            ip: src_addr,
                            latency: None,
                        },
                    );
                }
                NetworkEvent::Message(client_message) => {
                    debug!("[{src_addr:?}] message: {client_message:?}");
                    let host_client = self.users.get_mut(&IpOrHost::Ip(src_addr)).unwrap();
                    let host_client_info = host_client.client_info.as_mut().unwrap();
                    match client_message {
                        ClientMessage::Ping => {
                            host_client_info
                                .transport
                                .blind_send::<Message>(ServerMessage::Pong);
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
                                if game
                                    .get_current_player_type()
                                    .matches_address(Some(src_addr))
                                {
                                    let username = host_client.user.username.clone();
                                    game.handle_message(ctx, message.clone())?;
                                    self.broadcast_filter(
                                        ServerMessage::Game {
                                            message,
                                            user: PlayerType::from_details(
                                                username,
                                                Some(src_addr),
                                            ),
                                        },
                                        |ip| ip != src_addr,
                                    );
                                }
                            }
                        }
                        ClientMessage::Username(username) => {
                            host_client.user.username = username;
                            self.update_lobby_clients();
                        }
                    }
                }
                NetworkEvent::Disconnect => {
                    debug!("[{src_addr:?}] disconnected");
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
        info!("Game Start!");
        let result: Result<Option<GameState>, Box<dyn Error>> = try {
            if let Some(path) = &self.shared.args.multiplayer_load {
                let file = File::open(path)?;
                Some(bincode::deserialize_from(file)?)
            } else {
                None
            }
        };
        match result {
            Ok(Some(state)) => {
                self.broadcast(ServerMessage::GameState(state.clone().into()));
                self.phase = MultiplayerPhase::new_from_state(
                    ctx,
                    self.shared.clone(),
                    self.parent_channel.clone(),
                    state,
                    None,
                    self.username.clone(),
                )
            }
            result => {
                if let Err(err) = result {
                    log::error!("Error loading multiplayer game: {err}");
                }
                let game_seed = rand::random();
                let expansions = self.expansions_selector.get_selected_expansions();
                {
                    let expansions = expansions.clone();
                    self.broadcast(ServerMessage::StartGame {
                        game_seed,
                        expansions,
                    });
                }
                self.phase = MultiplayerPhase::new_game(
                    ctx,
                    self.shared.clone(),
                    self.parent_channel.clone(),
                    self.users.values().map(|user| user.user.clone()).collect(),
                    game_seed,
                    None,
                    self.username.clone(),
                    expansions,
                );
            }
        }
    }

    fn ping_clients(&mut self) {
        let now = Instant::now();
        let mut updated_ping = false;
        for client in self
            .users
            .values_mut()
            .filter_map(|user| user.client_info.as_mut())
        {
            if now - client.last_ping > self.shared.args.ping_interval {
                client.last_ping = now;
                client.transport.blind_send::<Message>(ServerMessage::Ping);
                updated_ping = true;
            }
        }
        if updated_ping {
            self.update_lobby_clients();
        }
    }
}

impl SubEventHandler for HostClient {
    fn mouse_wheel_event(&mut self, ctx: &mut Context, x: f32, y: f32) -> Result<(), GameError> {
        self.phase.mouse_wheel_event(ctx, x, y)
    }

    fn update(&mut self, ctx: &mut Context) -> Result<(), GameError> {
        if let MultiplayerPhase::Lobby(_) = &self.phase {
            self.ui.update(ctx)?;
            self.expansions_selector.update(ctx)?;
        }

        while let Ok(event) = self.event_receiver.try_recv() {
            self.handle_event(ctx, event)?;
        }

        self.phase.update(ctx)?;
        let actions = if let MultiplayerPhase::Game { action_channel, .. } = &mut self.phase {
            action_channel.try_iter().collect()
        } else {
            Vec::new()
        };
        for action in actions {
            match action {
                GameAction::Message(message) => {
                    self.broadcast(ServerMessage::Game {
                        message,
                        user: PlayerType::MultiplayerHost {
                            username: self.username.clone(),
                        },
                    });
                }
                GameAction::ReturnToLobby => {
                    self.phase = MultiplayerPhase::Lobby(LobbyClient::new(
                        Vec::new(),
                        None,
                        self.shared.clone(),
                        self.event_sender.clone(),
                    ));
                    self.update_lobby_clients();
                    break;
                }
            }
        }

        self.ping_clients();

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context, canvas: &mut Canvas) -> Result<(), GameError> {
        if let MultiplayerPhase::Lobby(_) = self.phase {
            Text::new(format!("Hosting on port {}", self.port))
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
            self.expansions_selector.draw(ctx, canvas)?;
        }

        Ok(())
    }
}

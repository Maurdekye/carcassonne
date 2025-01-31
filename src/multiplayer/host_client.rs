use std::{
    cell::LazyCell,
    collections::HashMap,
    net::IpAddr,
    sync::mpsc::{channel, Receiver, Sender},
    time::Instant,
};

use ggez::{
    glam::{vec2, Vec2},
    graphics::{Canvas, Color, Rect, Text},
    Context, GameError, GameResult,
};

use crate::{
    game_client::GameClient,
    main_client::MainEvent,
    multiplayer::transport::{message::Message, MessageTransporter},
    sub_event_handler::SubEventHandler,
    ui_manager::{Button, ButtonBounds, UIManager},
    util::{AnchorPoint, ContextExt, TextExt},
    Args,
};

use super::{
    lobby_client::LobbyClient,
    transport::{
        message::{ClientInfo, LobbyMessage, LobbyState, User},
        MessageServer, NetworkEvent,
    },
    MultiplayerPhase, PING_FREQUENCY,
};

#[derive(Clone)]
enum UIEvent {
    MainEvent(MainEvent),
}

enum HostEvent {
    UIEvent(UIEvent),
    NetworkEvent {
        src_addr: IpAddr,
        event: NetworkEvent,
    },
}

impl From<(IpAddr, NetworkEvent)> for HostEvent {
    fn from((src_addr, event): (IpAddr, NetworkEvent)) -> Self {
        HostEvent::NetworkEvent { src_addr, event }
    }
}

impl From<UIEvent> for HostEvent {
    fn from(value: UIEvent) -> Self {
        HostEvent::UIEvent(value)
    }
}

struct HostClientInfo {
    transport: MessageTransporter,
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
    phase: MultiplayerPhase,
}

impl HostClient {
    pub fn new(parent_channel: Sender<MainEvent>, args: Args) -> HostClient {
        let (event_sender, event_receiver) = channel();
        let ui_sender = event_sender.clone();
        let (ui, [..]) = UIManager::new_and_rc_buttons(
            ui_sender,
            [Button::new(
                ButtonBounds::absolute(Rect::new(30.0, 30.0, 120.0, 40.0)),
                Text::new("Back"),
                UIEvent::MainEvent(MainEvent::ReturnToMainMenu),
            )],
        );
        let message_server = MessageServer::start(event_sender.clone(), args.port);
        let mut this = HostClient {
            args,
            parent_channel,
            _event_sender: event_sender,
            event_receiver,
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
            phase: MultiplayerPhase::Lobby(LobbyClient::new(Vec::new())),
        };
        this.update_lobby_clients();
        this
    }

    fn update_lobby_clients(&mut self) {
        if let MultiplayerPhase::Lobby(lobby) = &mut self.phase {
            lobby.users = self.users.values().map(|user| user.user.clone()).collect();
            for client in self.users.values_mut() {
                if let Some(host_client_info) = &mut client.client_info {
                    host_client_info.transport.blind_send(&Message::Lobby(
                        LobbyMessage::LobbyState(LobbyState {
                            users: lobby.users.clone(),
                        }),
                    ));
                }
            }
        }
    }

    fn add_client(&mut self, transport: MessageTransporter, client_info: ClientInfo) {
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

    fn handle_event(&mut self, _ctx: &mut Context, event: HostEvent) -> GameResult<()> {
        match event {
            HostEvent::NetworkEvent { src_addr, event } => match event {
                NetworkEvent::Connect(transport) => {
                    self.add_client(
                        transport,
                        ClientInfo {
                            ip: src_addr,
                            latency: None,
                        },
                    );
                }
                NetworkEvent::Message(client_message) => {
                    dbg!(&client_message);
                    let host_client = self.users.get_mut(&IpOrHost::Ip(src_addr)).unwrap();
                    let host_client_info = host_client.client_info.as_mut().unwrap();
                    match client_message {
                        Message::Ping => {
                            let _ = host_client_info.transport.send(&Message::Pong);
                        }
                        Message::Pong => {
                            let last_ping = host_client_info.last_ping;
                            let client_info = host_client.user.client_info.as_mut().unwrap();
                            client_info.latency = Some(Instant::now() - last_ping);
                        }
                        _ => {}
                    }
                }
                NetworkEvent::Disconnect => {
                    self.users.remove(&IpOrHost::Ip(src_addr));
                    self.update_lobby_clients();
                }
            },
            HostEvent::UIEvent(uievent) => match uievent {
                UIEvent::MainEvent(main_event) => self.parent_channel.send(main_event).unwrap(),
            },
        }
        Ok(())
    }
}

impl SubEventHandler<GameError> for HostClient {
    fn update(&mut self, ctx: &mut Context) -> Result<(), GameError> {
        self.ui.update(ctx)?;
        while let Ok(event) = self.event_receiver.try_recv() {
            self.handle_event(ctx, event)?;
        }

        self.phase.update(ctx)?;

        let now = Instant::now();
        let mut updated_ping = false;
        for client in self
            .users
            .values_mut()
            .filter_map(|user| user.client_info.as_mut())
        {
            if now - client.last_ping > PING_FREQUENCY {
                client.last_ping = now;
                client.transport.blind_send(&Message::Ping);
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

        self.ui.draw(ctx, canvas)
    }
}

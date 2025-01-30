use std::{
    cell::LazyCell,
    collections::HashMap,
    net::IpAddr,
    sync::mpsc::{channel, Receiver, Sender},
};

use ggez::{
    glam::{vec2, Vec2},
    graphics::{Canvas, Text},
    Context, GameError, GameResult,
};

use crate::{
    main_client::MainEvent,
    multiplayer::transport::{Message, MessageTransporter},
    sub_event_handler::SubEventHandler,
    ui_manager::UIManager,
    util::{AnchorPoint, TextExt},
    Args,
};

use super::transport::{MessageServer, NetworkEvent};

#[derive(Clone)]
enum UIEvent {}

enum MultiplayerHostMenuEvent {
    MainEvent(MainEvent),
    UIEvent(UIEvent),
    NetworkEvent {
        src_addr: IpAddr,
        event: NetworkEvent,
    },
}

impl From<(IpAddr, NetworkEvent)> for MultiplayerHostMenuEvent {
    fn from((src_addr, event): (IpAddr, NetworkEvent)) -> Self {
        MultiplayerHostMenuEvent::NetworkEvent { src_addr, event }
    }
}

impl From<UIEvent> for MultiplayerHostMenuEvent {
    fn from(value: UIEvent) -> Self {
        MultiplayerHostMenuEvent::UIEvent(value)
    }
}

pub struct MultiplayerHostMenuClient {
    args: Args,
    parent_channel: Sender<MainEvent>,
    event_sender: Sender<MultiplayerHostMenuEvent>,
    event_receiver: Receiver<MultiplayerHostMenuEvent>,
    ui: UIManager<UIEvent, MultiplayerHostMenuEvent>,
    message_server: MessageServer,
    clients: HashMap<IpAddr, MessageTransporter>,
}

impl MultiplayerHostMenuClient {
    pub fn new(parent_channel: Sender<MainEvent>, args: Args) -> MultiplayerHostMenuClient {
        let (event_sender, event_receiver) = channel();
        let ui_sender = event_sender.clone();
        let (ui, [..]) = UIManager::new_and_rc_buttons(ui_sender, []);
        let message_server = MessageServer::start(event_sender.clone(), args.port);
        MultiplayerHostMenuClient {
            args,
            parent_channel,
            event_sender,
            event_receiver,
            ui,
            message_server,
            clients: HashMap::new(),
        }
    }

    fn handle_event(
        &mut self,
        _ctx: &mut Context,
        event: MultiplayerHostMenuEvent,
    ) -> GameResult<()> {
        match event {
            MultiplayerHostMenuEvent::MainEvent(main_event) => {
                self.parent_channel.send(main_event).unwrap()
            }
            MultiplayerHostMenuEvent::NetworkEvent { src_addr, event } => match event {
                NetworkEvent::Connect(transport) => {
                    self.clients.insert(src_addr, transport);
                }
                NetworkEvent::Message(client_message) => {
                    dbg!(&client_message);
                    let mut client = LazyCell::new(|| self.clients.get_mut(&src_addr).unwrap());
                    match client_message {
                        Message::Ping => LazyCell::force_mut(&mut client)
                            .send(&Message::Pong)
                            .unwrap(),
                        _ => {}
                    }
                }
                NetworkEvent::Disconnect => {
                    self.clients.remove(&src_addr);
                }
            },
            MultiplayerHostMenuEvent::UIEvent(uievent) => match uievent {},
        }
        Ok(())
    }
}

impl SubEventHandler<GameError> for MultiplayerHostMenuClient {
    fn update(&mut self, ctx: &mut Context) -> Result<(), GameError> {
        self.ui.update(ctx)?;
        while let Ok(event) = self.event_receiver.try_recv() {
            self.handle_event(ctx, event)?;
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context, canvas: &mut Canvas) -> Result<(), GameError> {
        let res: Vec2 = ctx.gfx.drawable_size().into();

        Text::new(format!(
            "Waiting for player connections on port {}...",
            self.args.port
        ))
        .size(64.0)
        .anchored_by(ctx, res * vec2(0.5, 0.1), AnchorPoint::NorthCenter)?
        .draw(canvas);

        Text::new(if self.clients.is_empty() {
            "No clients connected".to_string()
        } else {
            format!(
                "{} connected clients:\n{}",
                self.clients.len(),
                self.clients
                    .keys()
                    .map(|client| format!("{client}"))
                    .collect::<Vec<_>>()
                    .join("\n")
            )
        })
        .size(32.0)
        .anchored_by(ctx, vec2(50.0, 100.0), AnchorPoint::NorthWest)?
        .draw(canvas);

        self.ui.draw(ctx, canvas)
    }
}

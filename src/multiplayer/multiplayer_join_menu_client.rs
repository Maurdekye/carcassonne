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
    main_client::MainEvent,
    multiplayer::transport::Message,
    sub_event_handler::SubEventHandler,
    ui_manager::UIManager,
    util::{AnchorPoint, TextExt},
    Args,
};

use super::transport::{MessageClient, MessageTransporter, NetworkEvent};

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
    event_sender: Sender<MultiplayerJoinMenuEvent>,
    event_receiver: Receiver<MultiplayerJoinMenuEvent>,
    args: Args,
    ui: UIManager<UIEvent, MultiplayerJoinMenuEvent>,
    message_client: MessageClient,
    connection: Option<MessageTransporter>,
    last_ping: Instant,
    latency: Option<Duration>,
}

impl MultiplayerJoinMenuClient {
    pub fn new(parent_channel: Sender<MainEvent>, args: Args, socket: SocketAddr) -> Self {
        let (event_sender, event_receiver) = channel();
        let ui = UIManager::new(event_sender.clone(), []);
        let message_client = MessageClient::start(event_sender.clone(), socket);
        MultiplayerJoinMenuClient {
            parent_channel,
            event_sender,
            event_receiver,
            ui,
            args,
            message_client,
            connection: None,
            last_ping: Instant::now(),
            latency: None,
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
                }
                NetworkEvent::Message(server_message) => {
                    dbg!(&server_message);
                    let mut server = LazyCell::new(|| self.connection.as_mut().unwrap());
                    match server_message {
                        Message::Pong => {
                            let now = Instant::now();
                            self.latency = Some(now - self.last_ping);
                            dbg!(&self.latency);
                        }
                        Message::Ping => {
                            LazyCell::force_mut(&mut server)
                                .send(&Message::Pong)
                                .unwrap();
                        }
                        _ => {}
                    }
                }
                NetworkEvent::Disconnect => {
                    println!("Disconnected.");
                    self.connection = None
                }
            },
            MultiplayerJoinMenuEvent::UIEvent(uievent) => {}
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

        if let Some(connection) = &mut self.connection {
            let now = Instant::now();
            if now > self.last_ping + Duration::from_secs(2) {
                connection.send(&Message::Ping).unwrap();
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
        let res: Vec2 = ctx.gfx.drawable_size().into();

        Text::new(if self.connection.is_none() {
            format!(
                "Connecting to {}:{}...",
                self.args.ip.unwrap(),
                self.args.port
            )
        } else {
            "Connected!".to_string()
        })
        .size(86.0)
        .centered_on(ctx, res * vec2(0.5, 0.2))?
        .color(Color::BLACK)
        .draw(canvas);

        if let Some(latency) = self.latency {
            Text::new(format!("{} ms", latency.as_millis()))
                .size(32.0)
                .anchored_by(ctx, vec2(50.0, 50.0), AnchorPoint::NorthWest)?
                .color(Color::BLACK)
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

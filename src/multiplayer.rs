pub mod multiplayer_host_menu_client {
    use std::{
        collections::HashMap,
        net::IpAddr,
        sync::mpsc::{channel, Receiver, Sender},
    };

    use ggez::{graphics::Canvas, Context, GameError, GameResult};

    use crate::{
        main_client::MainEvent, multiplayer::transport::MessageTransporter,
        sub_event_handler::SubEventHandler, ui_manager::UIManager, Args,
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
        parent_channel: Sender<MainEvent>,
        event_sender: Sender<MultiplayerHostMenuEvent>,
        event_receiver: Receiver<MultiplayerHostMenuEvent>,
        ui: UIManager<UIEvent, MultiplayerHostMenuEvent>,
        message_server: MessageServer<MultiplayerHostMenuEvent>,
        clients: HashMap<IpAddr, MessageTransporter>,
    }

    impl MultiplayerHostMenuClient {
        pub fn new(parent_channel: Sender<MainEvent>, args: Args) -> MultiplayerHostMenuClient {
            let (event_sender, event_receiver) = channel();
            let ui_sender = event_sender.clone();
            let (ui, [..]) = UIManager::new_and_rc_buttons(ui_sender, []);
            let message_server = MessageServer::start(event_sender.clone(), args.port);
            MultiplayerHostMenuClient {
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
                        dbg!(client_message);
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
            self.ui.update(ctx);
            while let Ok(event) = self.event_receiver.try_recv() {
                self.handle_event(ctx, event)?;
            }
            Ok(())
        }

        fn draw(&mut self, ctx: &mut Context, canvas: &mut Canvas) -> Result<(), GameError> {
            self.ui.draw(ctx, canvas)
        }
    }
}

pub mod multiplayer_join_screen {
    use std::{
        net::SocketAddr,
        sync::mpsc::{channel, Receiver, Sender},
    };

    use ggez::{Context, GameError, GameResult};

    use crate::{
        main_client::MainEvent, sub_event_handler::SubEventHandler, ui_manager::UIManager, Args,
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
        ui: UIManager<UIEvent, MultiplayerJoinMenuEvent>,
        message_client: MessageClient<MultiplayerJoinMenuEvent>,
        connection: Option<MessageTransporter>,
    }

    impl MultiplayerJoinMenuClient {
        pub fn new(parent_channel: Sender<MainEvent>, _args: Args, socket: SocketAddr) -> Self {
            let (event_sender, event_receiver) = channel();
            let ui = UIManager::new(event_sender.clone(), []);
            let message_client = MessageClient::start(event_sender.clone(), socket);
            MultiplayerJoinMenuClient {
                parent_channel,
                event_sender,
                event_receiver,
                ui,
                message_client,
                connection: None,
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
                    NetworkEvent::Connect(transport) => self.connection = Some(transport),
                    NetworkEvent::Message(server_message) => {
                        dbg!(server_message);
                    }
                    NetworkEvent::Disconnect => self.connection = None,
                },
                MultiplayerJoinMenuEvent::UIEvent(uievent) => {}
            }
            Ok(())
        }
    }

    impl SubEventHandler<GameError> for MultiplayerJoinMenuClient {
        fn update(&mut self, ctx: &mut ggez::Context) -> Result<(), GameError> {
            self.ui.update(ctx);
            while let Ok(event) = self.event_receiver.try_recv() {
                self.handle_event(ctx, event)?;
            }
            Ok(())
        }

        fn draw(
            &mut self,
            ctx: &mut ggez::Context,
            canvas: &mut ggez::graphics::Canvas,
        ) -> Result<(), GameError> {
            self.ui.draw(ctx, canvas)
        }
    }
}

pub mod transport;

use std::{
    cell::RefCell,
    net::{SocketAddr, ToSocketAddrs},
    rc::Rc,
    sync::mpsc::{channel, Receiver, Sender},
    time::{Duration, Instant},
};

use ggez::{
    glam::vec2,
    graphics::{Canvas, Color, Rect, Text},
    Context, GameError, GameResult,
};
use log::trace;

use crate::{
    main_client::MainEvent,
    util::{AnchorPoint, ContextExt, RectExt, TextExt},
    Shared,
};
use ggez_no_re::{
    sub_event_handler::SubEventHandler,
    ui_manager::{
        button::Button, text_input::TextInput, Bounds, UIElement, UIElementRenderable, UIManager,
    },
};

const ERROR_DISPLAY_PERIOD: Duration = Duration::from_secs(10);

#[derive(Clone, Debug)]
enum MultiplayerMenuEvent {
    MainEvent(MainEvent),
    JoinLobby,
    HostLobby,
}

pub struct MultiplayerMenuClient {
    parent_channel: Sender<MainEvent>,
    shared: Shared,
    _event_sender: Sender<MultiplayerMenuEvent>,
    event_receiver: Receiver<MultiplayerMenuEvent>,
    ui: UIManager<MultiplayerMenuEvent, MultiplayerMenuEvent>,
    username_input: Rc<RefCell<TextInput>>,
    destination_input: Rc<RefCell<TextInput>>,
    port_input: Rc<RefCell<TextInput>>,
    join_button: Rc<RefCell<Button<MultiplayerMenuEvent>>>,
    host_button: Rc<RefCell<Button<MultiplayerMenuEvent>>>,
    error_message: Option<(String, Instant)>,
}

impl MultiplayerMenuClient {
    pub fn new(parent_channel: Sender<MainEvent>, shared: Shared) -> MultiplayerMenuClient {
        let relative = Rect::new(0.5, 0.5, 0.0, 0.0);
        let (event_sender, event_receiver) = channel();
        let (
            ui,
            [UIElement::TextInput(username_input), UIElement::TextInput(destination_input), UIElement::TextInput(port_input), _, UIElement::Button(join_button), UIElement::Button(host_button)],
        ) = UIManager::new_and_rc_elements(
            event_sender.clone(),
            [
                UIElement::TextInput(TextInput::new(Bounds {
                    relative,
                    absolute: Rect::new(-300.0, -122.0, 240.0, 24.0),
                })),
                UIElement::TextInput(TextInput::new(Bounds {
                    relative,
                    absolute: Rect::new(-300.0, -12.0, 240.0, 24.0),
                })),
                UIElement::TextInput(TextInput::new_masked(
                    Bounds {
                        relative,
                        absolute: Rect::new(160.0, -12.0, 80.0, 24.0),
                    },
                    char::is_numeric,
                )),
                UIElement::Button(Button::new(
                    Bounds::absolute(Rect::new(30.0, 30.0, 120.0, 40.0)),
                    Text::new("Back"),
                    MultiplayerMenuEvent::MainEvent(MainEvent::MainMenu),
                )),
                UIElement::Button(Button::new(
                    Bounds {
                        relative,
                        absolute: Rect::new(-300.0, 16.0, 240.0, 45.0),
                    },
                    Text::new("Join"),
                    MultiplayerMenuEvent::JoinLobby,
                )),
                UIElement::Button(Button::new(
                    Bounds {
                        relative,
                        absolute: Rect::new(60.0, 16.0, 240.0, 45.0),
                    },
                    Text::new("Host"),
                    MultiplayerMenuEvent::HostLobby,
                )),
            ],
        )
        else {
            panic!()
        };
        {
            let mut username_input = username_input.borrow_mut();
            username_input.text = shared.persistent.borrow().username.clone();
            let mut ip_input = destination_input.borrow_mut();
            ip_input.text = shared
                .persistent
                .borrow()
                .destination_address
                .clone()
                .unwrap_or_default();
            let mut port_input = port_input.borrow_mut();
            port_input.maxlen = Some(5);
            port_input.text = shared.persistent.borrow().host_port.to_string();
        }

        MultiplayerMenuClient {
            parent_channel,
            shared,
            _event_sender: event_sender,
            event_receiver,
            ui,
            username_input,
            destination_input,
            port_input,
            join_button,
            host_button,
            error_message: None,
        }
    }

    fn parse_username(&self) -> Result<String, String> {
        let username_input = self.username_input.borrow();
        let text = username_input.text.trim();
        (!text.is_empty())
            .then_some(text.to_string())
            .ok_or("Username cannot be empty".to_string())
    }

    fn parse_port(&self) -> Result<u16, String> {
        self.port_input
            .borrow()
            .text
            .parse()
            .map_err(|e| format!("Invalid port: {e}"))
    }

    fn parse_destination(&self) -> Result<(SocketAddr, String), String> {
        let text = &self.destination_input.borrow().text;
        Ok((
            text.to_socket_addrs()
                .map_err(|e| format!("Invalid destination address: {e}"))
                .map(|x| x.into_iter().next().ok_or("Invalid destination address"))??,
            text.clone(),
        ))
    }

    fn handle_event(&mut self, _ctx: &mut Context, event: MultiplayerMenuEvent) -> GameResult<()> {
        trace!("event = {event:?}");
        match event {
            MultiplayerMenuEvent::MainEvent(main_event) => {
                self.parent_channel.send(main_event).unwrap()
            }
            MultiplayerMenuEvent::JoinLobby => {
                let result = try {
                    let username = self.parse_username()?;
                    let (destination, destination_name) = self.parse_destination()?;
                    let mut persistent = self.shared.persistent.borrow_mut();
                    persistent.username = username.clone();
                    persistent.destination_address = Some(destination_name.clone());
                    let (ip, port) = (destination.ip(), destination.port());
                    self.parent_channel.send(MainEvent::MultiplayerJoin {
                        username,
                        socket: SocketAddr::new(ip, port),
                        destination_name,
                    })
                };
                if let Err(errmsg) = result {
                    self.error_message = Some((errmsg, Instant::now()))
                }
            }
            MultiplayerMenuEvent::HostLobby => {
                let result = try {
                    let username = self.parse_username()?;
                    let port = self.parse_port()?;
                    let mut persistent = self.shared.persistent.borrow_mut();
                    persistent.username = username.clone();
                    persistent.host_port = port;
                    self.parent_channel
                        .send(MainEvent::MultiplayerHost { username, port })
                };
                if let Err(errmsg) = result {
                    self.error_message = Some((errmsg, Instant::now()))
                }
            }
        }
        Ok(())
    }
}

impl SubEventHandler for MultiplayerMenuClient {
    fn update(&mut self, ctx: &mut Context) -> Result<(), GameError> {
        self.ui.update(ctx)?;
        while let Ok(event) = self.event_receiver.try_recv() {
            self.handle_event(ctx, event)?;
        }

        if let Some((_, error_reported)) = self.error_message {
            if Instant::now() - error_reported > ERROR_DISPLAY_PERIOD {
                self.error_message = None;
            }
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context, canvas: &mut Canvas) -> Result<(), GameError> {
        let res = ctx.res();

        Text::new("Multiplayer")
            .size(96.0)
            .anchored_by(
                ctx,
                res * vec2(0.5, 0.0) + vec2(0.0, 20.0),
                AnchorPoint::NorthCenter,
            )?
            .color(Color::BLACK)
            .draw(canvas);

        self.username_input.borrow().render_label(
            ctx,
            canvas,
            &Text::new("Username:"),
            AnchorPoint::CenterWest,
        )?;

        Text::new("Join Lobby")
            .size(24.0)
            .anchored_by(
                ctx,
                self.join_button
                    .borrow()
                    .bounds
                    .corrected_bounds(res)
                    .parametric(vec2(0.5, 0.0))
                    - vec2(0.0, 40.0),
                AnchorPoint::SouthCenter,
            )?
            .color(Color::BLACK)
            .draw(canvas);

        self.destination_input.borrow().render_label(
            ctx,
            canvas,
            &Text::new("Destination address:"),
            AnchorPoint::CenterWest,
        )?;

        Text::new("Host Lobby")
            .size(24.0)
            .anchored_by(
                ctx,
                self.host_button
                    .borrow()
                    .bounds
                    .corrected_bounds(res)
                    .parametric(vec2(0.5, 0.0))
                    - vec2(0.0, 40.0),
                AnchorPoint::SouthCenter,
            )?
            .color(Color::BLACK)
            .draw(canvas);

        self.port_input.borrow().render_label(
            ctx,
            canvas,
            &Text::new("Port:"),
            AnchorPoint::CenterWest,
        )?;

        if let Some((err_msg, _)) = &self.error_message {
            Text::new(err_msg)
                .anchored_by(
                    ctx,
                    ctx.res() * vec2(0.5, 0.5) + vec2(0.0, 80.0),
                    AnchorPoint::NorthCenter,
                )?
                .color(Color::from_rgb(96, 0, 0))
                .draw(canvas);
        }

        self.ui.draw(ctx, canvas)?;
        Ok(())
    }
}

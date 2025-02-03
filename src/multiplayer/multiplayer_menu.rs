use std::{
    cell::RefCell,
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

use crate::{
    main_client::MainEvent,
    sub_event_handler::SubEventHandler,
    ui_manager::{Bounds, Button, TextInput, UIElement, UIManager},
    util::{AnchorPoint, ContextExt, RectExt, TextExt},
    Args,
};

const ERROR_DISPLAY_PERIOD: Duration = Duration::from_secs(10);

#[derive(Clone)]
enum MultiplayerMenuEvent {
    MainEvent(MainEvent),
    JoinLobby,
    HostLobby,
}

pub struct MultiplayerMenuClient {
    parent_channel: Sender<MainEvent>,
    _args: Args,
    _event_sender: Sender<MultiplayerMenuEvent>,
    event_receiver: Receiver<MultiplayerMenuEvent>,
    ui: UIManager<MultiplayerMenuEvent, MultiplayerMenuEvent>,
    ip_input: Rc<RefCell<TextInput>>,
    port_input: Rc<RefCell<TextInput>>,
    join_button: Rc<RefCell<Button<MultiplayerMenuEvent>>>,
    host_button: Rc<RefCell<Button<MultiplayerMenuEvent>>>,
    error_message: Option<(String, Instant)>,
}

impl MultiplayerMenuClient {
    pub fn new(parent_channel: Sender<MainEvent>, args: Args) -> MultiplayerMenuClient {
        let (event_sender, event_receiver) = channel();
        let (
            ui,
            [UIElement::TextInput(ip_input), UIElement::TextInput(port_input), _, UIElement::Button(join_button), UIElement::Button(host_button)],
        ) = UIManager::new_and_rc_elements(
            event_sender.clone(),
            [
                UIElement::TextInput(TextInput::new(Bounds {
                    relative: Rect::new(0.5, 0.5, 0.0, 0.0),
                    absolute: Rect::new(10.0, -12.0, 240.0, 24.0),
                })),
                UIElement::TextInput(TextInput::new_masked(
                    Bounds {
                        relative: Rect::new(0.5, 0.5, 0.0, 0.0),
                        absolute: Rect::new(10.0, 32.0, 80.0, 24.0),
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
                        relative: Rect::new(0.5, 0.5, 0.0, 0.0),
                        absolute: Rect::new(-200.0, 80.0, 400.0, 45.0),
                    },
                    Text::new("Join lobby at"),
                    MultiplayerMenuEvent::JoinLobby,
                )),
                UIElement::Button(Button::new(
                    Bounds {
                        relative: Rect::new(0.5, 0.5, 0.0, 0.0),
                        absolute: Rect::new(-200.0, 155.0, 400.0, 45.0),
                    },
                    Text::new("Host lobby on port "),
                    MultiplayerMenuEvent::HostLobby,
                )),
            ],
        )
        else {
            panic!()
        };
        {
            let mut ip_input = ip_input.borrow_mut();
            ip_input.text = args.ip.map_or(String::new(), |ip| ip.to_string());
            let mut port_input = port_input.borrow_mut();
            port_input.maxlen = Some(5);
            port_input.text = args.port.to_string();
        }

        MultiplayerMenuClient {
            parent_channel,
            _args: args,
            _event_sender: event_sender,
            event_receiver,
            ui,
            ip_input,
            port_input,
            join_button,
            host_button,
            error_message: None,
        }
    }

    fn parse_port(&self) -> Result<u16, String> {
        self.port_input
            .borrow()
            .text
            .parse()
            .map_err(|e| format!("Invalid port: {e}"))
    }

    fn parse_ip(&self) -> Result<IpAddr, String> {
        self.ip_input
            .borrow()
            .text
            .parse()
            .map_err(|e| format!("Invalid IP address: {e}"))
    }

    fn handle_event(&mut self, _ctx: &mut Context, event: MultiplayerMenuEvent) -> GameResult<()> {
        match event {
            MultiplayerMenuEvent::MainEvent(main_event) => {
                self.parent_channel.send(main_event).unwrap()
            }
            MultiplayerMenuEvent::JoinLobby => {
                let result = try {
                    let ip = self.parse_ip()?;
                    let port = self.parse_port()?;
                    self.parent_channel
                        .send(MainEvent::MultiplayerJoin(SocketAddr::new(ip, port)))
                };
                if let Err(errmsg) = result {
                    self.error_message = Some((errmsg, Instant::now()))
                }
            }
            MultiplayerMenuEvent::HostLobby => {
                let result = try {
                    let port = self.parse_port()?;
                    self.parent_channel.send(MainEvent::MultiplayerHost(port))
                };
                if let Err(errmsg) = result {
                    self.error_message = Some((errmsg, Instant::now()))
                }
            }
        }
        Ok(())
    }
}

impl SubEventHandler<GameError> for MultiplayerMenuClient {
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

        {
            let ip_input = self.ip_input.borrow();
            let port_input = self.port_input.borrow();
            self.join_button.borrow_mut().text = Text::new(format!(
                "Join lobby at {}:{}",
                ip_input.text, port_input.text
            ));
            self.host_button.borrow_mut().text =
                Text::new(format!("Host lobby on port {}", port_input.text));
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

        Text::new("IP:")
            .anchored_by(
                ctx,
                self.ip_input
                    .borrow()
                    .bounds
                    .corrected_bounds(res)
                    .parametric(vec2(0.0, 0.5))
                    - vec2(6.0, 0.0),
                AnchorPoint::CenterEast,
            )?
            .color(Color::BLACK)
            .draw(canvas);

        Text::new("Port:")
            .anchored_by(
                ctx,
                self.port_input
                    .borrow()
                    .bounds
                    .corrected_bounds(res)
                    .parametric(vec2(0.0, 0.5))
                    - vec2(6.0, 0.0),
                AnchorPoint::CenterEast,
            )?
            .color(Color::BLACK)
            .draw(canvas);

        if let Some((err_msg, _)) = &self.error_message {
            Text::new(err_msg)
                .anchored_by(
                    ctx,
                    self.host_button
                        .borrow()
                        .bounds
                        .corrected_bounds(res)
                        .parametric(vec2(0.5, 1.0))
                        + vec2(0.0, 20.0),
                    AnchorPoint::NorthCenter,
                )?
                .color(Color::from_rgb(96, 0, 0))
                .draw(canvas);
        }

        self.ui.draw(ctx, canvas)?;
        Ok(())
    }
}

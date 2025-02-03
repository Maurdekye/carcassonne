use std::{
    cell::RefCell,
    rc::Rc,
    sync::mpsc::{channel, Receiver, Sender},
};

use clap::crate_version;
use ggez::{
    glam::{vec2, Vec2},
    graphics::{Canvas, Color, DrawMode, Mesh, Rect, Text},
    GameError,
};

use crate::{
    game_client::{GameClient, NUM_PLAYERS, PLAYER_COLORS},
    main_client::MainEvent,
    sub_event_handler::SubEventHandler,
    ui_manager::{Bounds, Button, UIElement, UIElementState, UIManager, BUTTON_COLOR},
    util::{AnchorPoint, ContextExt, DrawableWihParamsExt, TextExt},
    Args,
};

#[derive(Clone, Debug)]
enum MainMenuEvent {
    MainEvent(MainEvent),
    SelectColor(Color),
    StartGame,
}

impl From<Color> for MainMenuEvent {
    fn from(value: Color) -> Self {
        MainMenuEvent::SelectColor(value)
    }
}

pub struct MainMenuClient {
    parent_channel: Sender<MainEvent>,
    _event_sender: Sender<MainMenuEvent>,
    event_receiver: Receiver<MainMenuEvent>,
    _args: Args,
    ui: UIManager<MainMenuEvent, MainMenuEvent>,
    color_selection_ui: UIManager<Color, MainMenuEvent>,
    color_selection_buttons: [Rc<RefCell<Button<Color>>>; NUM_PLAYERS],
    selected_colors: Vec<Color>,
    start_game_button: Rc<RefCell<Button<MainMenuEvent>>>,
}

impl MainMenuClient {
    const BUTTONS_CENTER: Rect = Rect::new(0.5, 0.4, 0.0, 0.0);
    const BUTTON_SIZE: f32 = 40.0;
    const BUTTON_SPACING: f32 = 10.0;

    const SELECTED_COLOR: Color = Color {
        r: 0.5,
        g: 0.78,
        b: 0.5,
        a: 1.0,
    };
    const DESELECTED_COLOR: Color = BUTTON_COLOR;

    pub fn new(parent_channel: Sender<MainEvent>, args: Args) -> MainMenuClient {
        let (event_sender, event_receiver) = channel();
        let ui_sender = event_sender.clone();
        let (ui, [UIElement::Button(start_game_button), ..]) = UIManager::new_and_rc_elements(
            ui_sender,
            [
                UIElement::Button(Button::new(
                    Bounds {
                        relative: Self::BUTTONS_CENTER,
                        absolute: Rect::new(
                            -120.0,
                            Self::BUTTON_SIZE * 2.0 + Self::BUTTON_SPACING,
                            240.0,
                            40.0,
                        ),
                    },
                    Text::new("Start Local 0 Player Game"),
                    MainMenuEvent::StartGame,
                )),
                UIElement::Button(Button::new(
                    Bounds {
                        relative: Self::BUTTONS_CENTER,
                        absolute: Rect::new(
                            -120.0,
                            Self::BUTTON_SIZE * 2.0 + Self::BUTTON_SPACING + 60.0,
                            240.0,
                            40.0,
                        ),
                    },
                    Text::new("Multiplayer"),
                    MainMenuEvent::MainEvent(MainEvent::MultiplayerMenu),
                )),
                UIElement::Button(Button::new(
                    Bounds {
                        relative: Rect::new(0.5, 1.0, 0.0, 0.0),
                        absolute: Rect::new(-90.0, -80.0, 180.0, 60.0),
                    },
                    Text::new("Quit"),
                    MainMenuEvent::MainEvent(MainEvent::Close),
                )),
            ],
        ) else {
            panic!()
        };
        start_game_button.borrow_mut().state = UIElementState::Disabled;
        let (color_selection_ui, color_selection_buttons) = {
            let full_width = (Self::BUTTON_SIZE * NUM_PLAYERS as f32)
                + (Self::BUTTON_SPACING * (NUM_PLAYERS - 1) as f32);
            let ui_sender = event_sender.clone();
            let mut i = 0;
            UIManager::new_and_rc_elements(
                ui_sender,
                PLAYER_COLORS.map(|color| {
                    i += 1;
                    let offset = (i - 1) as f32;
                    UIElement::Button(Button::new(
                        Bounds {
                            relative: Self::BUTTONS_CENTER,
                            absolute: Rect::new(
                                (Self::BUTTON_SIZE + Self::BUTTON_SPACING) * offset
                                    - full_width / 2.0,
                                -Self::BUTTON_SIZE / 2.0,
                                Self::BUTTON_SIZE,
                                Self::BUTTON_SIZE,
                            ),
                        },
                        Text::new(""),
                        color,
                    ))
                }),
            )
        };
        let color_selection_buttons = color_selection_buttons.map(UIElement::unwrap_button);
        MainMenuClient {
            parent_channel,
            _event_sender: event_sender,
            event_receiver,
            _args: args,
            color_selection_ui,
            color_selection_buttons,
            selected_colors: Vec::new(),
            start_game_button,
            ui,
        }
    }

    fn handle_event(&mut self, event: MainMenuEvent) -> Result<(), GameError> {
        match event {
            MainMenuEvent::MainEvent(event) => self.parent_channel.send(event).unwrap(),
            MainMenuEvent::SelectColor(color) => {
                #[allow(clippy::match_like_matches_macro)] // breaks rustfmt
                let mut button = self
                    .color_selection_buttons
                    .iter()
                    .map(|button| button.borrow_mut())
                    .find(|button| button.event == color)
                    .unwrap();
                if self.selected_colors.contains(&color) {
                    self.selected_colors.retain(|c| *c != color);
                    button.color = Self::DESELECTED_COLOR;
                } else {
                    self.selected_colors.push(color);
                    button.color = Self::SELECTED_COLOR;
                }
                let mut start_game_button = self.start_game_button.borrow_mut();
                start_game_button.state =
                    UIElementState::disabled_if(self.selected_colors.len() < 2);
                start_game_button.text = Text::new(format!(
                    "Start Local {} Player Game",
                    self.selected_colors.len()
                ));
            }
            MainMenuEvent::StartGame => {
                if self.selected_colors.len() < 2 {
                    eprintln!("Can't start a game with less than two players!");
                } else {
                    self.parent_channel
                        .send(MainEvent::StartGame(self.selected_colors.clone()))
                        .unwrap()
                }
            }
        }
        Ok(())
    }
}

impl SubEventHandler<GameError> for MainMenuClient {
    fn update(&mut self, ctx: &mut ggez::Context) -> Result<(), GameError> {
        self.ui.update(ctx)?;
        self.color_selection_ui.update(ctx)?;
        while let Ok(event) = self.event_receiver.try_recv() {
            self.handle_event(event)?;
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut ggez::Context, canvas: &mut Canvas) -> Result<(), GameError> {
        let res = ctx.res();

        // render title
        Text::new("Carcassonne")
            .size(144.0)
            .centered_on(ctx, res * vec2(0.5, 0.2))?
            .color(Color::BLACK)
            .draw(canvas);

        // render version
        Text::new(format!("v{}", crate_version!()))
            .size(32.0)
            .anchored_by(
                ctx,
                res * vec2(0.0, 1.0) + vec2(20.0, -20.0),
                AnchorPoint::SouthWest,
            )?
            .color(Color::from_rgb(196, 196, 196))
            .draw(canvas);

        // render ui
        self.ui.draw(ctx, canvas)?;

        // render player choice buttons
        self.color_selection_ui.draw(ctx, canvas)?;

        for (button, color) in self.color_selection_buttons.iter().zip(PLAYER_COLORS) {
            let button = button.borrow();
            let center = button.corrected_bounds(res).center().into();
            GameClient::draw_meeple(ctx, canvas, center, color, 0.1)?;
        }

        // render selected meeple colors
        let width = self.selected_colors.len() as f32 * 40.0;
        let top_left = Vec2::from(Self::BUTTONS_CENTER.point()) * res + vec2(0.0, 30.0)
            - vec2(width / 2.0, 0.0);
        let panel = Rect::new(top_left.x, top_left.y, width, 40.0);
        Mesh::new_rounded_rectangle(
            ctx,
            DrawMode::fill(),
            panel,
            6.0,
            Color::from_rgb(160, 160, 160),
        )?
        .draw(canvas);
        for (i, color) in self.selected_colors.iter().enumerate() {
            let center = top_left + vec2(20.0 + 40.0 * i as f32, 20.0);
            GameClient::draw_meeple(ctx, canvas, center, *color, 0.1)?;
        }

        Ok(())
    }
}

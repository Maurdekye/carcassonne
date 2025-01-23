use std::sync::mpsc::{channel, Receiver, Sender};

use ggez::{
    event::EventHandler,
    glam::{vec2, Vec2},
    graphics::{Canvas, Color, DrawParam, Rect, Text},
    GameError,
};

use crate::{
    main_client::MainEvent,
    ui_manager::{Button, ButtonBounds, UIManager},
    Args,
};

#[derive(Clone, Debug)]
enum MainMenuEvent {
    MainEvent(MainEvent),
}

pub struct MainMenuClient {
    parent_channel: Sender<MainEvent>,
    _event_sender: Sender<MainMenuEvent>,
    event_receiver: Receiver<MainMenuEvent>,
    _args: Args,
    ui: UIManager<MainMenuEvent>,
}

impl MainMenuClient {
    pub fn new(parent_channel: Sender<MainEvent>, args: Args) -> MainMenuClient {
        let buttons_center = Rect::new(0.5, 0.65, 0.0, 0.0);
        let (_event_sender, event_receiver) = channel();
        let ui_sender = _event_sender.clone();
        let (ui, [..]) = UIManager::new_and_rc_buttons(
            ui_sender,
            [
                Button::new(
                    ButtonBounds {
                        relative: buttons_center,
                        absolute: Rect::new(-200.0, -80.0, 180.0, 60.0),
                    },
                    Text::new("2 Players"),
                    MainMenuEvent::MainEvent(MainEvent::StartGame(2)),
                ),
                Button::new(
                    ButtonBounds {
                        relative: buttons_center,
                        absolute: Rect::new(20.0, -80.0, 180.0, 60.0),
                    },
                    Text::new("3 Players"),
                    MainMenuEvent::MainEvent(MainEvent::StartGame(3)),
                ),
                Button::new(
                    ButtonBounds {
                        relative: buttons_center,
                        absolute: Rect::new(-200.0, 20.0, 180.0, 60.0),
                    },
                    Text::new("4 Players"),
                    MainMenuEvent::MainEvent(MainEvent::StartGame(4)),
                ),
                Button::new(
                    ButtonBounds {
                        relative: buttons_center,
                        absolute: Rect::new(20.0, 20.0, 180.0, 60.0),
                    },
                    Text::new("5 Players"),
                    MainMenuEvent::MainEvent(MainEvent::StartGame(5)),
                ),
                Button::new(
                    ButtonBounds {
                        relative: Rect::new(0.5, 1.0, 0.0, 0.0),
                        absolute: Rect::new(-90.0, -80.0, 180.0, 60.0),
                    },
                    Text::new("Quit"),
                    MainMenuEvent::MainEvent(MainEvent::Close),
                ),
            ],
        );
        MainMenuClient {
            parent_channel,
            _event_sender,
            event_receiver,
            _args: args,
            ui,
        }
    }
}

impl EventHandler<GameError> for MainMenuClient {
    fn update(&mut self, ctx: &mut ggez::Context) -> Result<(), GameError> {
        self.ui.update(ctx);
        while let Ok(event) = self.event_receiver.try_recv() {
            match event {
                MainMenuEvent::MainEvent(event) => self.parent_channel.send(event).unwrap(),
            }
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut ggez::Context) -> Result<(), GameError> {
        let res: Vec2 = ctx.gfx.drawable_size().into();
        let mut canvas = Canvas::from_frame(ctx, Color::WHITE);

        // render title
        let mut menu_text = Text::new("Carcassonne");
        menu_text.set_scale(144.0);
        let text_size: Vec2 = menu_text.measure(ctx)?.into();
        canvas.draw(
            &menu_text,
            DrawParam::from((res - text_size) * vec2(0.5, 0.2)).color(Color::BLACK),
        );

        // render ui
        self.ui.draw(ctx, &mut canvas)?;

        canvas.finish(ctx)
    }
}

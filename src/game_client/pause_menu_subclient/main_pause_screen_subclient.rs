use ggez::{
    graphics::{Canvas, Rect, Text},
    input::keyboard::KeyCode,
    Context, GameError,
};
use std::sync::mpsc::{channel, Receiver, Sender};

use crate::{
    game_client::GameEvent,
    main_client::MainEvent,
    sub_event_handler::SubEventHandler,
    ui_manager::{Button, ButtonBounds, UIManager},
};

use super::PauseMenuEvent;

#[derive(Clone)]
enum MainPauseScreenEvent {
    PauseMenuEvent(PauseMenuEvent),
}

impl MainPauseScreenEvent {
    fn game_event(event: GameEvent) -> Self {
        Self::PauseMenuEvent(PauseMenuEvent::GameEvent(event))
    }

    fn main_event(event: MainEvent) -> Self {
        Self::game_event(GameEvent::MainEvent(event))
    }
}

pub struct MainPauseScreenSubclient {
    parent_channel: Sender<PauseMenuEvent>,
    event_sender: Sender<MainPauseScreenEvent>,
    event_receiver: Receiver<MainPauseScreenEvent>,
    ui: UIManager<MainPauseScreenEvent>,
}

impl MainPauseScreenSubclient {
    pub fn new(parent_channel: Sender<PauseMenuEvent>) -> MainPauseScreenSubclient {
        let (event_sender, event_receiver) = channel();
        let ui_sender = event_sender.clone();
        let button_center = Rect::new(0.5, 0.2, 0.0, 0.0);
        MainPauseScreenSubclient {
            parent_channel,
            event_sender,
            event_receiver,
            ui: UIManager::new(
                ui_sender,
                vec![
                    Button::new(
                        ButtonBounds {
                            relative: button_center,
                            absolute: Rect::new(-120.0, 0.0, 240.0, 40.0),
                        },
                        Text::new("End Game"),
                        MainPauseScreenEvent::game_event(GameEvent::EndGame),
                    ),
                    Button::new(
                        ButtonBounds {
                            relative: button_center,
                            absolute: Rect::new(-120.0, 60.0, 240.0, 40.0),
                        },
                        Text::new("Return to Main Menu"),
                        MainPauseScreenEvent::main_event(MainEvent::ReturnToMainMenu),
                    ),
                ],
            ),
        }
    }

    fn handle_event(
        &mut self,
        _ctx: &mut Context,
        event: MainPauseScreenEvent,
    ) -> Result<(), GameError> {
        use MainPauseScreenEvent::*;
        match event {
            PauseMenuEvent(event) => self.parent_channel.send(event).unwrap(),
        }
        Ok(())
    }
}

impl SubEventHandler<GameError> for MainPauseScreenSubclient {
    fn update(&mut self, ctx: &mut Context) -> Result<(), GameError> {
        self.ui.update(ctx);

        if ctx.keyboard.is_key_just_pressed(KeyCode::Escape) {
            self.event_sender
                .send(MainPauseScreenEvent::game_event(GameEvent::ClosePauseMenu))
                .unwrap();
        }

        while let Ok(event) = self.event_receiver.try_recv() {
            self.handle_event(ctx, event)?;
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context, canvas: &mut Canvas) -> Result<(), GameError> {
        self.ui.draw(ctx, canvas)
    }
}

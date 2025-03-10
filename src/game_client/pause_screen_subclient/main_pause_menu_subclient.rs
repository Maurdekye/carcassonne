use ggez::{
    graphics::{Canvas, Rect, Text},
    Context, GameError,
};
use log::trace;
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
    sync::mpsc::{channel, Receiver, Sender},
};

use crate::{game_client::GameEvent, main_client::MainEvent, shared::Shared};

use ggez_no_re::{
    sub_event_handler::SubEventHandler,
    ui_manager::{Bounds, button::Button, UIElement, UIElementState, UIManager},
};

use super::PauseScreenEvent;

#[derive(Debug, Clone)]
pub enum MainPauseMenuEvent {
    PauseScreenEvent(PauseScreenEvent),
}

impl MainPauseMenuEvent {
    fn game_event(event: GameEvent) -> Self {
        Self::PauseScreenEvent(PauseScreenEvent::GameEvent(event))
    }

    fn main_event(event: MainEvent) -> Self {
        Self::game_event(GameEvent::MainEvent(event))
    }
}

pub struct MainPauseMenuSubclient {
    shared: Shared,
    parent_channel: Sender<PauseScreenEvent>,
    event_sender: Sender<MainPauseMenuEvent>,
    event_receiver: Receiver<MainPauseMenuEvent>,
    ui: UIManager<MainPauseMenuEvent, MainPauseMenuEvent>,
    can_end_game: Rc<Cell<bool>>,
    can_undo: Rc<Cell<bool>>,
    end_game_button: Rc<RefCell<Button<MainPauseMenuEvent>>>,
    undo_button: Rc<RefCell<Button<MainPauseMenuEvent>>>,
}

impl MainPauseMenuSubclient {
    pub fn new(
        shared: Shared,
        parent_channel: Sender<PauseScreenEvent>,
        can_end_game: Rc<Cell<bool>>,
        can_undo: Rc<Cell<bool>>,
    ) -> MainPauseMenuSubclient {
        let (event_sender, event_receiver) = channel();
        let ui_sender = event_sender.clone();
        let button_center = Rect::new(0.5, 0.2, 0.0, 0.0);
        let (ui, [UIElement::Button(end_game_button), _, _, UIElement::Button(undo_button), ..]) =
            UIManager::new_and_rc_elements(
                ui_sender,
                [
                    UIElement::Button(Button::new(
                        Bounds {
                            relative: button_center,
                            absolute: Rect::new(-250.0, 0.0, 240.0, 40.0),
                        },
                        Text::new("End Game"),
                        MainPauseMenuEvent::game_event(GameEvent::EndGame),
                    )),
                    UIElement::Button(Button::new(
                        Bounds {
                            relative: button_center,
                            absolute: Rect::new(10.0, 0.0, 240.0, 40.0),
                        },
                        Text::new("Reset Camera"),
                        MainPauseMenuEvent::game_event(GameEvent::ResetCamera),
                    )),
                    UIElement::Button(Button::new(
                        Bounds {
                            relative: button_center,
                            absolute: Rect::new(-250.0, 60.0, 240.0, 40.0),
                        },
                        Text::new("Return to Main Menu"),
                        MainPauseMenuEvent::main_event(MainEvent::MainMenu),
                    )),
                    UIElement::Button(Button::new(
                        Bounds {
                            relative: button_center,
                            absolute: Rect::new(10.0, 60.0, 240.0, 40.0),
                        },
                        Text::new("Undo Last Move"),
                        MainPauseMenuEvent::game_event(GameEvent::Undo),
                    )),
                    UIElement::Button(Button::new(
                        Bounds {
                            relative: button_center,
                            absolute: Rect::new(-250.0, 120.0, 240.0, 40.0),
                        },
                        Text::new("Controls"),
                        MainPauseMenuEvent::PauseScreenEvent(PauseScreenEvent::Controls),
                    )),
                    UIElement::Button(Button::new(
                        Bounds {
                            relative: button_center,
                            absolute: Rect::new(10.0, 120.0, 240.0, 40.0),
                        },
                        Text::new("Rules"),
                        MainPauseMenuEvent::PauseScreenEvent(PauseScreenEvent::Rules),
                    )),
                ],
            )
        else {
            panic!()
        };
        end_game_button.borrow_mut().state = UIElementState::disabled_if(can_end_game.get());
        undo_button.borrow_mut().state = UIElementState::disabled_if(!can_undo.get());
        MainPauseMenuSubclient {
            shared,
            parent_channel,
            event_sender,
            event_receiver,
            ui,
            can_undo,
            can_end_game,
            end_game_button,
            undo_button,
        }
    }

    fn handle_event(
        &mut self,
        _ctx: &mut Context,
        event: MainPauseMenuEvent,
    ) -> Result<(), GameError> {
        trace!("event = {event:?}");
        use MainPauseMenuEvent::*;
        match event {
            PauseScreenEvent(event) => self.parent_channel.send(event).unwrap(),
        }
        Ok(())
    }
}

impl SubEventHandler for MainPauseMenuSubclient {
    fn update(&mut self, ctx: &mut Context) -> Result<(), GameError> {
        self.end_game_button.borrow_mut().state =
            UIElementState::disabled_if(self.can_end_game.get());
        self.undo_button.borrow_mut().state = UIElementState::disabled_if(!self.can_undo.get());
        self.ui.update(ctx)?;

        if self
            .shared
            .persistent
            .borrow()
            .keybinds
            .pause
            .just_pressed(ctx)
        {
            self.event_sender
                .send(MainPauseMenuEvent::game_event(GameEvent::ClosePauseMenu))
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

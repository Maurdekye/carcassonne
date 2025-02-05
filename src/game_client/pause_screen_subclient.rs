use controls_menu_subclient::ControlsMenuSubclient;
use ggez::{
    glam::{vec2, Vec2},
    graphics::{Canvas, Color, DrawMode, Mesh, Rect, Text},
    Context, GameError,
};
use log::trace;
use main_pause_menu_subclient::MainPauseMenuSubclient;
use rules_menu_subclient::RulesMenuSubclient;
use std::{
    cell::Cell,
    rc::Rc,
    sync::mpsc::{channel, Receiver, Sender},
};

use crate::{
    game_client::GameEvent,
    sub_event_handler::SubEventHandler,
    ui_manager::{Bounds, Button, UIElement, UIManager},
    util::{AnchorPoint, DrawableWihParamsExt, TextExt},
};

mod controls_menu_subclient;
mod main_pause_menu_subclient;
mod rules_menu_subclient;

#[derive(Debug, Clone)]
pub enum PauseScreenEvent {
    GameEvent(GameEvent),
    MainMenu,
    Controls,
    Rules,
}

pub struct PauseScreenSubclient {
    scene: Box<dyn SubEventHandler<GameError>>,
    parent_channel: Sender<GameEvent>,
    event_sender: Sender<PauseScreenEvent>,
    event_receiver: Receiver<PauseScreenEvent>,
    ui: UIManager<PauseScreenEvent, PauseScreenEvent>,
    pub can_end_game: Rc<Cell<bool>>,
    pub can_undo: Rc<Cell<bool>>,
}

impl PauseScreenSubclient {
    pub fn new(
        parent_channel: Sender<GameEvent>,
        can_end_game: bool,
        can_undo: bool,
    ) -> PauseScreenSubclient {
        let (event_sender, event_receiver) = channel();
        let ui_sender = event_sender.clone();
        let can_end_game = Rc::new(Cell::new(can_end_game));
        let can_undo = Rc::new(Cell::new(can_undo));
        PauseScreenSubclient {
            parent_channel,
            scene: Box::new(MainPauseMenuSubclient::new(
                event_sender.clone(),
                can_end_game.clone(),
                can_undo.clone(),
            )),
            event_sender,
            event_receiver,
            can_end_game,
            can_undo,
            ui: UIManager::new(
                ui_sender,
                [UIElement::Button(Button::new(
                    Bounds::absolute(Rect::new(20.0, 20.0, 30.0, 30.0)),
                    Text::new("X"),
                    PauseScreenEvent::GameEvent(GameEvent::ClosePauseMenu),
                ))],
            ),
        }
    }

    pub fn handle_event(
        &mut self,
        _ctx: &mut Context,
        event: PauseScreenEvent,
    ) -> Result<(), GameError> {
        use PauseScreenEvent::*;
        trace!("event = {event:?}");
        match event {
            GameEvent(event) => self.parent_channel.send(event).unwrap(),
            MainMenu => {
                self.scene = Box::new(MainPauseMenuSubclient::new(
                    self.event_sender.clone(),
                    self.can_end_game.clone(),
                    self.can_undo.clone(),
                ))
            }
            Controls => {
                self.scene = Box::new(ControlsMenuSubclient::new(self.event_sender.clone()))
            }
            Rules => self.scene = Box::new(RulesMenuSubclient::new(self.event_sender.clone())),
        }
        Ok(())
    }
}

impl SubEventHandler<GameError> for PauseScreenSubclient {
    fn update(&mut self, ctx: &mut Context) -> Result<(), GameError> {
        self.scene.update(ctx)?;
        self.ui.update(ctx)?;
        while let Ok(event) = self.event_receiver.try_recv() {
            self.handle_event(ctx, event)?;
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context, canvas: &mut Canvas) -> Result<(), GameError> {
        let res: Vec2 = ctx.gfx.drawable_size().into();

        // background
        let screen_bounds = Rect::new(0.0, 0.0, res.x, res.y);
        Mesh::new_rectangle(
            ctx,
            DrawMode::fill(),
            screen_bounds,
            Color::from_rgba(128, 128, 128, 128),
        )?
        .draw(canvas);
        Mesh::new_rectangle(
            ctx,
            DrawMode::stroke(32.0),
            screen_bounds,
            Color::from_rgb(96, 96, 96),
        )?
        .draw(canvas);

        // "paused" text
        Text::new("Paused")
            .size(48.0)
            .anchored_by(
                ctx,
                vec2(res.x, 0.0) + vec2(-20.0, 20.0),
                AnchorPoint::NorthEast,
            )?
            .color(Color::BLACK)
            .draw(canvas);

        self.ui.draw(ctx, canvas)?;
        self.scene.draw(ctx, canvas)
    }
}

use controls_menu_subclient::ControlsMenuSubclient;
use ggez::{
    glam::{vec2, Vec2},
    graphics::{Canvas, Color, DrawMode, Mesh, Rect, Text},
    Context, GameError,
};
use main_pause_menu_subclient::MainPauseMenuSubclient;
use std::sync::mpsc::{channel, Receiver, Sender};

use crate::{
    game_client::GameEvent,
    sub_event_handler::SubEventHandler,
    ui_manager::{Button, ButtonBounds, UIManager},
    util::{AnchorPoint, DrawableWihParamsExt, TextExt},
};

mod main_pause_menu_subclient;
mod controls_menu_subclient;

#[derive(Debug, Clone)]
pub enum PauseScreenEvent {
    GameEvent(GameEvent),
    MainMenu,
    Controls,
}

pub struct PauseScreenSubclient {
    scene: Box<dyn SubEventHandler<GameError>>,
    parent_channel: Sender<GameEvent>,
    event_sender: Sender<PauseScreenEvent>,
    event_receiver: Receiver<PauseScreenEvent>,
    ui: UIManager<PauseScreenEvent, PauseScreenEvent>,
    is_endgame: bool,
    has_history: bool,
}

impl PauseScreenSubclient {
    pub fn new(
        parent_channel: Sender<GameEvent>,
        is_endgame: bool,
        has_history: bool,
    ) -> PauseScreenSubclient {
        let (event_sender, event_receiver) = channel();
        let ui_sender = event_sender.clone();
        PauseScreenSubclient {
            parent_channel,
            scene: Box::new(MainPauseMenuSubclient::new(
                event_sender.clone(),
                is_endgame,
                has_history,
            )),
            event_sender,
            event_receiver,
            is_endgame,
            has_history,
            ui: UIManager::new(
                ui_sender,
                [Button::new(
                    ButtonBounds::absolute(Rect::new(20.0, 20.0, 30.0, 30.0)),
                    Text::new("X"),
                    PauseScreenEvent::GameEvent(GameEvent::ClosePauseMenu),
                )],
            ),
        }
    }

    pub fn handle_event(
        &mut self,
        _ctx: &mut Context,
        event: PauseScreenEvent,
    ) -> Result<(), GameError> {
        use PauseScreenEvent::*;
        match event {
            GameEvent(event) => self.parent_channel.send(event).unwrap(),
            MainMenu => {
                self.scene = Box::new(MainPauseMenuSubclient::new(
                    self.event_sender.clone(),
                    self.is_endgame,
                    self.has_history,
                ))
            }
            Controls => {
                self.scene = Box::new(ControlsMenuSubclient::new(self.event_sender.clone()))
            }
        }
        Ok(())
    }
}

impl SubEventHandler<GameError> for PauseScreenSubclient {
    fn update(&mut self, ctx: &mut Context) -> Result<(), GameError> {
        self.scene.update(ctx)?;
        self.ui.update(ctx);
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

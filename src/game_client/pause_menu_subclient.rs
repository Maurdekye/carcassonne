use ggez::{
    glam::{vec2, Vec2},
    graphics::{Canvas, Color, DrawMode, Mesh, Rect, Text},
    Context, GameError,
};
use main_pause_screen_subclient::MainPauseScreenSubclient;
use std::sync::mpsc::{channel, Receiver, Sender};

use crate::{
    game_client::GameEvent,
    sub_event_handler::SubEventHandler,
    ui_manager::{Button, ButtonBounds, UIManager},
    util::{DrawableWihParamsExt, TextExt},
};

mod main_pause_screen_subclient;

#[derive(Clone)]
pub enum PauseMenuEvent {
    GameEvent(GameEvent),
}

pub struct PauseMenuSubclient {
    scene: Box<dyn SubEventHandler<GameError>>,
    parent_channel: Sender<GameEvent>,
    _event_sender: Sender<PauseMenuEvent>,
    event_receiver: Receiver<PauseMenuEvent>,
    ui: UIManager<PauseMenuEvent>,
}

impl PauseMenuSubclient {
    pub fn new(parent_channel: Sender<GameEvent>) -> PauseMenuSubclient {
        let (event_sender, event_receiver) = channel();
        let ui_sender = event_sender.clone();
        PauseMenuSubclient {
            parent_channel,
            scene: Box::new(MainPauseScreenSubclient::new(event_sender.clone())),
            _event_sender: event_sender,
            event_receiver,
            ui: UIManager::new(
                ui_sender,
                vec![Button::new(
                    ButtonBounds::absolute(Rect::new(15.0, 15.0, 30.0, 30.0)),
                    Text::new("X"),
                    PauseMenuEvent::GameEvent(GameEvent::ClosePauseMenu),
                )],
            ),
        }
    }

    pub fn handle_event(
        &mut self,
        _ctx: &mut Context,
        event: PauseMenuEvent,
    ) -> Result<(), GameError> {
        use PauseMenuEvent::*;
        match event {
            GameEvent(event) => self.parent_channel.send(event).unwrap(),
        }
        Ok(())
    }
}

impl SubEventHandler<GameError> for PauseMenuSubclient {
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
            .size(96.0)
            .centered_on(ctx, res * vec2(0.5, 0.1))?
            .color(Color::BLACK)
            .draw(canvas);

        self.ui.draw(ctx, canvas)?;
        self.scene.draw(ctx, canvas)
    }
}

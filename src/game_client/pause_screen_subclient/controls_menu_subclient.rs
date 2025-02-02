use ggez::{
    glam::{vec2, Vec2},
    graphics::{Canvas, DrawMode, Mesh, Rect, Text},
    input::keyboard::KeyCode,
    Context, GameError,
};
use std::sync::mpsc::{channel, Receiver, Sender};

use crate::{
    colors::PANEL_COLOR,
    sub_event_handler::SubEventHandler,
    ui_manager::{Button, ButtonBounds, UIManager},
    util::{AnchorPoint, DrawableWihParamsExt, TextExt},
};

use super::PauseScreenEvent;

#[derive(Clone)]
pub enum ControlsMenuEvent {
    PauseScreenEvent(PauseScreenEvent),
}

pub struct ControlsMenuSubclient {
    parent_channel: Sender<PauseScreenEvent>,
    event_sender: Sender<ControlsMenuEvent>,
    event_receiver: Receiver<ControlsMenuEvent>,
    ui: UIManager<ControlsMenuEvent, ControlsMenuEvent>,
}

impl ControlsMenuSubclient {
    pub fn new(parent_channel: Sender<PauseScreenEvent>) -> Self {
        let (event_sender, event_receiver) = channel();
        let ui_sender = event_sender.clone();
        Self {
            parent_channel,
            event_sender,
            event_receiver,
            ui: UIManager::new(
                ui_sender,
                [Button::new(
                    ButtonBounds::absolute(Rect::new(55.0, 20.0, 50.0, 30.0)),
                    Text::new("<").size(24.0),
                    ControlsMenuEvent::PauseScreenEvent(PauseScreenEvent::MainMenu),
                )],
            ),
        }
    }

    fn handle_event(
        &mut self,
        _ctx: &mut Context,
        event: ControlsMenuEvent,
    ) -> Result<(), GameError> {
        match event {
            ControlsMenuEvent::PauseScreenEvent(event) => self.parent_channel.send(event).unwrap(),
        }
        Ok(())
    }
}

impl SubEventHandler<GameError> for ControlsMenuSubclient {
    fn update(&mut self, ctx: &mut Context) -> Result<(), GameError> {
        self.ui.update(ctx)?;

        if ctx.keyboard.is_key_just_pressed(KeyCode::Escape) {
            self.event_sender
                .send(ControlsMenuEvent::PauseScreenEvent(
                    PauseScreenEvent::MainMenu,
                ))
                .unwrap();
        }

        while let Ok(event) = self.event_receiver.try_recv() {
            self.handle_event(ctx, event)?;
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context, canvas: &mut Canvas) -> Result<(), GameError> {
        let res: Vec2 = ctx.gfx.drawable_size().into();

        let panel_origin = vec2(100.0, 100.0);
        let panel = {
            let dims = res - vec2(200.0, 200.0);
            Rect::new(panel_origin.x, panel_origin.y, dims.x, dims.y)
        };

        Mesh::new_rectangle(ctx, DrawMode::fill(), panel, PANEL_COLOR)?.draw(canvas);

        Text::new("Controls")
            .size(56.0)
            .anchored_by(ctx, panel_origin + vec2(10.0, 10.0), AnchorPoint::NorthWest)?
            .draw(canvas);

        {
            let controls_text = "\
Left Mouse - Place tile or meeple
Right Mouse / WASD / Arrow keys - Move camera
Scroll - Zoom in and out
R - Rotate tile 90° clockwise
E - Rotate tile 90° counterclockwise
Enter - Skip meeples
Tab - Detailed game stats
Esc - Pause";
            let content_height = panel.h - 90.0;
            let font_size = 32.0f32.min(content_height / controls_text.lines().count() as f32);
            Text::new(controls_text)
                .size(font_size)
                .anchored_by(ctx, panel_origin + vec2(10.0, 80.0), AnchorPoint::NorthWest)?
                .draw(canvas);
        }

        self.ui.draw(ctx, canvas)?;

        Ok(())
    }
}

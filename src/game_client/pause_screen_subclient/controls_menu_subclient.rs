use ggez::{
    glam::{vec2, Vec2},
    graphics::{Canvas, Color, DrawMode, Mesh, Rect, Text},
    Context, GameError,
};
use log::trace;
use std::sync::mpsc::{channel, Receiver, Sender};

use crate::{
    colors::PANEL_COLOR,
    shared::Keybinds,
    shared::Shared,
    util::{AnchorPoint, DrawableWihParamsExt, RectExt, TextExt, Vec2ToRectExt},
};
use ggez_no_re::{
    sub_event_handler::SubEventHandler,
    ui_manager::{Bounds, Button, UIElement, UIManager},
};

use super::PauseScreenEvent;

#[derive(Clone, Debug)]
pub enum ControlsMenuEvent {
    PauseScreenEvent(PauseScreenEvent),
}

pub struct ControlsMenuSubclient {
    parent_channel: Sender<PauseScreenEvent>,
    event_sender: Sender<ControlsMenuEvent>,
    event_receiver: Receiver<ControlsMenuEvent>,
    ui: UIManager<ControlsMenuEvent, ControlsMenuEvent>,
    keybinds: Keybinds,
}

impl ControlsMenuSubclient {
    pub fn new(shared: Shared, parent_channel: Sender<PauseScreenEvent>) -> Self {
        let (event_sender, event_receiver) = channel();
        let ui_sender = event_sender.clone();
        let keybinds = shared.persistent.borrow().keybinds.clone();
        Self {
            parent_channel,
            event_sender,
            event_receiver,
            ui: UIManager::new(
                ui_sender,
                [UIElement::Button(Button::new(
                    Bounds::absolute(Rect::new(55.0, 20.0, 50.0, 30.0)),
                    Text::new("<").size(24.0),
                    ControlsMenuEvent::PauseScreenEvent(PauseScreenEvent::MainMenu),
                ))],
            ),
            keybinds,
        }
    }

    fn handle_event(
        &mut self,
        _ctx: &mut Context,
        event: ControlsMenuEvent,
    ) -> Result<(), GameError> {
        trace!("event = {event:?}");
        match event {
            ControlsMenuEvent::PauseScreenEvent(event) => self.parent_channel.send(event).unwrap(),
        }
        Ok(())
    }
}

impl SubEventHandler for ControlsMenuSubclient {
    fn update(&mut self, ctx: &mut Context) -> Result<(), GameError> {
        self.ui.update(ctx)?;

        if self.keybinds.pause.just_pressed(ctx) {
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
            Text::new(format!(
                "\
{} - Place tile
{} - Place meeple
{} / {}{}{}{} / {}{}{}{} - Move camera
Scroll / {} & {} - Zoom in and out
{} - Rotate tile 90° clockwise
{} - Rotate tile 90° counterclockwise
{} - Skip meeples
{} - Detailed game stats
{} - Pause",
                self.keybinds.place_tile,
                self.keybinds.place_meeple,
                self.keybinds.drag_camera,
                self.keybinds.move_up,
                self.keybinds.move_left,
                self.keybinds.move_down,
                self.keybinds.move_right,
                self.keybinds.move_up_alternate,
                self.keybinds.move_down_alternate,
                self.keybinds.move_left_alternate,
                self.keybinds.move_right_alternate,
                self.keybinds.zoom_in,
                self.keybinds.zoom_out,
                self.keybinds.rotate_clockwise,
                self.keybinds.rotate_counterclockwise,
                self.keybinds.skip_meeples,
                self.keybinds.detailed_view,
                self.keybinds.pause
            ))
            .draw_into_rect(
                ctx,
                canvas,
                Color::WHITE,
                32.0,
                (panel_origin + vec2(10.0, 80.0)).to(panel.bottom_right() - vec2(10.0, 10.0)),
            )?;
        }

        self.ui.draw(ctx, canvas)?;

        Ok(())
    }
}

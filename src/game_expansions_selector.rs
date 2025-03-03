use core::panic;
use std::{cell::RefCell, rc::Rc, sync::mpsc::channel};

use ggez::graphics::{Color, Rect, Text};
use ggez_no_re::{
    sub_event_handler::SubEventHandler,
    ui_manager::{Bounds, checkbox::Checkbox, UIElement, UIManager},
    util::{AnchorPoint, ContextExt, RectExt, TextExt},
};
use glam::vec2;

use crate::game_client::GameExpansions;

pub struct GameExpansionsSelector {
    _root: Bounds,
    ui: UIManager,
    rivers_1_checkbox: Rc<RefCell<Checkbox>>,
}

impl GameExpansionsSelector {
    pub fn new(root: Bounds) -> Self {
        let Bounds { relative, absolute } = root;
        let (_sender, _) = channel(); // dont like this :(
        let (ui, [UIElement::Checkbox(rivers_1_checkbox)]) = UIManager::new_and_rc_elements(
            _sender,
            [UIElement::Checkbox(Checkbox::new(Bounds {
                relative,
                absolute: Rect::new(absolute.x + 10.0, absolute.y + 10.0, 20.0, 20.0),
            }))],
        ) else {
            panic!()
        };
        GameExpansionsSelector {
            _root: root,
            ui,
            rivers_1_checkbox,
        }
    }

    pub fn get_selected_expansions(&self) -> GameExpansions {
        GameExpansions {
            rivers_1: self.rivers_1_checkbox.borrow().checked,
        }
    }
}

impl SubEventHandler for GameExpansionsSelector {
    fn update(&mut self, ctx: &mut ggez::Context) -> Result<(), ggez::GameError> {
        self.ui.update(ctx)
    }

    fn draw(
        &mut self,
        ctx: &mut ggez::Context,
        canvas: &mut ggez::graphics::Canvas,
    ) -> Result<(), ggez::GameError> {
        self.ui.draw(ctx, canvas)?;

        Text::new("River Expansion 1")
            .anchored_by(
                ctx,
                self.rivers_1_checkbox
                    .borrow()
                    .bounds
                    .corrected_bounds(ctx.res())
                    .parametric(vec2(1.0, 0.5))
                    + vec2(6.0, 0.0),
                AnchorPoint::CenterWest,
            )?
            .color(Color::BLACK)
            .draw(canvas);

        Ok(())
    }
}

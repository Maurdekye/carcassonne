use std::{cell::RefCell, rc::Rc, sync::mpsc::Sender};

use ggez::{
    event::MouseButton,
    glam::Vec2,
    graphics::{Canvas, Color, DrawMode, DrawParam, Mesh, Rect, Text},
    input::mouse::{set_cursor_type, CursorIcon},
    Context, GameError,
};

use crate::util::{color_mul, DrawableWihParamsExt};

pub const BUTTON_COLOR: Color = Color {
    r: 0.5,
    g: 0.5,
    b: 0.5,
    a: 1.0,
};

#[derive(Debug)]
pub struct ButtonBounds {
    pub relative: Rect,
    pub absolute: Rect,
}

impl ButtonBounds {
    #[allow(unused)]
    pub fn relative(bounds: Rect) -> ButtonBounds {
        ButtonBounds {
            relative: bounds,
            absolute: Rect::new(0.0, 0.0, 0.0, 0.0),
        }
    }

    #[allow(unused)]
    pub fn absolute(bounds: Rect) -> ButtonBounds {
        ButtonBounds {
            relative: Rect::new(0.0, 0.0, 0.0, 0.0),
            absolute: bounds,
        }
    }
}

#[derive(Debug)]
pub struct Button<E> {
    bounds: ButtonBounds,
    text: Text,
    body_color: Color,
    highlight_color: Color,
    depress_color: Color,
    text_drawparam: DrawParam,
    event: E,
    pub enabled: bool,
}

impl<E> Button<E> {
    pub fn new_with_styling(
        bounds: ButtonBounds,
        text: Text,
        text_drawparam: DrawParam,
        color: Color,
        event: E,
    ) -> Button<E> {
        Button {
            bounds,
            text,
            text_drawparam,
            body_color: color,
            highlight_color: color_mul(color, 1.2),
            depress_color: color_mul(color, 0.8),
            event,
            enabled: true,
        }
    }

    pub fn new(bounds: ButtonBounds, text: Text, event: E) -> Button<E> {
        Button::new_with_styling(bounds, text, DrawParam::default(), BUTTON_COLOR, event)
    }

    fn corrected_bounds(&self, res: Vec2) -> Rect {
        let ButtonBounds {
            relative: relative_bounds,
            absolute: absolute_bounds,
        } = self.bounds;
        Rect::new(
            relative_bounds.x * res.x + absolute_bounds.x,
            relative_bounds.y * res.y + absolute_bounds.y,
            relative_bounds.w * res.x + absolute_bounds.w,
            relative_bounds.h * res.y + absolute_bounds.h,
        )
    }
}

pub struct UIManager<E> {
    buttons: Vec<Rc<RefCell<Button<E>>>>,
    pub on_ui: bool,
    event_sender: Sender<E>,
    mouse_position: Vec2,
}

impl<E> UIManager<E> {
    pub fn new_and_rc_buttons<const N: usize>(
        event_sender: Sender<E>,
        buttons: [Button<E>; N],
    ) -> (UIManager<E>, [Rc<RefCell<Button<E>>>; N])
    where
        E: std::fmt::Debug,
    {
        let (buttons, return_buttons): (Vec<_>, Vec<_>) = buttons
            .into_iter()
            .map(RefCell::new)
            .map(Rc::new)
            .map(|button| (button.clone(), button))
            .unzip();
        (
            UIManager {
                buttons,
                on_ui: false,
                event_sender,
                mouse_position: Vec2::ZERO,
            },
            return_buttons.try_into().unwrap(),
        )
    }

    pub fn new<const N: usize>(event_sender: Sender<E>, buttons: [Button<E>; N]) -> UIManager<E> {
        UIManager {
            buttons: buttons.into_iter().map(RefCell::new).map(Rc::new).collect(),
            on_ui: false,
            event_sender,
            mouse_position: Vec2::ZERO,
        }
    }

    pub fn draw(&self, ctx: &Context, canvas: &mut Canvas) -> Result<(), GameError> {
        let res: Vec2 = ctx.gfx.drawable_size().into();
        for button in self
            .buttons
            .iter()
            .map(|button| button.borrow())
            .filter(|b| b.enabled)
        {
            let bounds = button.corrected_bounds(res);
            let contains = bounds.contains(self.mouse_position);
            let color = match (contains, ctx.mouse.button_pressed(MouseButton::Left)) {
                (true, true) => button.depress_color,
                (true, _) => button.highlight_color,
                _ => button.body_color,
            };
            Mesh::new_rounded_rectangle(ctx, DrawMode::fill(), bounds, 5.0, color)?.draw(canvas);
            button
                .text
                .with_params(button.text_drawparam)
                .centered_on(ctx, bounds.center().into())?
                .draw(canvas);
        }
        Ok(())
    }

    pub fn update(&mut self, ctx: &mut Context)
    where
        E: Clone,
    {
        let res: Vec2 = ctx.gfx.drawable_size().into();
        self.mouse_position = ctx.mouse.position().into();
        self.on_ui = false;
        for button in self
            .buttons
            .iter()
            .map(|button| button.borrow())
            .filter(|b| b.enabled)
        {
            let bounds = button.corrected_bounds(res);
            if bounds.contains(self.mouse_position) {
                self.on_ui = true;
                if ctx.mouse.button_just_released(MouseButton::Left) {
                    self.event_sender.send(button.event.clone()).unwrap();
                }
            }
        }
        if self.on_ui {
            set_cursor_type(ctx, CursorIcon::Hand);
        }
    }
}

use ggez::{
    event::MouseButton,
    glam::Vec2,
    graphics::{Canvas, Color, DrawMode, DrawParam, Mesh, Rect, Text},
    input::mouse::{set_cursor_type, CursorIcon},
    Context, GameError,
};

use crate::util::color_mul;

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
    pub fn new(
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
    pub buttons: Vec<Button<E>>,
    pub on_ui: bool,
}

impl<E> UIManager<E> {
    pub fn new(buttons: Vec<Button<E>>) -> UIManager<E> {
        UIManager {
            buttons,
            on_ui: false,
        }
    }

    pub fn draw(&self, ctx: &Context, canvas: &mut Canvas) -> Result<(), GameError> {
        let res: Vec2 = ctx.gfx.drawable_size().into();
        let mouse: Vec2 = ctx.mouse.position().into();
        for button in self.buttons.iter().filter(|b| b.enabled) {
            let bounds = button.corrected_bounds(res);
            let contains = bounds.contains(mouse);
            let color = match (contains, ctx.mouse.button_pressed(MouseButton::Left)) {
                (true, true) => button.depress_color,
                (true, _) => button.highlight_color,
                _ => button.body_color,
            };
            canvas.draw(
                &Mesh::new_rounded_rectangle(ctx, DrawMode::fill(), bounds, 5.0, color)?,
                DrawParam::default(),
            );
            let text_size = button.text.measure(ctx)?;
            let text_position = Vec2::from(bounds.center()) - Vec2::from(text_size) / 2.0;
            canvas.draw(&button.text, button.text_drawparam.dest(text_position));
        }
        Ok(())
    }

    pub fn update(&mut self, ctx: &mut Context) -> Vec<E>
    where
        E: Clone,
    {
        let res: Vec2 = ctx.gfx.drawable_size().into();
        let mouse: Vec2 = ctx.mouse.position().into();
        let mut events = Vec::new();
        self.on_ui = false;
        for button in self.buttons.iter().filter(|b| b.enabled) {
            let bounds = button.corrected_bounds(res);
            if bounds.contains(mouse) {
                self.on_ui = true;
                if ctx.mouse.button_just_released(MouseButton::Left) {
                    events.push(button.event.clone());
                }
            }
        }
        set_cursor_type(
            ctx,
            match self.on_ui {
                true => CursorIcon::Hand,
                false => CursorIcon::Arrow,
            },
        );
        events
    }
}

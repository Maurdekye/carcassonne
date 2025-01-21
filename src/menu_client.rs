use std::sync::mpsc::Sender;

use ggez::{
    event::EventHandler,
    glam::{vec2, Vec2},
    graphics::{Canvas, Color, DrawParam, Rect, Text},
    GameError,
};

use crate::{
    main_client::MainEvent,
    ui_manager::{Button, ButtonBounds, UIManager},
};

#[derive(Clone, Copy)]
enum MenuEvent {
    StartGame(usize),
}

pub struct MenuClient {
    parent_channel: Sender<MainEvent>,
    ui: UIManager<MenuEvent>,
}

impl MenuClient {
    pub fn new(parent_channel: Sender<MainEvent>) -> MenuClient {
        MenuClient {
            parent_channel,
            ui: UIManager::new(vec![
                Button::new(
                    ButtonBounds::relative(Rect::new(0.2, 0.5, 0.1, 0.045)),
                    Text::new("2 Players"),
                    Color::BLACK,
                    Color::from_rgb(127, 127, 127),
                    MenuEvent::StartGame(2),
                ),
                Button::new(
                    ButtonBounds::relative(Rect::new(0.35, 0.5, 0.1, 0.045)),
                    Text::new("3 Players"),
                    Color::BLACK,
                    Color::from_rgb(127, 127, 127),
                    MenuEvent::StartGame(3),
                ),
                Button::new(
                    ButtonBounds::relative(Rect::new(0.5, 0.5, 0.1, 0.045)),
                    Text::new("4 Players"),
                    Color::BLACK,
                    Color::from_rgb(127, 127, 127),
                    MenuEvent::StartGame(4),
                ),
                Button::new(
                    ButtonBounds::relative(Rect::new(0.65, 0.5, 0.1, 0.045)),
                    Text::new("5 Players"),
                    Color::BLACK,
                    Color::from_rgb(127, 127, 127),
                    MenuEvent::StartGame(5),
                ),
            ]),
        }
    }
}

impl EventHandler<GameError> for MenuClient {
    fn update(&mut self, ctx: &mut ggez::Context) -> Result<(), GameError> {
        // tick ui
        for event in self.ui.update(ctx) {
            match event {
                MenuEvent::StartGame(player_count) => self
                    .parent_channel
                    .send(MainEvent::StartGame(player_count))
                    .unwrap(),
            }
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut ggez::Context) -> Result<(), GameError> {
        let res: Vec2 = ctx.gfx.drawable_size().into();
        let mut canvas = Canvas::from_frame(ctx, Color::WHITE);

        // render title
        let mut menu_text = Text::new("Carcassone");
        menu_text.set_scale(144.0);
        let text_size: Vec2 = menu_text.measure(ctx)?.into();
        canvas.draw(
            &menu_text,
            DrawParam::from((res - text_size) * vec2(0.5, 0.2)).color(Color::BLACK),
        );

        // render ui
        self.ui.draw(ctx, &mut canvas)?;

        canvas.finish(ctx)
    }
}

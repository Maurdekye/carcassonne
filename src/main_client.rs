use std::sync::mpsc::{channel, Receiver, Sender};

use ggez::{event::EventHandler, Context, GameError};

use crate::{game_client::GameClient, menu_client::MenuClient};

#[derive(Clone)]
pub enum MainEvent {
    StartGame(usize),
    ReturnToMenu,
}

pub struct MainClient {
    scene: Box<dyn EventHandler<GameError>>,
    event_sender: Sender<MainEvent>,
    event_receiver: Receiver<MainEvent>,
}

impl MainClient {
    pub fn new() -> MainClient {
        let (event_sender, event_receiver) = channel();
        MainClient {
            scene: Box::new(MenuClient::new(event_sender.clone())),
            event_sender,
            event_receiver,
        }
    }

    fn handle_event(&mut self, ctx: &Context, event: MainEvent) {
        match event {
            MainEvent::StartGame(player_count) => {
                self.scene = Box::new(GameClient::new(
                    ctx,
                    player_count,
                    self.event_sender.clone(),
                ))
            }
            MainEvent::ReturnToMenu => {
                self.scene = Box::new(MenuClient::new(self.event_sender.clone()))
            }
        }
    }
}

impl EventHandler<GameError> for MainClient {
    fn mouse_wheel_event(&mut self, ctx: &mut Context, x: f32, y: f32) -> Result<(), GameError> {
        self.scene.mouse_wheel_event(ctx, x, y)
    }

    fn update(&mut self, ctx: &mut ggez::Context) -> Result<(), GameError> {
        self.scene.update(ctx)?;
        self.event_receiver
            .try_iter()
            .collect::<Vec<_>>()
            .into_iter()
            .for_each(|event| self.handle_event(ctx, event));

        Ok(())
    }

    fn draw(&mut self, ctx: &mut ggez::Context) -> Result<(), GameError> {
        ctx.gfx
            .set_window_title(&format!("Carcassone: {:.2} fps", ctx.time.fps()));

        self.scene.draw(ctx)
    }
}

use std::sync::mpsc::{channel, Receiver, Sender};

use ggez::{event::EventHandler, GameError};

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
}

impl EventHandler<GameError> for MainClient {
    fn update(&mut self, ctx: &mut ggez::Context) -> Result<(), GameError> {
        self.scene.update(ctx)?;

        while let Ok(event) = self.event_receiver.try_recv() {
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

        Ok(())
    }

    fn draw(&mut self, ctx: &mut ggez::Context) -> Result<(), GameError> {
        ctx.gfx
            .set_window_title(&format!("Carcassone: {:.2} fps", ctx.time.fps()));

        self.scene.draw(ctx)
    }
}

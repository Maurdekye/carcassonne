use std::sync::mpsc::{channel, Receiver, Sender};

use ggez::{
    event::EventHandler,
    input::{
        keyboard::KeyCode,
        mouse::{set_cursor_type, CursorIcon},
    },
    Context, GameError,
};

use crate::{game::Game, game_client::GameClient, main_menu_client::MainMenuClient, Args};

#[derive(Clone, Debug)]
pub enum MainEvent {
    StartGame(usize),
    StartDebugGame,
    ReturnToMainMenu,
    Close,
}

pub struct MainClient {
    scene: Box<dyn EventHandler<GameError>>,
    event_sender: Sender<MainEvent>,
    event_receiver: Receiver<MainEvent>,
    quitting: bool,
    args: Args,
}

impl MainClient {
    pub fn new(args: Args) -> MainClient {
        let (event_sender, event_receiver) = channel();
        if args.debug_game {
            event_sender.send(MainEvent::StartDebugGame).unwrap();
        }
        MainClient {
            scene: Box::new(MainMenuClient::new(event_sender.clone(), args.clone())),
            event_sender,
            event_receiver,
            quitting: false,
            args,
        }
    }

    fn handle_event(&mut self, ctx: &mut Context, event: MainEvent) -> Result<(), GameError> {
        match event {
            MainEvent::StartGame(player_count) => {
                self.scene = Box::new(GameClient::new(
                    ctx,
                    player_count,
                    self.event_sender.clone(),
                ))
            }
            MainEvent::ReturnToMainMenu => {
                self.scene = Box::new(MainMenuClient::new(
                    self.event_sender.clone(),
                    self.args.clone(),
                ))
            }
            MainEvent::Close => {
                self.quitting = true;
                ctx.request_quit();
            }
            MainEvent::StartDebugGame => {
                self.scene = Box::new(GameClient::new_with_game(
                    ctx,
                    Game::meeple_locations_debug_game(),
                    self.event_sender.clone(),
                ))
            }
        }
        Ok(())
    }
}

impl EventHandler<GameError> for MainClient {
    fn mouse_wheel_event(&mut self, ctx: &mut Context, x: f32, y: f32) -> Result<(), GameError> {
        self.scene.mouse_wheel_event(ctx, x, y)
    }

    fn update(&mut self, ctx: &mut Context) -> Result<(), GameError> {
        set_cursor_type(ctx, CursorIcon::Arrow);
        self.scene.update(ctx)?;
        while let Ok(event) = self.event_receiver.try_recv() {
            self.handle_event(ctx, event)?;
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> Result<(), GameError> {
        ctx.gfx
            .set_window_title(&format!("Carcassone: {:.2} fps", ctx.time.fps()));

        self.scene.draw(ctx)
    }

    fn quit_event(&mut self, ctx: &mut Context) -> Result<bool, GameError> {
        match (self.quitting, ctx.keyboard.is_key_pressed(KeyCode::Escape)) {
            (true, _) => Ok(false),
            (_, true) => Ok(true),
            _ => Ok(false),
        }
    }
}

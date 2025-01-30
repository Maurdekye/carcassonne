use std::{
    net::SocketAddr,
    sync::mpsc::{channel, Receiver, Sender},
};

use ggez::{
    event::EventHandler,
    graphics::{Canvas, Color},
    input::{
        keyboard::KeyCode,
        mouse::{set_cursor_type, CursorIcon},
    },
    Context, GameError,
};

use crate::{
    game_client::GameClient,
    main_menu_client::MainMenuClient,
    multiplayer::{
        multiplayer_host_menu_client::MultiplayerHostMenuClient,
        multiplayer_join_menu_client::MultiplayerJoinMenuClient,
    },
    sub_event_handler::SubEventHandler,
    Args, DebugGameConfiguration,
};

#[derive(Clone, Debug)]
pub enum MainEvent {
    StartGame(Vec<Color>),
    StartDebugGame(DebugGameConfiguration),
    ReturnToMainMenu,
    HostMultiplayerMenu,
    JoinMultiplayerMenu(SocketAddr),
    Close,
}

pub struct MainClient {
    scene: Box<dyn SubEventHandler<GameError>>,
    event_sender: Sender<MainEvent>,
    event_receiver: Receiver<MainEvent>,
    quitting: bool,
    args: Args,
}

impl MainClient {
    pub fn new(args: Args) -> MainClient {
        let (event_sender, event_receiver) = channel();
        if let Some(debug_config) = &args.debug_config {
            event_sender
                .send(MainEvent::StartDebugGame(debug_config.clone()))
                .unwrap();
        } else if let Some(ip) = args.ip {
            event_sender
                .send(MainEvent::JoinMultiplayerMenu(SocketAddr::new(
                    ip, args.port,
                )))
                .unwrap();
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
            MainEvent::StartGame(player_colors) => {
                self.scene = Box::new(GameClient::new(
                    ctx,
                    self.args.clone(),
                    player_colors,
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
            MainEvent::StartDebugGame(config) => {
                self.scene = Box::new(GameClient::new_with_game(
                    ctx,
                    self.args.clone(),
                    config.get_game()?,
                    self.event_sender.clone(),
                ))
            }
            MainEvent::HostMultiplayerMenu => {
                self.scene = Box::new(MultiplayerHostMenuClient::new(
                    self.event_sender.clone(),
                    self.args.clone(),
                ))
            }
            MainEvent::JoinMultiplayerMenu(socket) => {
                self.scene = Box::new(MultiplayerJoinMenuClient::new(
                    self.event_sender.clone(),
                    self.args.clone(),
                    socket,
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

        let mut canvas = Canvas::from_frame(ctx, Color::WHITE);

        self.scene.draw(ctx, &mut canvas)?;

        canvas.finish(ctx)
    }

    fn quit_event(&mut self, ctx: &mut Context) -> Result<bool, GameError> {
        match (self.quitting, ctx.keyboard.is_key_pressed(KeyCode::Escape)) {
            (true, _) => Ok(false),
            (_, true) => Ok(true),
            _ => Ok(false),
        }
    }
}

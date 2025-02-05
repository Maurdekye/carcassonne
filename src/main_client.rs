use std::{
    net::SocketAddr,
    path::PathBuf,
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
use log::{info, trace};

use crate::{
    game_client::GameClient,
    main_menu_client::MainMenuClient,
    multiplayer::{
        host_client::HostClient, join_client::JoinClient, multiplayer_menu::MultiplayerMenuClient,
    },
    sub_event_handler::SubEventHandler,
    DebugGameConfiguration, SharedResources,
};

#[derive(Clone, Debug)]
pub enum MainEvent {
    StartGame(Vec<Color>),
    LoadGame(PathBuf),
    StartDebugGame(DebugGameConfiguration),
    MainMenu,
    MultiplayerHost(u16),
    MultiplayerMenu,
    MultiplayerJoin(SocketAddr),
    Close,
}

pub struct MainClient {
    scene: Box<dyn SubEventHandler<GameError>>,
    event_sender: Sender<MainEvent>,
    event_receiver: Receiver<MainEvent>,
    quitting: bool,
    shared: SharedResources,
}

impl MainClient {
    pub fn new(shared: SharedResources) -> MainClient {
        let (event_sender, event_receiver) = channel();
        if let Some(load_path) = &shared.args.load {
            event_sender
                .send(MainEvent::LoadGame(load_path.clone()))
                .unwrap();
        } else if let Some(debug_config) = &shared.args.debug_config {
            event_sender
                .send(MainEvent::StartDebugGame(debug_config.clone()))
                .unwrap();
        }
        MainClient {
            scene: Box::new(MainMenuClient::new(event_sender.clone(), shared.clone())),
            event_sender,
            event_receiver,
            quitting: false,
            shared,
        }
    }

    fn handle_event(&mut self, ctx: &mut Context, event: MainEvent) -> Result<(), GameError> {
        trace!("event = {event:?}");
        match event {
            MainEvent::StartGame(player_colors) => {
                self.scene = Box::new(GameClient::new(
                    ctx,
                    self.shared.clone(),
                    player_colors,
                    self.event_sender.clone(),
                ))
            }
            MainEvent::MainMenu => {
                self.scene = Box::new(MainMenuClient::new(
                    self.event_sender.clone(),
                    self.shared.clone(),
                ))
            }
            MainEvent::Close => {
                self.quitting = true;
                ctx.request_quit();
            }
            MainEvent::LoadGame(path) => {
                self.scene = Box::new(GameClient::load(
                    ctx,
                    self.shared.clone(),
                    self.event_sender.clone(),
                    None,
                    path,
                )?);
            }
            MainEvent::StartDebugGame(config) => {
                self.scene = Box::new(GameClient::new_with_game(
                    ctx,
                    self.shared.clone(),
                    config.get_game()?,
                    self.event_sender.clone(),
                ));
            }
            MainEvent::MultiplayerHost(port) => {
                self.scene = Box::new(HostClient::new(
                    self.event_sender.clone(),
                    self.shared.clone(),
                    port,
                ));
            }
            MainEvent::MultiplayerJoin(socket) => {
                self.scene = Box::new(JoinClient::new(
                    self.event_sender.clone(),
                    self.shared.clone(),
                    socket,
                ));
            }
            MainEvent::MultiplayerMenu => {
                self.scene = Box::new(MultiplayerMenuClient::new(
                    self.event_sender.clone(),
                    self.shared.clone(),
                ));
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
        info!("Quitting");
        match (self.quitting, ctx.keyboard.is_key_pressed(KeyCode::Escape)) {
            (true, _) => Ok(false),
            (_, true) => Ok(true),
            _ => Ok(false),
        }
    }
}

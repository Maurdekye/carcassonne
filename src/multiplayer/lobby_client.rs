use ggez::{
    glam::vec2,
    graphics::{Color, Text},
    GameError, GameResult,
};

use crate::{
    game_client::GameClient,
    sub_event_handler::SubEventHandler,
    util::{AnchorPoint, TextExt},
};

use super::transport::message::{LobbyMessage, User};

pub struct LobbyClient {
    pub users: Vec<User>,
}

impl LobbyClient {
    pub fn new(users: Vec<User>) -> LobbyClient {
        LobbyClient { users }
    }

    pub fn handle_message(&mut self, message: LobbyMessage) -> Result<(), GameError> {
        match message {
            LobbyMessage::LobbyState(state) => {
                self.users = state.users;
            }
            _ => {}
        }

        Ok(())
    }
}

impl SubEventHandler<GameError> for LobbyClient {
    fn update(&mut self, ctx: &mut ggez::Context) -> Result<(), GameError> {
        Ok(())
    }

    fn draw(
        &mut self,
        ctx: &mut ggez::Context,
        canvas: &mut ggez::graphics::Canvas,
    ) -> Result<(), GameError> {
        let player_list_pos = vec2(50.0, 100.0);
        Text::new(format!("{} user(s) in lobby:", self.users.len()))
            .size(32.0)
            .anchored_by(ctx, player_list_pos, AnchorPoint::NorthWest)?
            .color(Color::BLACK)
            .draw(canvas);

        for (i, user) in self.users.iter().enumerate() {
            let text = if let Some(client_info) = &user.client_info {
                let mut text = format!("{}", client_info.ip);
                if let Some(latency) = client_info.latency {
                    text += &format!(" {}ms", latency.as_millis());
                }
                text
            } else {
                "Host".to_string()
            };
            let client_row_position = player_list_pos + vec2(0.0, 32.0) * (i + 1) as f32;
            Text::new(text)
                .size(32.0)
                .anchored_by(ctx, client_row_position, AnchorPoint::NorthWest)?
                .color(Color::BLACK)
                .draw(canvas);
            if let Some(color) = user.color {
                GameClient::draw_meeple(
                    ctx,
                    canvas,
                    client_row_position + vec2(16.0, -32.0),
                    color,
                    0.1,
                )?;
            }
        }

        Ok(())
    }
}

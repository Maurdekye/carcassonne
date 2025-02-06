use std::{cell::RefCell, net::IpAddr, rc::Rc, sync::mpsc::Sender};

use ggez::{
    glam::vec2,
    graphics::{Color, Rect, Text},
    GameError,
};
use log::trace;

use crate::{
    game_client::{GameClient, NUM_PLAYERS, PLAYER_COLORS},
    sub_event_handler::SubEventHandler,
    ui_manager::{Bounds, Button, UIElement, UIElementState, UIManager},
    util::{AnchorPoint, ContextExt, TextExt},
};

use super::transport::message::server::{LobbyMessage, User};

#[derive(Clone, Debug)]
pub enum LobbyEvent {
    ChooseColor(Option<Color>),
}

pub struct LobbyClient<T> {
    pub users: Vec<User>,
    me: Option<IpAddr>,
    _parent_channel: Sender<T>,
    color_choice_ui: UIManager<LobbyEvent, T>,
    color_choice_buttons: [Rc<RefCell<Button<LobbyEvent>>>; NUM_PLAYERS],
    ui: UIManager<LobbyEvent, T>,
    deselect_color_button: Rc<RefCell<Button<LobbyEvent>>>,
}

impl<T> LobbyClient<T>
where
    T: From<LobbyEvent>,
{
    pub fn new(users: Vec<User>, me: Option<IpAddr>, parent_channel: Sender<T>) -> LobbyClient<T> {
        let button_pos = Rect::new(0.6, 0.3, 0.0, 0.0);
        let mut i = 0;
        let (color_choice_ui, color_choice_buttons) = UIManager::new_and_rc_elements(
            parent_channel.clone(),
            PLAYER_COLORS.map(|color| {
                i += 1;
                UIElement::Button(Button::new(
                    Bounds {
                        relative: button_pos,
                        absolute: Rect::new((i - 1) as f32 * 40.0, 0.0, 30.0, 30.0),
                    },
                    Text::new(""),
                    LobbyEvent::ChooseColor(Some(color)),
                ))
            }),
        );
        let (ui, [UIElement::Button(deselect_color_button)]) = UIManager::new_and_rc_elements(
            parent_channel.clone(),
            [UIElement::Button(Button::new(
                Bounds {
                    relative: button_pos,
                    absolute: Rect::new(0.0, 40.0, 120.0, 40.0),
                },
                Text::new("Deselect"),
                LobbyEvent::ChooseColor(None),
            ))],
        ) else {
            panic!()
        };
        deselect_color_button.borrow_mut().state = UIElementState::Disabled;
        let color_choice_buttons = color_choice_buttons.map(UIElement::unwrap_button);
        LobbyClient {
            me,
            users,
            _parent_channel: parent_channel,
            color_choice_ui,
            color_choice_buttons,
            ui,
            deselect_color_button,
        }
    }

    pub fn handle_message(&mut self, message: LobbyMessage) -> Result<(), GameError> {
        trace!("message = {message:?}");
        #[allow(clippy::single_match)]
        match message {
            LobbyMessage::LobbyState(state) => {
                self.users = state.users;
                let selected_colors: Vec<_> =
                    self.users.iter().filter_map(|user| user.color).collect();
                for (color, button) in PLAYER_COLORS.iter().zip(self.color_choice_buttons.iter()) {
                    button.borrow_mut().state =
                        UIElementState::disabled_if(selected_colors.contains(color));
                }

                let me = self
                    .users
                    .iter()
                    .find(|user| user.ip() == self.me.as_ref())
                    .unwrap();
                self.deselect_color_button.borrow_mut().state =
                    UIElementState::disabled_if(me.color.is_none());
            }
        }

        Ok(())
    }
}

impl<T> SubEventHandler<GameError> for LobbyClient<T>
where
    T: From<LobbyEvent>,
{
    fn update(&mut self, ctx: &mut ggez::Context) -> Result<(), GameError> {
        self.color_choice_ui.update(ctx)?;
        self.ui.update(ctx)?;
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
                let mut text = user.username.clone();
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
                    client_row_position + vec2(-16.0, 16.0),
                    color,
                    0.1,
                )?;
            }
        }

        self.color_choice_ui.draw(ctx, canvas)?;
        for (color, button) in PLAYER_COLORS.iter().zip(self.color_choice_buttons.iter()) {
            let meeple_pos = button.borrow().corrected_bounds(ctx.res()).center().into();
            GameClient::draw_meeple(ctx, canvas, meeple_pos, *color, 0.1)?;
        }
        self.ui.draw(ctx, canvas)?;

        Ok(())
    }
}

use std::sync::mpsc::Sender;

use ggez_no_re::sub_event_handler::SubEventHandler;

use crate::{main_client::MainEvent, shared::{Keybinds, Shared}};

pub mod board_viewable_handler {
    use ggez::{graphics::Canvas, Context, GameResult};
    use ggez_no_re::sub_event_handler::SubEventHandler;

    pub trait BoardViewableEventHandler: SubEventHandler {
        fn draw_to_board(&mut self, ctx: &mut Context, canvas: &mut Canvas, origin_square: Rect) -> GameResult<()>;
    }
}


pub struct BoardView {
    parent_channel: Sender<MainEvent>,
    shared: Shared,
    keybinds: Keybinds,
    viewable:,
    pause_menu: Option<PauseScreenSubclient>,
    offset: Vec2,
    scale: f32,
    camera_movement: Vec2,
    camera_zoom: f32,
}

impl BoardView {

}

impl SubEventHandler for BoardView {

}
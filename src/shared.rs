use std::net::IpAddr;

use ggez::winit::{event::MouseButton, keyboard::NamedKey};
use ggez_no_re::{keybinds, persist::PersistenceManager};
use serde::{Deserialize, Serialize};

use crate::Args;

#[derive(Clone, Debug)]
pub struct SharedResources {
    pub args: Args,
    pub persistent: PersistenceManager<SaveData>,
}

impl SharedResources {
    pub fn new(args: Args) -> SharedResources {
        let persistent = PersistenceManager::new(&args.save_path);
        SharedResources { args, persistent }
    }
}

keybinds! {
    struct Keybinds {
        place_tile: MouseButton::Left,
        place_meeple: MouseButton::Left,
        rotate_clockwise: "r",
        rotate_counterclockwise: "e",
        pause: NamedKey::Escape,
        quit: NamedKey::Escape,
        skip_meeples: NamedKey::Space,
        detailed_view: NamedKey::Tab,
        drag_camera: MouseButton::Right,
        zoom_in: "-",
        zoom_out: "=",
        move_up: "w",
        move_up_alternate: NamedKey::ArrowUp,
        move_right: "d",
        move_right_alternate: NamedKey::ArrowRight,
        move_down: "s",
        move_down_alternate: NamedKey::ArrowDown,
        move_left: "a",
        move_left_alternate: NamedKey::ArrowLeft,
        move_faster: NamedKey::Shift,
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SaveData {
    pub ip: Option<IpAddr>,
    pub port: u16,
    pub username: String,
    pub keybinds: Keybinds,
}

impl Default for SaveData {
    fn default() -> Self {
        Self {
            ip: None,
            port: 11069,
            username: String::new(),
            keybinds: Keybinds::default(),
        }
    }
}

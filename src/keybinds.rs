use std::fmt::Display;

use ggez::{
    winit::{
        event::MouseButton,
        keyboard::{Key, NamedKey},
    },
    Context,
};
use serde::{Deserialize, Serialize};

macro_rules! define_keybinds {
    ($($name:ident: $key:expr,)*) => {
        #[derive(Clone, Debug, Serialize, Deserialize)]
        pub struct Keybinds {
            $(pub $name: Keybind,)*
        }

        impl Default for Keybinds {
            fn default() -> Keybinds {
                Keybinds {
                    $($name: $key.into(),)*
                }
            }
        }
    };
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Keybind {
    Mouse(MouseButton),
    Key(Key),
}

impl Keybind {
    pub fn pressed(&self, ctx: &Context) -> bool {
        match self {
            Keybind::Mouse(mouse_button) => ctx.mouse.button_pressed(*mouse_button),
            Keybind::Key(key) => ctx.keyboard.is_logical_key_pressed(key),
        }
    }

    pub fn just_pressed(&self, ctx: &Context) -> bool {
        match self {
            Keybind::Mouse(mouse_button) => ctx.mouse.button_just_pressed(*mouse_button),
            Keybind::Key(key) => ctx.keyboard.is_logical_key_just_pressed(key),
        }
    }

    #[allow(unused)]
    pub fn just_released(&self, ctx: &Context) -> bool {
        match self {
            Keybind::Mouse(mouse_button) => ctx.mouse.button_just_released(*mouse_button),
            Keybind::Key(key) => ctx.keyboard.is_logical_key_just_released(key),
        }
    }
}

impl From<MouseButton> for Keybind {
    fn from(value: MouseButton) -> Self {
        Keybind::Mouse(value)
    }
}

impl From<&str> for Keybind {
    fn from(value: &str) -> Self {
        Keybind::Key(Key::Character(value.into()))
    }
}

impl From<NamedKey> for Keybind {
    fn from(value: NamedKey) -> Self {
        Keybind::Key(Key::Named(value))
    }
}

impl Display for Keybind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Keybind::Mouse(mouse_button) => write!(f, "{:?} Mouse", mouse_button),
            Keybind::Key(key) => match key {
                Key::Named(named_key) => match named_key {
                    NamedKey::ArrowUp => write!(f, "↑"),
                    NamedKey::ArrowDown => write!(f, "↓"),
                    NamedKey::ArrowLeft => write!(f, "←"),
                    NamedKey::ArrowRight => write!(f, "→"),
                    named_key => write!(f, "{named_key:?}"),
                },
                Key::Character(ch) => write!(f, "{}", ch.to_uppercase()),
                key => match key.to_text() {
                    Some(text) => write!(f, "{}", text.to_uppercase()),
                    None => write!(f, "{key:?}"),
                },
            },
        }
    }
}

define_keybinds! {
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

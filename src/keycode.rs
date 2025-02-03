use ggez::input::keyboard::KeyCode;

pub trait KeyCodeExt {
    fn chr(self) -> Option<char>;
}

// thank you copilot 🙏
impl KeyCodeExt for KeyCode {
    fn chr(self) -> Option<char> {
        match self {
            KeyCode::Key1 => Some('1'),
            KeyCode::Key2 => Some('2'),
            KeyCode::Key3 => Some('3'),
            KeyCode::Key4 => Some('4'),
            KeyCode::Key5 => Some('5'),
            KeyCode::Key6 => Some('6'),
            KeyCode::Key7 => Some('7'),
            KeyCode::Key8 => Some('8'),
            KeyCode::Key9 => Some('9'),
            KeyCode::Key0 => Some('0'),
            KeyCode::A => Some('a'),
            KeyCode::B => Some('b'),
            KeyCode::C => Some('c'),
            KeyCode::D => Some('d'),
            KeyCode::E => Some('e'),
            KeyCode::F => Some('f'),
            KeyCode::G => Some('g'),
            KeyCode::H => Some('h'),
            KeyCode::I => Some('i'),
            KeyCode::J => Some('j'),
            KeyCode::K => Some('k'),
            KeyCode::L => Some('l'),
            KeyCode::M => Some('m'),
            KeyCode::N => Some('n'),
            KeyCode::O => Some('o'),
            KeyCode::P => Some('p'),
            KeyCode::Q => Some('q'),
            KeyCode::R => Some('r'),
            KeyCode::S => Some('s'),
            KeyCode::T => Some('t'),
            KeyCode::U => Some('u'),
            KeyCode::V => Some('v'),
            KeyCode::W => Some('w'),
            KeyCode::X => Some('x'),
            KeyCode::Y => Some('y'),
            KeyCode::Z => Some('z'),
            KeyCode::Space => Some(' '),
            KeyCode::Apostrophe => Some('\''),
            KeyCode::Comma => Some(','),
            KeyCode::Minus => Some('-'),
            KeyCode::Period => Some('.'),
            KeyCode::Slash => Some('/'),
            KeyCode::Semicolon => Some(';'),
            KeyCode::Equals => Some('='),
            KeyCode::LBracket => Some('['),
            KeyCode::Backslash => Some('\\'),
            KeyCode::RBracket => Some(']'),
            KeyCode::Grave => Some('`'),
            _ => None,
        }
    }
}

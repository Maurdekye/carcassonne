use std::ops::Add;

use ggez::glam::{vec2, Vec2};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct GridPos(pub i32, pub i32);

impl Add<GridPos> for GridPos {
    type Output = GridPos;

    fn add(self, rhs: GridPos) -> Self::Output {
        GridPos(self.0 + rhs.0, self.1 + rhs.1)
    }
}

impl From<GridPos> for Vec2 {
    fn from(GridPos(x, y): GridPos) -> Self {
        vec2(x as f32, y as f32)
    }
}

impl From<Vec2> for GridPos {
    fn from(Vec2 { x, y }: Vec2) -> Self {
        GridPos(x as i32, y as i32)
    }
}

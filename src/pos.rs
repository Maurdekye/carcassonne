use std::ops::Add;

use ggez::glam::{vec2, Vec2};
use serde::{Deserialize, Serialize};

const ADJACENT_OFFSETS: [GridPos; 4] =
    [GridPos(-1, 0), GridPos(0, -1), GridPos(1, 0), GridPos(0, 1)];

const SURROUNDING_OFFSETS: [GridPos; 8] = [
    GridPos(-1, -1),
    GridPos(-1, 0),
    GridPos(-1, 1),
    GridPos(0, 1),
    GridPos(1, 1),
    GridPos(1, 0),
    GridPos(1, -1),
    GridPos(0, -1),
];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct GridPos(pub i32, pub i32);

impl GridPos {
    pub fn adjacent(self) -> impl Iterator<Item = Self> {
        ADJACENT_OFFSETS.iter().map(move |&offset| self + offset)
    }

    pub fn surrounding(self) -> impl Iterator<Item = Self> {
        SURROUNDING_OFFSETS.iter().map(move |&offset| self + offset)
    }
}

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
        GridPos(x.floor() as i32, y.floor() as i32)
    }
}

impl From<GridPos> for (i32, i32) {
    fn from(GridPos(x, y): GridPos) -> Self {
        (x, y)
    }
}

impl From<(i32, i32)> for GridPos {
    fn from((x, y): (i32, i32)) -> Self {
        GridPos(x, y)
    }
}
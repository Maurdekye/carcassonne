use std::ops::Add;

use ggez::{graphics::Rect, mint::Point2, Context};

use crate::GRID_SIZE;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct GridPos(pub i32, pub i32);

impl GridPos {
    pub fn rect(&self, ctx: &Context) -> Rect {
        let (width, height) = ctx.gfx.drawable_size();
        let width = width * GRID_SIZE;
        let height = height * GRID_SIZE;
        let near_corner = self.to_screen_pos(ctx);
        Rect::new(near_corner.x, near_corner.y, width, height)
    }

    pub fn from_screen_pos(screen_pos: Point2<f32>, ctx: &Context) -> GridPos {
        let (width, height) = ctx.gfx.drawable_size();
        let uv = Point2 {
            x: screen_pos.x / width,
            y: screen_pos.y / height,
        };
        GridPos((uv.x / GRID_SIZE) as i32, (uv.y / GRID_SIZE) as i32)
    }

    pub fn to_screen_pos(self, ctx: &Context) -> Point2<f32> {
        let res = ctx.gfx.window().inner_size();
        Point2 {
            x: (self.0 as f32 * GRID_SIZE) * res.width as f32,
            y: (self.1 as f32 * GRID_SIZE) * res.height as f32,
        }
    }
}

impl Add<GridPos> for GridPos {
    type Output = GridPos;

    fn add(self, rhs: GridPos) -> Self::Output {
        GridPos(self.0 + rhs.0, self.1 + rhs.1)
    }
}

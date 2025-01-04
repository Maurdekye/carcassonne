use ggez::{
    glam::{vec2, Vec2},
    graphics::Rect,
};

pub fn refit_to_rect(vec: Vec2, rect: Rect) -> Vec2 {
    vec2(vec.x * rect.w + rect.x, vec.y * rect.h + rect.y)
}

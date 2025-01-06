use ggez::{
    glam::{vec2, Vec2},
    graphics::Rect,
};

pub fn refit_to_rect(vec: Vec2, rect: Rect) -> Vec2 {
    vec2(vec.x * rect.w + rect.x, vec.y * rect.h + rect.y)
}

pub fn point_in_polygon(point: Vec2, polygon: &[Vec2]) -> bool {
    let mut crossings = 0;
    for (i, j) in (0..polygon.len()).zip((1..polygon.len()).chain([0])) {
        let a = polygon[i];
        let b = polygon[j];
        if (a.y > point.y) != (b.y > point.y) {
            let slope = (b.x - a.x) / (b.y - a.y);
            let x_intersect = slope * (point.y - a.y) + a.x;

            if point.x < x_intersect {
                crossings += 1;
            }
        }
    }
    crossings % 2 == 1
}

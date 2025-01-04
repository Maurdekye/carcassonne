mod tile_definitions;

use ggez::{
    glam::{vec2, Vec2},
    graphics::{Canvas, Color, DrawMode, DrawParam, Mesh, Rect},
    Context, GameError,
};
use tile_definitions::{L_CURVE_ROAD, STRAIGHT_ROAD};

use crate::util::refit_to_rect;

#[derive(Clone)]
enum SegmentType {
    Field,
    City,
    Road,
}

impl SegmentType {
    fn color(&self) -> Color {
        match self {
            SegmentType::Field => Color::from_rgb(171, 219, 59),
            SegmentType::City => Color::from_rgb(222, 133, 38),
            SegmentType::Road => Color::from_rgb(207, 194, 149),
        }
    }
}

#[derive(Clone)]
enum Mount {
    Beginning,
    Middle,
    End,
    BegginingAndEnd,
    Full,
}

#[derive(Clone)]
struct Mounts {
    north: Option<Mount>,
    east: Option<Mount>,
    south: Option<Mount>,
    west: Option<Mount>,
}

impl Mounts {
    fn rotate(self) -> Mounts {
        let Mounts {
            north,
            east,
            south,
            west,
        } = self;
        Mounts {
            north: west,
            east: north,
            south: east,
            west: south,
        }
    }
}

#[derive(Clone)]
struct Segment {
    stype: SegmentType,
    poly: Vec<usize>,
    mounts: Mounts,
}

#[derive(Clone)]
pub struct Tile {
    verts: Vec<Vec2>,
    segments: Vec<Segment>,
}

impl Tile {
    pub fn render(
        &self,
        ctx: &Context,
        canvas: &mut Canvas,
        bounds: Rect,
    ) -> Result<(), GameError> {
        for segment in &self.segments {
            let verts: Vec<Vec2> = segment
                .poly
                .iter()
                .map(|i| refit_to_rect(self.verts[*i], bounds))
                .collect();
            canvas.draw(
                &Mesh::new_polygon(ctx, DrawMode::fill(), &verts, segment.stype.color())?,
                DrawParam::default(),
            );
        }
        Ok(())
    }

    pub fn rotate(&mut self) {
        for vert in &mut self.verts {
            *vert = vec2(1.0 - vert.y, vert.x);
        }
        
        for segment in &mut self.segments {
            segment.mounts = segment.mounts.clone().rotate();
        }
    }
}

pub fn get_tile_library() -> Vec<Tile> {
    vec![
        STRAIGHT_ROAD.clone(),
        L_CURVE_ROAD.clone(),
        STRAIGHT_ROAD.clone(),
        L_CURVE_ROAD.clone(),
        STRAIGHT_ROAD.clone(),
        L_CURVE_ROAD.clone(),
    ]
}

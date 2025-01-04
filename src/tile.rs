use ggez::{
    glam::{vec2, Vec2},
    graphics::{Canvas, Color, DrawMode, DrawParam, Mesh, Rect},
    Context, GameError,
};

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
struct Poly(Vec<usize>);

#[derive(Clone)]
struct Mounts {
    north: Option<Mount>,
    east: Option<Mount>,
    south: Option<Mount>,
    west: Option<Mount>,
}

#[derive(Clone)]
struct Segment {
    stype: SegmentType,
    poly_ids: Vec<usize>,
    mounts: Mounts,
}

#[derive(Clone)]
pub struct Tile {
    verts: Vec<Vec2>,
    polys: Vec<Poly>,
    segments: Vec<Segment>,
}

impl Tile {
    pub fn render(&self, ctx: &Context, canvas: &mut Canvas, bounds: Rect) -> Result<(), GameError> {
        for segment in &self.segments {
            for pid in &segment.poly_ids {
                let verts: Vec<Vec2> = self.polys[*pid]
                    .0
                    .iter()
                    .map(|i| refit_to_rect(self.verts[*i], bounds))
                    .collect();
                canvas.draw(
                    &Mesh::new_polygon(ctx, DrawMode::fill(), &verts, segment.stype.color())?,
                    DrawParam::default(),
                );
            }
        }
        Ok(())
    }
}

pub fn get_tile_library() -> Vec<Tile> {
    vec![Tile {
        verts: vec![
            vec2(0.0, 0.0),
            vec2(0.0, 1.0),
            vec2(1.0, 1.0),
            vec2(1.0, 0.0),
            vec2(0.0, 0.45),
            vec2(0.0, 0.55),
            vec2(1.0, 0.45),
            vec2(1.0, 0.55),
        ],
        polys: vec![
            Poly(vec![0, 3, 6, 4]),
            Poly(vec![4, 6, 7, 5]),
            Poly(vec![5, 7, 2, 1]),
        ],
        segments: vec![
            Segment {
                stype: SegmentType::Field,
                poly_ids: vec![0],
                mounts: Mounts {
                    north: Some(Mount::Full),
                    east: Some(Mount::Beginning),
                    south: None,
                    west: Some(Mount::End),
                },
            },
            Segment {
                stype: SegmentType::Road,
                poly_ids: vec![1],
                mounts: Mounts {
                    north: None,
                    east: Some(Mount::Middle),
                    south: None,
                    west: Some(Mount::Middle),
                },
            },
            Segment {
                stype: SegmentType::Field,
                poly_ids: vec![2],
                mounts: Mounts {
                    north: None,
                    east: Some(Mount::End),
                    south: Some(Mount::Full),
                    west: Some(Mount::Beginning),
                },
            },
        ],
    }]
}

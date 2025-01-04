pub mod tile_definitions;

use ggez::{
    glam::{vec2, Vec2},
    graphics::{Canvas, Color, DrawMode, DrawParam, Mesh, Rect},
    Context, GameError,
};
use tile_definitions::{L_CURVE_ROAD, STRAIGHT_ROAD};

use crate::util::refit_to_rect;

#[derive(Clone, Copy)]
pub enum Orientation {
    North,
    East,
    South,
    West,
}

impl Orientation {
    fn opposite(self) -> Orientation {
        use Orientation::*;
        match self {
            North => South,
            East => West,
            South => North,
            West => East,
        }
    }
}

pub struct MountingPair {
    from_segment: usize,
    to_segment: usize,
}

#[derive(Clone, PartialEq, Eq)]
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

enum MountAlignment {
    Disjoint,
    Misaligned,
    Aligned,
}

impl Mount {
    fn compare(&self, rhs: &Mount) -> MountAlignment {
        use Mount::*;
        use MountAlignment::*;
        match (self, rhs) {
            (Beginning, End) => Aligned,
            (Middle, Middle) => Aligned,
            (End, Beginning) => Aligned,
            (Full, Full) => Aligned,
            (Beginning, BegginingAndEnd) => Aligned,
            (Middle, BegginingAndEnd) => Disjoint,
            (End, BegginingAndEnd) => Aligned,
            (BegginingAndEnd, Beginning) => Aligned,
            (BegginingAndEnd, Middle) => Disjoint,
            (BegginingAndEnd, End) => Aligned,
            (_, Full) => Misaligned,
            (Full, _) => Misaligned,
            _ => Disjoint,
        }
    }
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

    fn by_orientation(&self, orientation: Orientation) -> &Option<Mount> {
        use Orientation::*;
        match orientation {
            North => &self.north,
            East => &self.east,
            South => &self.south,
            West => &self.west,
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
            // *vert = vec2(vert.y, 1.0 - vert.x);
        }

        for segment in &mut self.segments {
            segment.mounts = segment.mounts.clone().rotate();
        }
    }

    pub fn validate_mounting(
        &self,
        adjacent: &Tile,
        location: Orientation,
    ) -> Option<Vec<MountingPair>> {
        let mut pairs = Vec::new();
        for (i, segment) in self.segments.iter().enumerate() {
            if let Some(mount) = &segment.mounts.by_orientation(location) {
                for (j, adj_segment) in adjacent.segments.iter().enumerate() {
                    if let Some(adj_mount) = &adj_segment.mounts.by_orientation(location.opposite())
                    {
                        match mount.compare(adj_mount) {
                            MountAlignment::Disjoint => {}
                            MountAlignment::Misaligned => return None,
                            MountAlignment::Aligned => {
                                if segment.stype == adj_segment.stype {
                                    pairs.push(MountingPair {
                                        from_segment: i,
                                        to_segment: j,
                                    });
                                } else {
                                    return None;
                                }
                            }
                        }
                    }
                }
            }
        }
        Some(pairs)
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

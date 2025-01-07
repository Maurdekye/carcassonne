pub mod tile_definitions;

use ggez::{
    glam::{vec2, Vec2},
    graphics::{Canvas, Color, DrawMode, DrawParam, Mesh, Rect},
    Context, GameError,
};
use tile_definitions::{CITY_ENTRANCE, CORNER_CITY, DEAD_END_ROAD, L_CURVE_ROAD, STRAIGHT_ROAD};

use crate::{pos::Pos, util::refit_to_rect};

#[cfg(test)]
mod test;

const MOUNTS_PER_SIDE: usize = 3;

pub type Mount = [usize; MOUNTS_PER_SIDE];

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum Orientation {
    North,
    East,
    South,
    West,
}

impl Orientation {
    pub fn opposite(self) -> Orientation {
        use Orientation::*;
        match self {
            North => South,
            East => West,
            South => North,
            West => East,
        }
    }

    pub fn iter_with_offsets() -> impl Iterator<Item = (Orientation, Pos)> {
        use Orientation::*;
        [North, East, South, West]
            .into_iter()
            .zip([Pos(0, -1), Pos(1, 0), Pos(0, 1), Pos(-1, 0)])
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct MountingPair {
    pub from_segment: usize,
    pub to_segment: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SegmentType {
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

#[derive(Clone, Debug)]
pub struct Mounts {
    north: Mount,
    east: Mount,
    south: Mount,
    west: Mount,
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

    pub fn by_orientation(&self, orientation: Orientation) -> &Mount {
        use Orientation::*;
        match orientation {
            North => &self.north,
            East => &self.east,
            South => &self.south,
            West => &self.west,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Segment {
    pub stype: SegmentType,
    pub poly: Vec<usize>,
}

#[derive(Clone, Debug)]
pub struct Tile {
    pub verts: Vec<Vec2>,
    pub segments: Vec<Segment>,
    pub mounts: Mounts,
}

impl Tile {
    pub fn render(
        &self,
        ctx: &Context,
        canvas: &mut Canvas,
        bounds: Rect,
    ) -> Result<(), GameError> {
        for i in 0..self.segments.len() {
            self.render_segment(i, ctx, canvas, bounds, None)?;
        }
        Ok(())
    }

    pub fn render_segment(
        &self,
        seg_index: usize,
        ctx: &Context,
        canvas: &mut Canvas,
        bounds: Rect,
        color: Option<Color>,
    ) -> Result<(), GameError> {
        let segment = &self.segments[seg_index];
        let verts: Vec<Vec2> = segment
            .poly
            .iter()
            .map(|i| refit_to_rect(self.verts[*i], bounds))
            .collect();
        canvas.draw(
            &Mesh::new_polygon(
                ctx,
                DrawMode::fill(),
                &verts,
                color.unwrap_or_else(|| segment.stype.color()),
            )?,
            DrawParam::default(),
        );
        Ok(())
    }

    pub fn rotate(&mut self) {
        for vert in &mut self.verts {
            *vert = vec2(1.0 - vert.y, vert.x);
            // *vert = vec2(vert.y, 1.0 - vert.x);
        }

        self.mounts = self.mounts.clone().rotate();
    }

    pub fn rotated(mut self) -> Self {
        self.rotate();
        self
    }

    pub fn validate_mounting(
        &self,
        adjacent: &Tile,
        location: Orientation,
    ) -> Option<Vec<MountingPair>> {
        let mut pairs = Vec::new();
        for (seg_index, adj_seg_index) in self.mounts.by_orientation(location).iter().cloned().zip(
            adjacent
                .mounts
                .by_orientation(location.opposite())
                .iter()
                .rev()
                .cloned(),
        ) {
            let segment = &self.segments[seg_index];
            let adj_segment = &adjacent.segments[adj_seg_index];
            if segment.stype == adj_segment.stype {
                let pair = MountingPair {
                    from_segment: seg_index,
                    to_segment: adj_seg_index,
                };
                if !pairs.contains(&pair) {
                    pairs.push(pair);
                }
            } else {
                return None;
            }
        }
        Some(pairs)
    }
}

pub fn get_tile_library() -> Vec<Tile> {
    vec![
        CITY_ENTRANCE.clone(),
        STRAIGHT_ROAD.clone(),
        CORNER_CITY.clone(),
        L_CURVE_ROAD.clone(),
        CITY_ENTRANCE.clone(),
        STRAIGHT_ROAD.clone(),
        CORNER_CITY.clone(),
        L_CURVE_ROAD.clone(),
        DEAD_END_ROAD.clone(),
        STRAIGHT_ROAD.clone(),
    ]
}

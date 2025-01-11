pub mod tile_definitions;

use std::collections::HashMap;

use ggez::{
    glam::{vec2, Vec2},
    graphics::{Canvas, Color, DrawMode, DrawParam, Mesh, Rect},
    Context, GameError,
};
use tile_definitions::{CITY_ENTRANCE, CORNER_CITY, L_CURVE_ROAD, STRAIGHT_ROAD};

use crate::{game::SegmentIndex, pos::GridPos, util::refit_to_rect};

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

    pub fn iter_with_offsets() -> impl Iterator<Item = (Orientation, GridPos)> {
        use Orientation::*;
        [North, East, South, West].into_iter().zip([
            GridPos(0, -1),
            GridPos(1, 0),
            GridPos(0, 1),
            GridPos(-1, 0),
        ])
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
    Monastary,
    Village,
}

impl SegmentType {
    fn color(&self) -> Color {
        use SegmentType::*;
        match self {
            Field => Color::from_rgb(171, 219, 59),
            City => Color::from_rgb(222, 133, 38),
            Road => Color::from_rgb(207, 194, 149),
            Monastary => Color::from_rgb(183, 222, 235),
            Village => Color::from_rgb(227, 204, 166),
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

    pub fn by_orientation_mut(&mut self, orientation: Orientation) -> &mut Mount {
        use Orientation::*;
        match orientation {
            North => &mut self.north,
            East => &mut self.east,
            South => &mut self.south,
            West => &mut self.west,
        }
    }
}

#[derive(Clone, Debug)]
pub enum SegmentAttribute {
    Fortified,
    CustomMeepleSpot(Vec2),
}

#[derive(Clone, Debug)]
pub struct Segment {
    pub stype: SegmentType,
    pub poly: Vec<usize>,
    pub attributes: Vec<SegmentAttribute>,
    pub meeple_spot: Vec2,
}

impl Segment {
    pub fn new(
        stype: SegmentType,
        poly: Vec<usize>,
        attributes: Vec<SegmentAttribute>,
        meeple_spot: Vec2,
    ) -> Self {
        Self {
            stype,
            poly,
            attributes,
            meeple_spot,
        }
    }
}

#[derive(Clone, Debug)]
pub enum TileAttribute {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, std::hash::Hash)]
pub enum SegmentEdgePortion {
    Beginning,
    Middle,
    End,
    Full,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, std::hash::Hash)]
pub enum SegmentBorderPiece {
    Edge(SegmentEdgePortion, Orientation),
    Vert(usize),
}

#[derive(Debug)]
pub enum SegmentDefinition {
    Segment {
        stype: SegmentType,
        edges: Vec<SegmentBorderPiece>,
    },
    SpecialSegment {
        stype: SegmentType,
        attributes: Vec<SegmentAttribute>,
        edges: Vec<SegmentBorderPiece>,
    },
}

#[derive(Clone, Debug)]
pub struct Tile {
    pub verts: Vec<Vec2>,
    pub segments: Vec<Segment>,
    pub mounts: Mounts,
    segment_adjacency: Vec<bool>,
    pub attributes: Vec<TileAttribute>,
}

impl Tile {
    pub fn new_with_attributes(
        mut verts: Vec<Vec2>,
        segment_definitions: Vec<SegmentDefinition>,
        attributes: Vec<TileAttribute>,
    ) -> Self {
        fn edges_contiguous(
            before: (SegmentEdgePortion, Orientation),
            after: (SegmentEdgePortion, Orientation),
        ) -> bool {
            use Orientation::*;
            use SegmentEdgePortion::*;
            match (before, after) {
                ((End | Full, prev_orientation), (Beginning | Full, orientation)) => {
                    matches!(
                        (prev_orientation, orientation),
                        (West, North) | (North, East) | (East, South) | (South, West)
                    )
                }
                ((Beginning, prev_orientation), (Middle, orientation))
                | ((Middle, prev_orientation), (End, orientation))
                    if prev_orientation == orientation =>
                {
                    true
                }
                _ => false,
            }
        }

        let mut edge_verts_map: HashMap<(SegmentEdgePortion, Orientation), [usize; 2]> =
            HashMap::new();
        let mut segments: Vec<Segment> = Vec::new();
        let mut mounts: Mounts = Mounts {
            north: [0, 0, 0],
            east: [0, 0, 0],
            south: [0, 0, 0],
            west: [0, 0, 0],
        };

        for (i, segment_definition) in segment_definitions.into_iter().enumerate() {
            let mut poly = Vec::new();
            let (stype, edges, attributes) = match segment_definition {
                SegmentDefinition::Segment { stype, edges } => (stype, edges, Vec::new()),
                SegmentDefinition::SpecialSegment {
                    stype,
                    attributes,
                    edges,
                } => (stype, edges, attributes),
            };

            let num_edges = edges.len();
            for (edge_index, edge) in edges.into_iter().enumerate() {
                match edge {
                    SegmentBorderPiece::Vert(index) => {
                        poly.push(index);
                    }
                    SegmentBorderPiece::Edge(portion, orientation) => {
                        use Orientation::*;
                        use SegmentEdgePortion::*;

                        let mount = mounts.by_orientation_mut(orientation);
                        match portion {
                            Beginning => mount[0] = i,
                            Middle => mount[1] = i,
                            End => mount[2] = i,
                            Full => *mount = [i; MOUNTS_PER_SIDE],
                        }

                        const LM: f32 = 0.45; // low middle
                        const HM: f32 = 0.55; // high middle
                        const XB: Vec2 = Vec2::ZERO;
                        const XLM: Vec2 = vec2(LM, 0.0); // x beginning
                        const XHM: Vec2 = vec2(HM, 0.0); // x middle
                        const XE: Vec2 = Vec2::X; // x end
                        const YB: Vec2 = Vec2::ZERO; // y beginning
                        const YLM: Vec2 = vec2(0.0, LM); // y beginning
                        const YHM: Vec2 = vec2(0.0, HM); // y middle
                        const YE: Vec2 = Vec2::Y; // y end
                        let [start_vert, end_vert] = match (portion, orientation) {
                            (Beginning, North) => [XB + YB, XLM + YB],
                            (Middle, North) => [XLM + YB, XHM + YB],
                            (End, North) => [XHM + YB, XE + YB],
                            (Full, North) => [XB + YB, XE + YB],

                            (Beginning, East) => [XE + YB, XE + YLM],
                            (Middle, East) => [XE + YLM, XE + YHM],
                            (End, East) => [XE + YHM, XE + YE],
                            (Full, East) => [XE + YB, XE + YE],

                            (Beginning, South) => [XE + YE, XHM + YE],
                            (Middle, South) => [XHM + YE, XLM + YE],
                            (End, South) => [XLM + YE, XB + YE],
                            (Full, South) => [XE + YE, XB + YE],

                            (Beginning, West) => [XB + YE, XB + YHM],
                            (Middle, West) => [XB + YHM, XB + YLM],
                            (End, West) => [XB + YLM, XB + YB],
                            (Full, West) => [XB + YE, XB + YB],
                        };

                        let this_edge = (portion, orientation);
                        let mut poly_indicies = [0, 0];
                        let start_index = if let Some((_, &[_, i])) = edge_verts_map
                            .iter()
                            .find(|(&key, _)| edges_contiguous(key, this_edge))
                        {
                            i
                        } else {
                            verts.push(start_vert);
                            verts.len() - 1
                        };
                        if poly.last() != Some(&start_index) {
                            poly.push(start_index);
                        }
                        poly_indicies[0] = start_index;

                        let end_index = if let Some((_, &[i, _])) = edge_verts_map
                            .iter()
                            .find(|(&key, _)| edges_contiguous(this_edge, key))
                        {
                            i
                        } else {
                            verts.push(end_vert);
                            verts.len() - 1
                        };
                        if !(edge_index == num_edges - 1 && poly.first() == Some(&end_index)) {
                            poly.push(end_index);
                        }
                        poly_indicies[1] = end_index;

                        edge_verts_map.insert(this_edge, poly_indicies);
                    }
                }
            }

            let meeple_spot = match attributes
                .iter()
                .filter_map(|attr| match attr {
                    SegmentAttribute::CustomMeepleSpot(pos) => Some(pos),
                    _ => None,
                })
                .next()
            {
                Some(pos) => *pos,
                None => {
                    poly.iter().map(|i| verts[*i]).reduce(|a, b| a + b).unwrap() / poly.len() as f32
                }
            };

            segments.push(Segment {
                stype,
                poly,
                attributes,
                meeple_spot,
            });
        }

        let segment_adjacency = (0..segments.len())
            .flat_map(|i| {
                (0..segments.len())
                    .map(|j| {
                        (i != j)
                            && segments[i]
                                .poly
                                .iter()
                                .any(|k| segments[j].poly.contains(k))
                    })
                    .collect::<Vec<_>>()
            })
            .collect();

        Tile {
            verts,
            segments,
            mounts,
            segment_adjacency,
            attributes,
        }
    }

    pub fn new(verts: Vec<Vec2>, segment_definitions: Vec<SegmentDefinition>) -> Self {
        Tile::new_with_attributes(verts, segment_definitions, Vec::new())
    }

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
        }
        for segment in &mut self.segments {
            let mspot = &mut segment.meeple_spot;
            *mspot = vec2(1.0 - mspot.y, mspot.x);
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

    pub fn adjacent_segments(
        &self,
        seg_index: SegmentIndex,
    ) -> impl Iterator<Item = (SegmentIndex, &Segment)> {
        let n = self.segments.len();
        self.segment_adjacency[n * seg_index..(n + 1) * seg_index]
            .iter()
            .enumerate()
            .filter_map(|(i, a)| a.then_some((i, &self.segments[i])))
        // let seg_poly = &self.segments[seg_index].poly;
        // self.segments
        //     .iter()
        //     .enumerate()
        //     .filter_map(move |(i, segment)| {
        //         (i != seg_index && segment.poly.iter().any(|j| seg_poly.contains(j)))
        //             .then_some((i, &self.segments[i]))
        //     })
    }
}

pub fn get_tile_library() -> Vec<Tile> {
    vec![
        CITY_ENTRANCE.clone(),
        STRAIGHT_ROAD.clone(),
        CORNER_CITY.clone(),
        L_CURVE_ROAD.clone(),
    ]
    .into_iter()
    .cycle()
    .take(20)
    .collect()
}

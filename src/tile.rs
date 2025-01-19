pub mod tile_definitions;

use std::{collections::HashMap, vec};

use ggez::{
    glam::{vec2, Vec2},
    graphics::{Canvas, Color, DrawMode, DrawParam, Mesh, Rect},
    Context, GameError,
};
use tile_definitions::{
    ADJACENT_EDGE_CITIES, BRIDGE_CITY, CORNER_CITY, CORNER_CITY_CURVE_ROAD, CROSSROADS, CURVE_ROAD,
    EDGE_CITY, EDGE_CITY_CROSSROADS, EDGE_CITY_LEFT_CURVE_ROAD, EDGE_CITY_RIGHT_CURVE_ROAD,
    FORITIFED_THREE_QUARTER_CITY_ENTRANCE, FORTIFIED_BRIDGE_CITY, FORTIFIED_CORNER_CITY,
    FORTIFIED_CORNER_CITY_CURVE_ROAD, FORTIFIED_THREE_QUARTER_CITY, FOUR_WAY_CROSSROADS,
    FULL_FORTIFIED_CITY, MONASTARY, OPPOSING_EDGE_CITIES, ROAD_MONASTARY, STARTING_TILE,
    STRAIGHT_ROAD, THREE_QUARTER_CITY, THREE_QUARTER_CITY_ENTRANCE,
};

use crate::{
    game::SegmentIndex,
    pos::GridPos,
    util::{refit_to_rect, RotateExt},
};

#[cfg(test)]
mod test;

const MOUNTS_PER_SIDE: usize = 3;

pub type Mount = [usize; MOUNTS_PER_SIDE];
pub type TileEdge = (TileEdgeSpan, Orientation);

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum Orientation {
    North,
    East,
    South,
    West,
}

impl Orientation {
    pub fn rotate(self) -> Orientation {
        use Orientation::*;
        match self {
            North => East,
            East => South,
            South => West,
            West => North,
        }
    }

    pub fn offset(&self) -> GridPos {
        use Orientation::*;
        match self {
            North => GridPos(0, -1),
            East => GridPos(1, 0),
            South => GridPos(0, 1),
            West => GridPos(-1, 0),
        }
    }

    pub fn iter_with_offsets() -> impl Iterator<Item = (Orientation, GridPos)> {
        use Orientation::*;
        [North, East, South, West]
            .into_iter()
            .map(|orientation| (orientation, orientation.offset()))
    }
}

impl Opposite for Orientation {
    fn opposite(self) -> Self {
        use Orientation::*;
        match self {
            North => South,
            East => West,
            South => North,
            West => East,
        }
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

    pub fn placeable(&self) -> bool {
        !matches!(self, SegmentType::Village)
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
    Fortified { shield_location: Vec2 },
    CustomMeepleSpot(Vec2),
}

#[derive(Clone, Debug)]
pub struct Segment {
    pub stype: SegmentType,
    pub poly: Vec<usize>,
    pub attributes: Vec<SegmentAttribute>,
    pub meeple_spot: Vec2,
    pub edge_definition: Vec<SegmentBorderPiece>,
}

#[derive(Clone, Debug)]
pub enum TileAttribute {}

pub trait Opposite {
    fn opposite(self) -> Self;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, std::hash::Hash)]
pub enum TileEdgeSpanPosition {
    Start,
    LowerMiddle,
    UpperMiddle,
    End,
}

impl Opposite for TileEdgeSpanPosition {
    fn opposite(self) -> Self {
        use TileEdgeSpanPosition::*;
        match self {
            Start => End,
            LowerMiddle => UpperMiddle,
            UpperMiddle => LowerMiddle,
            End => Start,
        }
    }
}

pub type TileEdgeVertex = (TileEdgeSpanPosition, Orientation);

impl Opposite for TileEdgeVertex {
    fn opposite(self) -> Self {
        let (span, orientation) = self;
        (span.opposite(), orientation.opposite())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GridBorderCoordinateOffset {
    None,
    LowMiddleSouth,
    HighMiddleSouth,
    LowMiddleEast,
    HighMiddleEast,
}

impl GridBorderCoordinateOffset {
    pub fn to_position_offset(self) -> Vec2 {
        use GridBorderCoordinateOffset::*;
        match self {
            None => Vec2::ZERO,
            LowMiddleSouth => vec2(0.0, 0.45),
            HighMiddleSouth => vec2(0.0, 0.55),
            LowMiddleEast => vec2(0.45, 0.0),
            HighMiddleEast => vec2(0.55, 0.0),
        }
    }

    pub fn from_south_edge(position: TileEdgeSpanPosition) -> Self {
        use GridBorderCoordinateOffset::*;
        use TileEdgeSpanPosition::*;
        match position {
            Start => None,
            LowerMiddle => LowMiddleSouth,
            UpperMiddle => HighMiddleSouth,
            End => None,
        }
    }

    pub fn from_east_edge(position: TileEdgeSpanPosition) -> Self {
        use GridBorderCoordinateOffset::*;
        use TileEdgeSpanPosition::*;
        match position {
            Start => None,
            LowerMiddle => LowMiddleEast,
            UpperMiddle => HighMiddleEast,
            End => None,
        }
    }

    pub fn from_tile_edge_vertex(vertex: TileEdgeVertex) -> (GridPos, Self) {
        use Orientation::*;
        use TileEdgeSpanPosition::*;
        let (span, orientation) = vertex;
        let offset = match orientation {
            North => Self::from_east_edge(span),
            East => Self::from_south_edge(span),
            South => Self::from_east_edge(span).opposite(),
            West => Self::from_south_edge(span).opposite(),
        };
        let grid_offset = match (orientation, span) {
            (East, End) | (South, Start) => GridPos(1, 1),
            (East, _) | (North, End) => GridPos(1, 0),
            (South, _) | (West, Start) => GridPos(0, 1),
            _ => GridPos(0, 0),
        };
        (grid_offset, offset)
    }
}

impl Opposite for GridBorderCoordinateOffset {
    fn opposite(self) -> Self {
        use GridBorderCoordinateOffset::*;
        match self {
            None => None,
            LowMiddleSouth => HighMiddleSouth,
            HighMiddleSouth => LowMiddleSouth,
            LowMiddleEast => HighMiddleEast,
            HighMiddleEast => LowMiddleEast,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct GridBorderCoordinate {
    pub grid_pos: GridPos,
    pub offset: GridBorderCoordinateOffset,
}

impl GridBorderCoordinate {
    pub fn from_tile_edge_vertex(grid_pos: GridPos, vertex: TileEdgeVertex) -> Self {
        let (grid_offset, offset) = GridBorderCoordinateOffset::from_tile_edge_vertex(vertex);
        GridBorderCoordinate {
            grid_pos: grid_pos + grid_offset,
            offset,
        }
    }

    pub fn get_adjacent_gridposes(&self) -> impl Iterator<Item = GridPos> {
        let GridBorderCoordinate { grid_pos, offset } = self;
        let GridPos(x, y) = *grid_pos;
        use GridBorderCoordinateOffset::*;
        match offset {
            None => vec![
                GridPos(x - 1, y - 1),
                GridPos(x, y - 1),
                GridPos(x - 1, y),
                GridPos(x, y),
            ],
            LowMiddleSouth | HighMiddleSouth => vec![GridPos(x - 1, y), GridPos(x, y)],
            LowMiddleEast | HighMiddleEast => vec![GridPos(x, y - 1), GridPos(x, y)],
        }
        .into_iter()
    }
}

impl From<GridBorderCoordinate> for Vec2 {
    fn from(coord: GridBorderCoordinate) -> Self {
        let GridBorderCoordinate { grid_pos, offset } = coord;
        let GridPos(x, y) = grid_pos;
        let offset = offset.to_position_offset();
        vec2(x as f32, y as f32) + offset
    }
}

impl std::fmt::Debug for GridBorderCoordinate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let GridPos(x, y) = self.grid_pos;
        use GridBorderCoordinateOffset::*;
        write!(
            f,
            "{{{x}, {y}, {}}}",
            match self.offset {
                None => "*",
                LowMiddleSouth => "v",
                HighMiddleSouth => "vv",
                LowMiddleEast => ">",
                HighMiddleEast => ">>",
            }
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, std::hash::Hash)]
pub enum TileEdgeSpan {
    Beginning,
    Middle,
    End,
    Full,
}

impl TileEdgeSpan {
    pub fn start(self) -> TileEdgeSpanPosition {
        use TileEdgeSpan::*;
        use TileEdgeSpanPosition::*;
        match self {
            Beginning => Start,
            Middle => LowerMiddle,
            TileEdgeSpan::End => UpperMiddle,
            Full => Start,
        }
    }

    pub fn end(self) -> TileEdgeSpanPosition {
        use TileEdgeSpan::*;
        use TileEdgeSpanPosition::*;
        match self {
            Beginning => LowerMiddle,
            Middle => UpperMiddle,
            TileEdgeSpan::End => TileEdgeSpanPosition::End,
            Full => TileEdgeSpanPosition::End,
        }
    }
}

impl Opposite for TileEdgeSpan {
    fn opposite(self) -> Self {
        use TileEdgeSpan::*;
        match self {
            Beginning => End,
            Middle => Middle,
            End => Beginning,
            Full => Full,
        }
    }
}

impl Opposite for TileEdge {
    fn opposite(self) -> Self {
        let (portion, orientation) = self;
        (portion.opposite(), orientation.opposite())
    }
}

pub fn edges_contiguous(before: TileEdge, after: TileEdge) -> bool {
    use Orientation::*;
    use TileEdgeSpan::*;
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, std::hash::Hash)]
pub enum SegmentBorderPiece {
    Edge(TileEdge),
    Vert(usize),
    Break,
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
    #[allow(unused)]
    pub attributes: Vec<TileAttribute>,
    pub edge_verts_map: HashMap<TileEdge, [usize; 2]>,
}

impl Tile {
    pub fn new_with_attributes(
        mut verts: Vec<Vec2>,
        segment_definitions: Vec<SegmentDefinition>,
        attributes: Vec<TileAttribute>,
    ) -> Self {
        let mut edge_verts_map: HashMap<TileEdge, [usize; 2]> = HashMap::new();
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

            for (edge_index, edge) in edges.iter().cloned().enumerate() {
                match edge {
                    SegmentBorderPiece::Break => {}
                    SegmentBorderPiece::Vert(index) => {
                        poly.push(index);
                    }
                    SegmentBorderPiece::Edge(border_piece) => {
                        use Orientation::*;
                        use TileEdgeSpan::*;
                        let (portion, orientation) = border_piece;

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
                        let [start_vert, end_vert] = match border_piece {
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

                        let mut poly_indicies = [0, 0];
                        let start_index = if let Some((_, &[_, i])) = edge_verts_map
                            .iter()
                            .find(|(&key, _)| edges_contiguous(key, border_piece))
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
                            .find(|(&key, _)| edges_contiguous(border_piece, key))
                        {
                            i
                        } else {
                            verts.push(end_vert);
                            verts.len() - 1
                        };
                        if !(edge_index == edges.len() - 1 && poly.first() == Some(&end_index)) {
                            poly.push(end_index);
                        }
                        poly_indicies[1] = end_index;

                        edge_verts_map.insert(border_piece, poly_indicies);
                    }
                }
            }

            let meeple_spot = match attributes.iter().find_map(|attr| match attr {
                SegmentAttribute::CustomMeepleSpot(pos) => Some(pos),
                _ => None,
            }) {
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
                edge_definition: edges,
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
            edge_verts_map,
        }
    }

    pub fn new(verts: Vec<Vec2>, segment_definitions: Vec<SegmentDefinition>) -> Self {
        Tile::new_with_attributes(verts, segment_definitions, Vec::new())
    }

    pub fn segment_polygon(&self, seg_index: SegmentIndex) -> impl Iterator<Item = Vec2> + '_ {
        self.segments[seg_index]
            .poly
            .iter()
            .map(move |i| self.verts[*i])
    }

    pub fn refit_segment_polygon(
        &self,
        seg_index: SegmentIndex,
        bounds: Rect,
    ) -> impl Iterator<Item = Vec2> + '_ {
        self.segment_polygon(seg_index)
            .map(move |v| refit_to_rect(v, bounds))
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
        let verts: Vec<Vec2> = self.refit_segment_polygon(seg_index, bounds).collect();
        canvas.draw(
            &Mesh::new_polygon(
                ctx,
                DrawMode::fill(),
                &verts,
                color.unwrap_or_else(|| segment.stype.color()),
            )?,
            DrawParam::default(),
        );
        for attribute in &segment.attributes {
            if let SegmentAttribute::Fortified { shield_location } = attribute {
                canvas.draw(
                    &Mesh::new_polygon(
                        ctx,
                        DrawMode::fill(),
                        &[
                            vec2(-0.075, -0.075),
                            vec2(0.075, -0.075),
                            vec2(0.075, 0.075),
                            vec2(0.0, 0.15),
                            vec2(-0.075, 0.075),
                        ]
                        .into_iter()
                        .map(|offset| refit_to_rect(*shield_location + offset, bounds))
                        .collect::<Vec<_>>(),
                        Color::from_rgb(134, 146, 228),
                    )?,
                    DrawParam::default(),
                );
            }
        }
        Ok(())
    }

    pub fn rotate(&mut self) {
        self.verts.iter_mut().for_each(RotateExt::rotate_);
        for segment in &mut self.segments {
            segment.meeple_spot.rotate_();
            for edge in segment.edge_definition.iter_mut() {
                if let SegmentBorderPiece::Edge((_, orientation)) = edge {
                    *orientation = orientation.rotate();
                }
            }
            for attribute in &mut segment.attributes {
                if let SegmentAttribute::Fortified { shield_location } = attribute {
                    shield_location.rotate_();
                }
            }
        }
        self.edge_verts_map = self
            .edge_verts_map
            .iter()
            .map(|((p, orientation), v)| ((*p, orientation.rotate()), *v))
            .collect();

        self.mounts = self.mounts.clone().rotate();
    }

    #[allow(unused)]
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
    }
}

#[rustfmt::skip]
pub fn get_tile_library() -> Vec<Tile> {
    [
        (&*STARTING_TILE, 3),

        (&*STRAIGHT_ROAD, 8),
        (&*CURVE_ROAD, 9),
        (&*CROSSROADS, 4),
        (&*FOUR_WAY_CROSSROADS, 1),

        (&*EDGE_CITY, 5),
        (&*EDGE_CITY_LEFT_CURVE_ROAD, 3),
        (&*EDGE_CITY_RIGHT_CURVE_ROAD, 3),
        (&*EDGE_CITY_CROSSROADS, 3),

        (&*ADJACENT_EDGE_CITIES, 2),
        (&*OPPOSING_EDGE_CITIES, 3),

        (&*CORNER_CITY, 3),
        (&*FORTIFIED_CORNER_CITY, 2),
        (&*CORNER_CITY_CURVE_ROAD, 3),
        (&*FORTIFIED_CORNER_CITY_CURVE_ROAD, 2),

        (&*BRIDGE_CITY, 1),
        (&*FORTIFIED_BRIDGE_CITY, 2),

        (&*THREE_QUARTER_CITY, 3),
        (&*FORTIFIED_THREE_QUARTER_CITY, 1),
        (&*THREE_QUARTER_CITY_ENTRANCE, 1),
        (&*FORITIFED_THREE_QUARTER_CITY_ENTRANCE, 2),

        (&*FULL_FORTIFIED_CITY, 1),

        (&*MONASTARY, 4),
        (&*ROAD_MONASTARY, 2),
    ]
    .into_iter()
    .flat_map(|(tile, count)| vec![tile.clone(); count])
    .collect()
}

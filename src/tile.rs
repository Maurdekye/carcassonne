pub mod tile_definitions;

use std::{collections::HashMap, vec};

use ggez::{
    glam::{vec2, Vec2},
    graphics::{Canvas, Color, DrawMode, DrawParam, Mesh, Rect},
    Context, GameError,
};
use serde::{Deserialize, Serialize};
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
use ggez_no_re::line::Line;

#[cfg(test)]
mod test;

const DEFAULT_MIDDLE_SECTION_WIDTH: f32 = 0.1;

const MOUNTS_PER_SIDE: usize = 3;

pub type Mount = [usize; MOUNTS_PER_SIDE];
pub type TileEdge = (TileEdgeSpan, Orientation);

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SegmentType {
    Farm,
    City,
    Road,
    Monastary,
    Village,
    River,
}

impl SegmentType {
    pub fn color(&self) -> Color {
        use SegmentType::*;
        match self {
            Farm => Color::from_rgb(171, 219, 59),
            City => Color::from_rgb(222, 133, 38),
            Road => Color::from_rgb(207, 194, 149),
            Monastary => Color::from_rgb(183, 222, 235),
            Village => Color::from_rgb(227, 204, 166),
            River => Color::from_rgb(84, 118, 218),
        }
    }

    pub fn name(&self) -> &'static str {
        use SegmentType::*;
        match self {
            Farm => "Farm",
            City => "City",
            Road => "Road",
            Monastary => "Monastary",
            Village => "Village",
            River => "River",
        }
    }

    pub fn placeable(&self) -> bool {
        !matches!(self, SegmentType::Village | SegmentType::River)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum SegmentAttribute {
    Fortified { shield_location: Vec2 },
    CustomMeepleSpot(Vec2),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Segment {
    pub stype: SegmentType,
    pub poly: Vec<usize>,
    pub attributes: Vec<SegmentAttribute>,
    pub meeple_spot: Vec2,
    pub edge_definition: Vec<SegmentBorderPiece>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum TileAttribute {
    MiddleSegmentWidth(Orientation, f32),
}

pub trait Opposite {
    fn opposite(self) -> Self;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, std::hash::Hash)]
pub enum TileEdgeSpanPositionKind {
    Start,
    LowerMiddle,
    UpperMiddle,
    End,
}

#[derive(Debug, Clone, Copy)]
pub struct TileEdgeSpanPosition {
    kind: TileEdgeSpanPositionKind,
    middle_section_width: f32,
}

impl PartialEq for TileEdgeSpanPosition {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind
    }
}

impl Eq for TileEdgeSpanPosition {}

impl std::hash::Hash for TileEdgeSpanPosition {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.kind.hash(state);
    }
}

impl Opposite for TileEdgeSpanPosition {
    fn opposite(self) -> Self {
        use TileEdgeSpanPositionKind::*;
        let TileEdgeSpanPosition {
            kind,
            middle_section_width,
        } = self;
        let kind = match kind {
            Start => End,
            LowerMiddle => UpperMiddle,
            UpperMiddle => LowerMiddle,
            End => Start,
        };
        TileEdgeSpanPosition {
            kind,
            middle_section_width,
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
pub enum GridBorderCoordinateOffsetKind {
    None,
    LowMiddleSouth,
    HighMiddleSouth,
    LowMiddleEast,
    HighMiddleEast,
}

#[derive(Debug, Clone, Copy)]
pub struct GridBorderCoordinateOffset {
    kind: GridBorderCoordinateOffsetKind,
    middle_section_width: f32,
}

impl PartialEq for GridBorderCoordinateOffset {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind
    }
}

impl Eq for GridBorderCoordinateOffset {}

impl GridBorderCoordinateOffset {
    pub fn to_position_offset(self) -> Vec2 {
        use GridBorderCoordinateOffsetKind::*;
        let GridBorderCoordinateOffset {
            kind,
            middle_section_width,
        } = self;
        match kind {
            None => Vec2::ZERO,
            LowMiddleSouth => vec2(0.0, 0.5 - middle_section_width / 2.0),
            HighMiddleSouth => vec2(0.0, 0.5 + middle_section_width / 2.0),
            LowMiddleEast => vec2(0.5 - middle_section_width / 2.0, 0.0),
            HighMiddleEast => vec2(0.5 + middle_section_width / 2.0, 0.0),
        }
    }

    pub fn from_south_edge(position: TileEdgeSpanPosition) -> Self {
        use GridBorderCoordinateOffsetKind::*;
        use TileEdgeSpanPositionKind::*;
        let TileEdgeSpanPosition {
            kind,
            middle_section_width,
        } = position;
        let kind = match kind {
            Start => None,
            LowerMiddle => LowMiddleSouth,
            UpperMiddle => HighMiddleSouth,
            End => None,
        };
        GridBorderCoordinateOffset {
            kind,
            middle_section_width,
        }
    }

    pub fn from_east_edge(position: TileEdgeSpanPosition) -> Self {
        use GridBorderCoordinateOffsetKind::*;
        use TileEdgeSpanPositionKind::*;
        let TileEdgeSpanPosition {
            kind,
            middle_section_width,
        } = position;
        let kind = match kind {
            Start => None,
            LowerMiddle => LowMiddleEast,
            UpperMiddle => HighMiddleEast,
            End => None,
        };
        GridBorderCoordinateOffset {
            kind,
            middle_section_width,
        }
    }

    pub fn from_tile_edge_vertex(vertex: TileEdgeVertex) -> (GridPos, Self) {
        use Orientation::*;
        use TileEdgeSpanPositionKind::*;
        let (span, orientation) = vertex;
        let offset = match orientation {
            North => Self::from_east_edge(span),
            East => Self::from_south_edge(span),
            South => Self::from_east_edge(span).opposite(),
            West => Self::from_south_edge(span).opposite(),
        };
        let grid_offset = match (orientation, span.kind) {
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
        use GridBorderCoordinateOffsetKind::*;
        let GridBorderCoordinateOffset {
            kind,
            middle_section_width,
        } = self;
        let kind = match kind {
            None => None,
            LowMiddleSouth => HighMiddleSouth,
            HighMiddleSouth => LowMiddleSouth,
            LowMiddleEast => HighMiddleEast,
            HighMiddleEast => LowMiddleEast,
        };
        GridBorderCoordinateOffset {
            kind,
            middle_section_width,
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
        use GridBorderCoordinateOffsetKind::*;
        match offset.kind {
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
        use GridBorderCoordinateOffsetKind::*;
        write!(
            f,
            "{{{x}, {y}, {}{}}}",
            match self.offset.kind {
                None => "*",
                LowMiddleSouth => "v",
                HighMiddleSouth => "vv",
                LowMiddleEast => ">",
                HighMiddleEast => ">>",
            },
            if self.offset.middle_section_width != 0.1 {
                format!(", {}", self.offset.middle_section_width)
            } else {
                String::new()
            }
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, std::hash::Hash, Serialize, Deserialize)]
pub enum TileEdgeSpan {
    Beginning,
    Middle,
    End,
    Full,
}

impl TileEdgeSpan {
    pub fn start(self) -> TileEdgeSpanPositionKind {
        use TileEdgeSpan::*;
        use TileEdgeSpanPositionKind::*;
        match self {
            Beginning => Start,
            Middle => LowerMiddle,
            TileEdgeSpan::End => UpperMiddle,
            Full => Start,
        }
    }

    pub fn end(self) -> TileEdgeSpanPositionKind {
        use TileEdgeSpan::*;
        use TileEdgeSpanPositionKind::*;
        match self {
            Beginning => LowerMiddle,
            Middle => UpperMiddle,
            TileEdgeSpan::End => TileEdgeSpanPositionKind::End,
            Full => TileEdgeSpanPositionKind::End,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, std::hash::Hash, Serialize, Deserialize)]
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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Tile {
    pub verts: Line,
    pub segments: Vec<Segment>,
    pub mounts: Mounts,
    segment_adjacency: Vec<bool>,
    #[allow(unused)]
    pub attributes: Vec<TileAttribute>,
    pub edge_verts_map: HashMap<TileEdge, [usize; 2]>,
    pub rotation: usize,
}

impl Tile {
    pub fn default_library_tallies() -> Vec<(&'static Tile, usize)> {
        vec![
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
    }

    pub fn default_library() -> Vec<Tile> {
        Self::default_library_tallies()
            .into_iter()
            .flat_map(|(tile, count)| (0..count).map(|_| tile.clone()))
            .collect()
    }

    pub fn new_with_attributes(
        mut verts: Line,
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

        let tile_attributes = &attributes;

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

                        let middle_section_width =
                            Tile::get_middle_section_width(&tile_attributes, orientation);

                        let lm: f32 = 0.5 - middle_section_width / 2.0; // low middle
                        let hm: f32 = 0.5 + middle_section_width / 2.0; // high middle
                        let xb: Vec2 = Vec2::ZERO;
                        let xlm: Vec2 = vec2(lm, 0.0); // x beginning
                        let xhm: Vec2 = vec2(hm, 0.0); // x middle
                        let xe: Vec2 = Vec2::X; // x end
                        let yb: Vec2 = Vec2::ZERO; // y beginning
                        let ylm: Vec2 = vec2(0.0, lm); // y beginning
                        let yhm: Vec2 = vec2(0.0, hm); // y middle
                        let ye: Vec2 = Vec2::Y; // y end
                        let [start_vert, end_vert] = match border_piece {
                            (Beginning, North) => [xb + yb, xlm + yb],
                            (Middle, North) => [xlm + yb, xhm + yb],
                            (End, North) => [xhm + yb, xe + yb],
                            (Full, North) => [xb + yb, xe + yb],

                            (Beginning, East) => [xe + yb, xe + ylm],
                            (Middle, East) => [xe + ylm, xe + yhm],
                            (End, East) => [xe + yhm, xe + ye],
                            (Full, East) => [xe + yb, xe + ye],

                            (Beginning, South) => [xe + ye, xhm + ye],
                            (Middle, South) => [xhm + ye, xlm + ye],
                            (End, South) => [xlm + ye, xb + ye],
                            (Full, South) => [xe + ye, xb + ye],

                            (Beginning, West) => [xb + ye, xb + yhm],
                            (Middle, West) => [xb + yhm, xb + ylm],
                            (End, West) => [xb + ylm, xb + yb],
                            (Full, West) => [xb + ye, xb + yb],
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
            rotation: 0,
        }
    }

    pub fn new(verts: Line, segment_definitions: Vec<SegmentDefinition>) -> Self {
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
        let verts: Line = self.refit_segment_polygon(seg_index, bounds).collect();
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

    /// dumb hack but works lol
    pub fn rotate_counterclockwise(&mut self) {
        self.rotate_clockwise();
        self.rotate_clockwise();
        self.rotate_clockwise();
    }

    pub fn rotate_clockwise(&mut self) {
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

        for mut attribute in &mut self.attributes {
            #[allow(irrefutable_let_patterns)]
            if let TileAttribute::MiddleSegmentWidth(orientation, _) = &mut attribute {
                *orientation = orientation.rotate();
            }
        }

        self.mounts = self.mounts.clone().rotate();
        self.rotation = (self.rotation + 1) % 4;
    }

    #[allow(unused)]
    pub fn rotated(mut self) -> Self {
        self.rotate_clockwise();
        self
    }

    pub fn rotate_to(&mut self, rotations: usize) {
        while self.rotation != rotations % 4 {
            self.rotate_clockwise();
        }
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

    fn get_middle_section_width(attributes: &Vec<TileAttribute>, orientation: Orientation) -> f32 {
        attributes
            .iter()
            .find_map(|attr| match attr {
                TileAttribute::MiddleSegmentWidth(or, width) if *or == orientation => Some(*width),
                _ => None,
            })
            .unwrap_or(DEFAULT_MIDDLE_SECTION_WIDTH)
    }

    pub fn encode_edge_vertex(
        &self,
        span_position_kind: TileEdgeSpanPositionKind,
        orientation: Orientation,
    ) -> TileEdgeVertex {
        let middle_section_width = Tile::get_middle_section_width(&self.attributes, orientation);
        (
            TileEdgeSpanPosition {
                kind: span_position_kind,
                middle_section_width,
            },
            orientation,
        )
    }
}

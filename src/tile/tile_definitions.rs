use ggez::glam::vec2;
use lazy_static::lazy_static;

use crate::tile::{
    Orientation, SegmentAttribute, SegmentBorderPiece, SegmentDefinition, TileEdgeSpan,
    SegmentType, Tile,
};

use Orientation::*;
use SegmentBorderPiece::*;
use SegmentDefinition::*;
use TileEdgeSpan::*;
use SegmentType::*;

lazy_static! {
    pub static ref STRAIGHT_ROAD: Tile = Tile::new(
        vec![],
        vec![
            Segment {
                stype: Field,
                edges: vec![Edge((End, West)), Edge((Full, North)), Edge((Beginning, East))]
            },
            Segment {
                stype: Road,
                edges: vec![Edge((Middle, West)), Edge((Middle, East))]
            },
            Segment {
                stype: Field,
                edges: vec![Edge((Beginning, West)), Edge((End, East)), Edge((Full, South))]
            }
        ]
    );
    pub static ref L_CURVE_ROAD: Tile = Tile::new(
        vec![vec2(0.45, 0.45), vec2(0.55, 0.55)],
        vec![
            Segment {
                stype: Field,
                edges: vec![Edge((End, West)), Edge((Beginning, North)), Vert(0)]
            },
            SpecialSegment {
                stype: Road,
                edges: vec![Edge((Middle, West)), Vert(0), Edge((Middle, North)), Vert(1)],
                attributes: vec![SegmentAttribute::CustomMeepleSpot(vec2(0.5, 0.5))]
            },
            SpecialSegment {
                stype: Field,
                edges: vec![
                    Edge((Beginning, West)),
                    Vert(1),
                    Edge((End, North)),
                    Edge((Full, East)),
                    Edge((Full, South))
                ],
                attributes: vec![SegmentAttribute::CustomMeepleSpot(vec2(0.75, 0.75))]
            }
        ]
    );
    pub static ref CORNER_CITY: Tile = Tile::new(
        vec![],
        vec![
            Segment {
                stype: City,
                edges: vec![Edge((Full, West)), Edge((Full, North))]
            },
            Segment {
                stype: Field,
                edges: vec![Edge((Full, East)), Edge((Full, South))]
            }
        ]
    );
    pub static ref CITY_ENTRANCE: Tile = Tile::new(
        vec![vec2(0.45, 0.3), vec2(0.55, 0.3)],
        vec![
            Segment {
                stype: City,
                edges: vec![Edge((Full, North)), Vert(1), Vert(0)]
            },
            Segment {
                stype: Road,
                edges: vec![Edge((Middle, South)), Vert(0), Vert(1)]
            },
            Segment {
                stype: Field,
                edges: vec![Edge((End, South)), Edge((Full, West)), Vert(0)]
            },
            Segment {
                stype: Field,
                edges: vec![Edge((Full, East)), Edge((Beginning, South)), Vert(1)]
            }
        ]
    );
    pub static ref CROSSROADS: Tile = Tile::new(
        vec![
            vec2(0.35, 0.45),
            vec2(0.65, 0.45),
            vec2(0.65, 0.55),
            vec2(0.55, 0.65),
            vec2(0.45, 0.65),
            vec2(0.35, 0.55)
        ],
        vec![
            Segment {
                stype: Village,
                edges: vec![Vert(0), Vert(1), Vert(2), Vert(3), Vert(4), Vert(5)]
            },
            Segment {
                stype: Field,
                edges: vec![
                    Edge((End, West)),
                    Edge((Full, North)),
                    Edge((Beginning, East)),
                    Vert(1),
                    Vert(0)
                ]
            },
            Segment {
                stype: Road,
                edges: vec![Edge((Middle, East)), Vert(2), Vert(1)]
            },
            Segment {
                stype: Field,
                edges: vec![Edge((End, East)), Edge((Beginning, South)), Vert(3), Vert(2)]
            },
            Segment {
                stype: Road,
                edges: vec![Edge((Middle, South)), Vert(4), Vert(3)]
            },
            Segment {
                stype: Field,
                edges: vec![Edge((End, South)), Edge((Beginning, West)), Vert(5), Vert(4)]
            },
            Segment {
                stype: Road,
                edges: vec![Edge((Middle, West)), Vert(0), Vert(5)]
            }
        ]
    );
    pub static ref MONASTARY: Tile = Tile::new(
        vec![
            vec2(0.3, 0.7),
            vec2(0.3, 0.3),
            vec2(0.5, 0.15),
            vec2(0.7, 0.3),
            vec2(0.7, 0.7)
        ],
        vec![
            Segment {
                stype: Monastary,
                edges: vec![Vert(0), Vert(1), Vert(2), Vert(3), Vert(4),]
            },
            SpecialSegment {
                stype: Field,
                attributes: vec![SegmentAttribute::CustomMeepleSpot(vec2(0.8, 0.5))],
                edges: vec![
                    Edge((Full, West)),
                    Edge((Full, North)),
                    Edge((Full, East)),
                    Edge((Full, South)),
                    Break,
                    Vert(0),
                    Vert(4),
                    Vert(3),
                    Vert(2),
                    Vert(1),
                    Vert(0),
                    Break,
                ]
            }
        ]
    );
    pub static ref ROAD_MONASTARY: Tile = Tile::new(
        vec![
            vec2(0.3, 0.7),
            vec2(0.3, 0.3),
            vec2(0.5, 0.15),
            vec2(0.7, 0.3),
            vec2(0.7, 0.7),
            vec2(0.55, 0.7),
            vec2(0.45, 0.7)
        ],
        vec![
            Segment {
                stype: Monastary,
                edges: vec![Vert(0), Vert(1), Vert(2), Vert(3), Vert(4),]
            },
            SpecialSegment {
                stype: Field,
                attributes: vec![SegmentAttribute::CustomMeepleSpot(vec2(0.8, 0.5))],
                edges: vec![
                    Edge((End, South)),
                    Edge((Full, West)),
                    Edge((Full, North)),
                    Edge((Full, East)),
                    Edge((Beginning, South)),
                    Vert(5),
                    Vert(4),
                    Vert(3),
                    Vert(2),
                    Vert(1),
                    Vert(0),
                    Vert(6),
                ]
            },
            Segment {
                stype: Road,
                edges: vec![Edge((Middle, South)), Vert(6), Vert(5),]
            }
        ]
    );
}

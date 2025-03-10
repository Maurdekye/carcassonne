use std::vec;

use ggez::glam::vec2;
use lazy_static::lazy_static;

use crate::tile::{
    Orientation, SegmentAttribute, SegmentBorderPiece, SegmentDefinition, SegmentType, Tile,
    TileEdgeSpan,
};

use Orientation::*;
use SegmentBorderPiece::*;
use SegmentDefinition::*;
use SegmentType::*;
use TileEdgeSpan::*;

pub mod rivers_1 {
    use std::{iter::empty, vec};

    use ggez::glam::vec2;
    use lazy_static::lazy_static;

    use crate::tile::{
        Orientation, SegmentBorderPiece, SegmentDefinition, SegmentType, Tile, TileAttribute,
        TileEdgeSpan,
    };

    use Orientation::*;
    use SegmentBorderPiece::*;
    use SegmentDefinition::*;
    use SegmentType::*;
    use TileEdgeSpan::*;

    lazy_static! {
        pub static ref MONASTARY_POND: Tile = Tile::new_with_attributes(
            vec![
                vec2(0.6, 0.1),
                vec2(0.65, 0.15),
                vec2(0.65, 0.25),
                vec2(0.95, 0.3),
                vec2(0.95, 0.5),
                vec2(0.75, 0.55),
                vec2(0.55, 0.55),
                vec2(0.45, 0.6),
                vec2(0.4, 0.7),
                vec2(0.4, 0.75),
                vec2(0.45, 0.8),
                vec2(0.45, 0.9),
                vec2(0.4, 0.95),
                vec2(0.3, 0.95),
                vec2(0.15, 0.9),
                vec2(0.15, 0.75),
                vec2(0.1, 0.7),
                vec2(0.1, 0.6),
                vec2(0.25, 0.55),
                vec2(0.25, 0.5),
                vec2(0.15, 0.45),
                vec2(0.2, 0.35),
                vec2(0.15, 0.3),
                vec2(0.15, 0.25),
                vec2(0.2, 0.15),
                vec2(0.5, 0.15),
                vec2(0.5, 0.1),
                vec2(0.4, 0.05),
                vec2(0.9, 0.95),
                vec2(0.6, 0.95),
                vec2(0.6, 0.75),
                vec2(0.75, 0.6),
                vec2(0.9, 0.75),
            ],
            vec![
                Segment {
                    stype: River,
                    edges: empty()
                        .chain([Edge((Middle, North))])
                        .chain((0..=27).map(Vert))
                        .collect()
                },
                Segment {
                    stype: Monastary,
                    edges: (28..=32).map(Vert).collect()
                },
                Segment {
                    stype: Farm,
                    edges: empty()
                        .chain([Edge((End, North)), Edge((Full, East)), Break])
                        .chain((28..=32).rev().chain([32]).map(Vert))
                        .chain([
                            Break,
                            Edge((Full, South)),
                            Edge((Full, West)),
                            Edge((Beginning, North)),
                        ])
                        .chain((0..=27).rev().map(Vert))
                        .collect()
                }
            ],
            vec![TileAttribute::MiddleSegmentWidth(North, 0.2)]
        );
        pub static ref RIVER_CROSSING: Tile = Tile::new_with_attributes(
            vec![
                vec2(0.40, 0.95),
                vec2(0.30, 0.88),
                vec2(0.20, 0.77),
                vec2(0.15, 0.71),
                vec2(0.20, 0.55),
                vec2(0.29, 0.47),
                vec2(0.37, 0.40),
                vec2(0.40, 0.38),
                vec2(0.40, 0.30),
                vec2(0.34, 0.22),
                vec2(0.34, 0.15),
                vec2(0.40, 0.07),
                vec2(0.60, 0.05),
                vec2(0.54, 0.10),
                vec2(0.52, 0.23),
                vec2(0.61, 0.32),
                vec2(0.57, 0.45),
                vec2(0.55, 0.47),
                vec2(0.47, 0.54),
                vec2(0.40, 0.60),
                vec2(0.44, 0.65),
                vec2(0.43, 0.72),
                vec2(0.50, 0.75),
                vec2(0.49, 0.85),
                vec2(0.60, 0.95),
                vec2(0.08, 0.45),
                vec2(0.12, 0.41),
                vec2(0.18, 0.39),
                vec2(0.25, 0.39),
                vec2(0.41, 0.41),
                vec2(0.63, 0.55),
                vec2(0.72, 0.56),
                vec2(0.80, 0.55),
                vec2(0.93, 0.46),
                vec2(0.93, 0.55),
                vec2(0.87, 0.60),
                vec2(0.77, 0.64),
                vec2(0.65, 0.64),
                vec2(0.55, 0.60),
                vec2(0.42, 0.51),
                vec2(0.18, 0.48),
                vec2(0.10, 0.55),
            ],
            vec![
                Segment {
                    stype: River,
                    edges: empty()
                        .chain([Edge((Middle, South))])
                        .chain((0..=11).map(Vert))
                        .chain([Edge((Middle, North))])
                        .chain((13..=24).map(Vert))
                        .collect()
                },
                Segment {
                    stype: Road,
                    edges: empty()
                        .chain([Edge((Middle, West))])
                        .chain((25..=28).chain([6, 29, 17]).chain(30..=33).map(Vert))
                        .chain([Edge((Middle, East))])
                        .chain((34..=38).chain([18, 39, 5]).chain(40..=41).map(Vert))
                        .collect()
                },
                Segment {
                    stype: Farm,
                    edges: empty()
                        .chain([(End, South), (Beginning, West)].map(Edge))
                        .chain((0..=5).chain(40..=41).rev().map(Vert))
                        .collect()
                },
                Segment {
                    stype: Farm,
                    edges: empty()
                        .chain([(End, West), (Beginning, North)].map(Edge))
                        .chain((25..=28).chain(6..=11).rev().map(Vert))
                        .collect()
                },
                Segment {
                    stype: Farm,
                    edges: empty()
                        .chain([(End, North), (Beginning, East)].map(Edge))
                        .chain((13..=17).chain(30..=33).rev().map(Vert))
                        .collect()
                },
                Segment {
                    stype: Farm,
                    edges: empty()
                        .chain([(End, East), (Beginning, South)].map(Edge))
                        .chain((34..=38).chain(18..=24).rev().map(Vert))
                        .collect()
                }
            ],
            vec![
                TileAttribute::MiddleSegmentWidth(North, 0.2),
                TileAttribute::MiddleSegmentWidth(South, 0.2),
            ]
        );
        pub static ref RIVER_CORNER: Tile = Tile::new_with_attributes(
            vec![
                vec2(0.40, 0.80),
                vec2(0.45, 0.75),
                vec2(0.45, 0.65),
                vec2(0.30, 0.50),
                vec2(0.24, 0.50),
                vec2(0.15, 0.60),
                vec2(0.07, 0.40),
                vec2(0.17, 0.30),
                vec2(0.40, 0.30),
                vec2(0.66, 0.56),
                vec2(0.66, 0.75),
                vec2(0.60, 0.90),
            ],
            vec![
                Segment {
                    stype: River,
                    edges: empty()
                        .chain([Edge((Middle, South))])
                        .chain((0..=5).map(Vert))
                        .chain([Edge((Middle, West))])
                        .chain((6..=11).map(Vert))
                        .collect()
                },
                Segment {
                    stype: Farm,
                    edges: empty()
                        .chain([(End, South), (Beginning, West)].map(Edge))
                        .chain((0..=5).rev().map(Vert))
                        .collect()
                },
                Segment {
                    stype: Farm,
                    edges: empty()
                        .chain([
                            Edge((End, West)),
                            Edge((Full, North)),
                            Edge((Full, East)),
                            Edge((Beginning, South))
                        ])
                        .chain((6..=11).rev().map(Vert))
                        .collect()
                }
            ],
            vec![
                TileAttribute::MiddleSegmentWidth(West, 0.2),
                TileAttribute::MiddleSegmentWidth(South, 0.2),
            ]
        );
        pub static ref CORNER_CITY_RIVER: Tile = Tile::new_with_attributes(
            vec![
                vec2(0.90, 0.60),
                vec2(0.75, 0.70),
                vec2(0.60, 0.90),
                vec2(0.40, 0.91),
                vec2(0.50, 0.70),
                vec2(0.75, 0.45),
                vec2(0.90, 0.40),
            ],
            vec![
                Segment {
                    stype: River,
                    edges: empty()
                        .chain([Edge((Middle, East))])
                        .chain((0..=2).map(Vert))
                        .chain([Edge((Middle, South))])
                        .chain((3..=6).map(Vert))
                        .collect()
                },
                Segment {
                    stype: City,
                    edges: vec![Edge((Full, West)), Edge((Full, North))]
                },
                Segment {
                    stype: Farm,
                    edges: empty()
                        .chain([(End, South), (Beginning, East)].map(Edge))
                        .chain((3..=6).rev().map(Vert))
                        .collect()
                },
                Segment {
                    stype: Farm,
                    edges: empty()
                        .chain([(End, East), (Beginning, South)].map(Edge))
                        .chain((0..=2).rev().map(Vert))
                        .collect()
                }
            ],
            vec![
                TileAttribute::MiddleSegmentWidth(East, 0.2),
                TileAttribute::MiddleSegmentWidth(South, 0.2),
            ]
        );
        pub static ref RIVER_MONASTARY_BRIDGE: Tile = Tile::new_with_attributes(
            vec![
                vec2(0.60, 0.09),
                vec2(0.72, 0.11),
                vec2(0.82, 0.23),
                vec2(0.83, 0.42),
                vec2(0.69, 0.79),
                vec2(0.60, 0.92),
                vec2(0.40, 0.89),
                vec2(0.51, 0.81),
                vec2(0.64, 0.57),
                vec2(0.69, 0.35),
                vec2(0.65, 0.25),
                vec2(0.53, 0.22),
                vec2(0.40, 0.10),
                vec2(0.91, 0.55),
                vec2(0.85, 0.65),
                vec2(0.77, 0.63),
                vec2(0.66, 0.41),
                vec2(0.60, 0.37),
                vec2(0.53, 0.38),
                vec2(0.53, 0.32),
                vec2(0.59, 0.31),
                vec2(0.79, 0.57),
                vec2(0.82, 0.57),
                vec2(0.90, 0.45),
                vec2(0.53, 0.30),
                vec2(0.53, 0.60),
                vec2(0.20, 0.60),
                vec2(0.05, 0.45),
                vec2(0.20, 0.30),
                vec2(0.67, 0.43),
                vec2(0.76, 0.61),
                vec2(0.78, 0.55),
            ],
            vec![
                Segment {
                    stype: River,
                    edges: empty()
                        .chain([Edge((Middle, North))])
                        .chain((0..=5).map(Vert))
                        .chain([Edge((Middle, South))])
                        .chain((6..=8).chain([29]).chain(9..=12).map(Vert))
                        .collect()
                },
                Segment {
                    stype: Road,
                    edges: empty()
                        .chain([Edge((Middle, East))])
                        .chain((13..=20).chain([9]).chain(21..=23).map(Vert))
                        .collect()
                },
                Segment {
                    stype: Monastary,
                    edges: (24..=28).map(Vert).collect()
                },
                Segment {
                    stype: Farm,
                    edges: empty()
                        .chain([(End, South), (Full, West), (Beginning, North)].map(Edge))
                        .chain(
                            (6..=8)
                                .chain([29])
                                .chain(16..=18)
                                .chain(25..=28)
                                .chain([24])
                                .chain(19..=20)
                                .chain(9..=12)
                                .rev()
                                .map(Vert)
                        )
                        .collect()
                },
                Segment {
                    stype: Farm,
                    edges: empty()
                        .chain([(End, East), (Beginning, South)].map(Edge))
                        .chain([5, 4, 30, 15, 14, 13].map(Vert))
                        .collect()
                },
                Segment {
                    stype: Farm,
                    edges: empty()
                        .chain([(End, North), (Beginning, East)].map(Edge))
                        .chain([23, 22, 21, 31, 3, 2, 1, 0].map(Vert))
                        .collect()
                }
            ],
            vec![
                TileAttribute::MiddleSegmentWidth(North, 0.2),
                TileAttribute::MiddleSegmentWidth(South, 0.2),
            ]
        );
        pub static ref CURVY_STRAIGHT_RIVER: Tile = Tile::new_with_attributes(
            vec![
                vec2(0.40, 0.93),
                vec2(0.51, 0.82),
                vec2(0.65, 0.82),
                vec2(0.70, 0.79),
                vec2(0.70, 0.75),
                vec2(0.65, 0.72),
                vec2(0.45, 0.75),
                vec2(0.30, 0.75),
                vec2(0.16, 0.61),
                vec2(0.16, 0.50),
                vec2(0.25, 0.39),
                vec2(0.35, 0.35),
                vec2(0.40, 0.30),
                vec2(0.60, 0.34),
                vec2(0.47, 0.47),
                vec2(0.35, 0.47),
                vec2(0.28, 0.53),
                vec2(0.28, 0.58),
                vec2(0.35, 0.61),
                vec2(0.50, 0.55),
                vec2(0.74, 0.55),
                vec2(0.90, 0.68),
                vec2(0.90, 0.75),
                vec2(0.85, 0.85),
                vec2(0.60, 0.95),
            ],
            vec![
                Segment {
                    stype: River,
                    edges: empty()
                        .chain([Edge((Middle, South))])
                        .chain((0..=12).map(Vert))
                        .chain([Edge((Middle, North))])
                        .chain((13..=24).map(Vert))
                        .collect()
                },
                Segment {
                    stype: Farm,
                    edges: empty()
                        .chain([(End, South), (Full, West), (Beginning, North)].map(Edge))
                        .chain((0..=12).rev().map(Vert))
                        .collect()
                },
                Segment {
                    stype: Farm,
                    edges: empty()
                        .chain([(End, North), (Full, East), (Beginning, South)].map(Edge))
                        .chain((13..=24).rev().map(Vert))
                        .collect()
                }
            ],
            vec![
                TileAttribute::MiddleSegmentWidth(North, 0.2),
                TileAttribute::MiddleSegmentWidth(South, 0.2),
            ]
        );
        pub static ref CORNER_ROAD_CORNER_RIVER: Tile = Tile::new_with_attributes(
            vec![
                vec2(0.60, 0.20),
                vec2(0.80, 0.40),
                vec2(0.70, 0.60),
                vec2(0.40, 0.35),
                vec2(0.25, 0.45),
                vec2(0.55, 0.80),
                vec2(0.45, 0.83),
                vec2(0.20, 0.55),
            ],
            vec![
                Segment {
                    stype: River,
                    edges: vec![
                        Edge((Middle, North)),
                        Vert(0),
                        Vert(1),
                        Edge((Middle, East)),
                        Vert(2),
                        Vert(3),
                    ]
                },
                Segment {
                    stype: Road,
                    edges: vec![
                        Edge((Middle, West)),
                        Vert(4),
                        Vert(5),
                        Edge((Middle, South)),
                        Vert(6),
                        Vert(7)
                    ]
                },
                Segment {
                    stype: Farm,
                    edges: vec![
                        Edge((End, South)),
                        Edge((Beginning, West)),
                        Vert(7),
                        Vert(6)
                    ]
                },
                Segment {
                    stype: Farm,
                    edges: vec![
                        Edge((End, West)),
                        Edge((Beginning, North)),
                        Vert(3),
                        Vert(2),
                        Edge((End, East)),
                        Edge((Beginning, South)),
                        Vert(5),
                        Vert(4)
                    ]
                },
                Segment {
                    stype: Farm,
                    edges: vec![
                        Edge((End, North)),
                        Edge((Beginning, East)),
                        Vert(1),
                        Vert(0)
                    ]
                }
            ],
            vec![
                TileAttribute::MiddleSegmentWidth(North, 0.2),
                TileAttribute::MiddleSegmentWidth(East, 0.2),
            ],
        );
        pub static ref STRAIGHT_RIVER: Tile = Tile::new_with_attributes(
            vec![],
            vec![
                Segment {
                    stype: River,
                    edges: vec![Edge((Middle, West)), Edge((Middle, East))]
                },
                Segment {
                    stype: Farm,
                    edges: vec![
                        Edge((End, West)),
                        Edge((Full, North)),
                        Edge((Beginning, East))
                    ]
                },
                Segment {
                    stype: Farm,
                    edges: vec![
                        Edge((End, East)),
                        Edge((Full, South)),
                        Edge((Beginning, West))
                    ]
                }
            ],
            vec![
                TileAttribute::MiddleSegmentWidth(East, 0.2),
                TileAttribute::MiddleSegmentWidth(West, 0.2),
            ]
        );
        pub static ref STRAIGHT_RIVER_DUELING_CITIES: Tile = Tile::new_with_attributes(
            vec![
                vec2(0.25, 0.25),
                vec2(0.25, 0.75),
                vec2(0.75, 0.75),
                vec2(0.75, 0.25),
            ],
            vec![
                Segment {
                    stype: River,
                    edges: vec![Edge((Middle, South)), Edge((Middle, North))]
                },
                Segment {
                    stype: City,
                    edges: vec![Edge((Full, West)), Vert(0), Vert(1)]
                },
                Segment {
                    stype: City,
                    edges: vec![Edge((Full, East)), Vert(2), Vert(3)]
                },
                Segment {
                    stype: Farm,
                    edges: vec![
                        Edge((Beginning, North)),
                        Edge((End, South)),
                        Vert(1),
                        Vert(0)
                    ]
                },
                Segment {
                    stype: Farm,
                    edges: vec![
                        Edge((Beginning, South)),
                        Edge((End, North)),
                        Vert(3),
                        Vert(2)
                    ]
                }
            ],
            vec![
                TileAttribute::MiddleSegmentWidth(North, 0.2),
                TileAttribute::MiddleSegmentWidth(South, 0.2),
            ]
        );
        pub static ref CITY_RIVER_CROSSING: Tile = Tile::new_with_attributes(
            vec![
                vec2(0.60, 0.40),
                vec2(0.40, 0.70),
                vec2(0.40, 0.75),
                vec2(0.60, 0.90),
                vec2(0.40, 0.95),
                vec2(0.20, 0.75),
                vec2(0.20, 0.60),
                vec2(0.40, 0.30),
                vec2(0.09, 0.45),
                vec2(0.15, 0.30),
                vec2(0.35, 0.30),
                vec2(0.54, 0.49),
                vec2(0.54, 0.65),
                vec2(0.57, 0.65),
                vec2(0.65, 0.45),
                vec2(0.75, 0.45),
                vec2(0.75, 0.55),
                vec2(0.70, 0.55),
                vec2(0.60, 0.75),
                vec2(0.50, 0.75),
                vec2(0.44, 0.68),
                vec2(0.44, 0.49),
                vec2(0.30, 0.35),
                vec2(0.20, 0.35),
                vec2(0.10, 0.55),
                vec2(0.75, 0.75),
                vec2(0.75, 0.25),
                vec2(0.34, 0.39),
                vec2(0.38, 0.33),
                vec2(0.54, 0.49),
                vec2(0.44, 0.64),
            ],
            vec![
                Segment {
                    stype: River,
                    edges: empty()
                        .chain([Edge((Middle, North))])
                        .chain((0..=3).map(Vert))
                        .chain([Edge((Middle, South))])
                        .chain((4..=7).map(Vert))
                        .collect()
                },
                Segment {
                    stype: Road,
                    edges: empty()
                        .chain([Edge((Middle, West))])
                        .chain((8..=24).map(Vert))
                        .collect()
                },
                Segment {
                    stype: City,
                    edges: vec![Edge((Full, East)), Vert(25), Vert(26)]
                },
                Segment {
                    stype: Farm,
                    edges: empty()
                        .chain([(End, South), (Beginning, West)].map(Edge))
                        .chain([24, 23, 22, 27, 6, 5, 4].map(Vert))
                        .collect()
                },
                Segment {
                    stype: Farm,
                    edges: empty()
                        .chain([(End, West), (Beginning, North)].map(Edge))
                        .chain([7, 28, 10, 9, 8].map(Vert))
                        .collect()
                },
                Segment {
                    stype: Farm,
                    edges: empty()
                        .chain([Edge((End, North))])
                        .chain([26, 15, 14, 13, 12, 29, 0].map(Vert))
                        .collect()
                },
                Segment {
                    stype: Farm,
                    edges: empty()
                        .chain([Edge((Beginning, South))])
                        .chain(
                            [3, 2, 1, 30]
                                .into_iter()
                                .chain((16..=20).rev())
                                .chain([25])
                                .map(Vert)
                        )
                        .collect()
                }
            ],
            vec![
                TileAttribute::MiddleSegmentWidth(North, 0.2),
                TileAttribute::MiddleSegmentWidth(South, 0.2)
            ]
        );
        pub static ref CORNER_ROAD_WATERFALL: Tile = Tile::new_with_attributes(
            vec![
                vec2(0.55, 0.22),
                vec2(0.78, 0.45),
                vec2(0.75, 0.55),
                vec2(0.45, 0.25),
                vec2(0.40, 0.80),
                vec2(0.30, 0.70),
                vec2(0.30, 0.60),
                vec2(0.55, 0.35),
                vec2(0.65, 0.45),
                vec2(0.50, 0.60),
                vec2(0.50, 0.65),
                vec2(0.60, 0.75),
            ],
            vec![
                Segment {
                    stype: River,
                    edges: empty()
                        .chain([Edge((Middle, South))])
                        .chain((4..=11).map(Vert))
                        .collect()
                },
                Segment {
                    stype: Road,
                    edges: vec![
                        Edge((Middle, North)),
                        Vert(0),
                        Vert(1),
                        Edge((Middle, East)),
                        Vert(2),
                        Vert(3)
                    ]
                },
                Segment {
                    stype: Farm,
                    edges: empty()
                        .chain([(End, South), (Full, West), (Beginning, North)].map(Edge))
                        .chain((4..=7).chain([3]).rev().map(Vert))
                        .collect()
                },
                Segment {
                    stype: Farm,
                    edges: vec![
                        Edge((End, North)),
                        Edge((Beginning, East)),
                        Vert(1),
                        Vert(0)
                    ]
                },
                Segment {
                    stype: Farm,
                    edges: empty()
                        .chain([(End, East), (Beginning, South)].map(Edge))
                        .chain((8..=11).rev().chain([2]).map(Vert))
                        .collect()
                }
            ],
            vec![TileAttribute::MiddleSegmentWidth(South, 0.2)]
        );
    }
}

lazy_static! {
    pub static ref STARTING_TILE: Tile = Tile::new(
        vec![vec2(0.35, 0.3), vec2(0.65, 0.3)],
        vec![
            Segment {
                stype: City,
                edges: vec![Edge((Full, North)), Vert(1), Vert(0)]
            },
            Segment {
                stype: Road,
                edges: vec![Edge((Middle, West)), Edge((Middle, East))]
            },
            SpecialSegment {
                stype: Farm,
                edges: vec![Edge((End, West)), Vert(0), Vert(1), Edge((Beginning, East))],
                attributes: vec![SegmentAttribute::CustomMeepleSpot(vec2(0.85, 0.315))]
            },
            Segment {
                stype: Farm,
                edges: vec![
                    Edge((Beginning, West)),
                    Edge((End, East)),
                    Edge((Full, South))
                ]
            }
        ]
    );
    pub static ref STRAIGHT_ROAD: Tile = Tile::new(
        vec![],
        vec![
            Segment {
                stype: Farm,
                edges: vec![
                    Edge((End, West)),
                    Edge((Full, North)),
                    Edge((Beginning, East))
                ]
            },
            Segment {
                stype: Road,
                edges: vec![Edge((Middle, West)), Edge((Middle, East))]
            },
            Segment {
                stype: Farm,
                edges: vec![
                    Edge((Beginning, West)),
                    Edge((End, East)),
                    Edge((Full, South))
                ]
            }
        ]
    );
    pub static ref CURVE_ROAD: Tile = Tile::new(
        vec![vec2(0.45, 0.45), vec2(0.55, 0.55)],
        vec![
            Segment {
                stype: Farm,
                edges: vec![Edge((End, West)), Edge((Beginning, North)), Vert(0)]
            },
            SpecialSegment {
                stype: Road,
                edges: vec![
                    Edge((Middle, West)),
                    Vert(0),
                    Edge((Middle, North)),
                    Vert(1)
                ],
                attributes: vec![SegmentAttribute::CustomMeepleSpot(vec2(0.5, 0.5))]
            },
            SpecialSegment {
                stype: Farm,
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
            SpecialSegment {
                stype: Farm,
                edges: vec![
                    Edge((End, West)),
                    Edge((Full, North)),
                    Edge((Beginning, East)),
                    Vert(1),
                    Vert(0)
                ],
                attributes: vec![SegmentAttribute::CustomMeepleSpot(vec2(0.5, 0.25))]
            },
            Segment {
                stype: Road,
                edges: vec![Edge((Middle, East)), Vert(2), Vert(1)]
            },
            Segment {
                stype: Farm,
                edges: vec![
                    Edge((End, East)),
                    Edge((Beginning, South)),
                    Vert(3),
                    Vert(2)
                ]
            },
            Segment {
                stype: Road,
                edges: vec![Edge((Middle, South)), Vert(4), Vert(3)]
            },
            Segment {
                stype: Farm,
                edges: vec![
                    Edge((End, South)),
                    Edge((Beginning, West)),
                    Vert(5),
                    Vert(4)
                ]
            },
            Segment {
                stype: Road,
                edges: vec![Edge((Middle, West)), Vert(0), Vert(5)]
            }
        ]
    );
    pub static ref FOUR_WAY_CROSSROADS: Tile = Tile::new(
        vec![
            vec2(0.35, 0.45),
            vec2(0.45, 0.35),
            vec2(0.55, 0.35),
            vec2(0.65, 0.45),
            vec2(0.65, 0.55),
            vec2(0.55, 0.65),
            vec2(0.45, 0.65),
            vec2(0.35, 0.55)
        ],
        vec![
            Segment {
                stype: Village,
                edges: (0..=7).map(Vert).collect()
            },
            Segment {
                stype: Farm,
                edges: vec![
                    Edge((End, West)),
                    Edge((Beginning, North)),
                    Vert(1),
                    Vert(0)
                ]
            },
            Segment {
                stype: Road,
                edges: vec![Edge((Middle, North)), Vert(2), Vert(1)]
            },
            Segment {
                stype: Farm,
                edges: vec![
                    Edge((End, North)),
                    Edge((Beginning, East)),
                    Vert(3),
                    Vert(2)
                ]
            },
            Segment {
                stype: Road,
                edges: vec![Edge((Middle, East)), Vert(4), Vert(3)]
            },
            Segment {
                stype: Farm,
                edges: vec![
                    Edge((End, East)),
                    Edge((Beginning, South)),
                    Vert(5),
                    Vert(4)
                ]
            },
            Segment {
                stype: Road,
                edges: vec![Edge((Middle, South)), Vert(6), Vert(5)]
            },
            Segment {
                stype: Farm,
                edges: vec![
                    Edge((End, South)),
                    Edge((Beginning, West)),
                    Vert(7),
                    Vert(6)
                ]
            },
            Segment {
                stype: Road,
                edges: vec![Edge((Middle, West)), Vert(0), Vert(7)]
            }
        ]
    );
    pub static ref EDGE_CITY: Tile = Tile::new(
        vec![vec2(0.35, 0.3), vec2(0.65, 0.3),],
        vec![
            Segment {
                stype: City,
                edges: vec![Edge((Full, North)), Vert(1), Vert(0),]
            },
            SpecialSegment {
                stype: Farm,
                edges: vec![
                    Edge((Full, East)),
                    Edge((Full, South)),
                    Edge((Full, West)),
                    Vert(0),
                    Vert(1),
                ],
                attributes: vec![SegmentAttribute::CustomMeepleSpot(vec2(0.5, 0.65))]
            },
        ]
    );
    pub static ref EDGE_CITY_LEFT_CURVE_ROAD: Tile = Tile::new(
        vec![
            vec2(0.35, 0.3),
            vec2(0.65, 0.3),
            vec2(0.45, 0.55),
            vec2(0.55, 0.45)
        ],
        vec![
            Segment {
                stype: City,
                edges: vec![Edge((Full, North)), Vert(1), Vert(0)]
            },
            SpecialSegment {
                stype: Road,
                edges: vec![
                    Edge((Middle, West)),
                    Vert(3),
                    Edge((Middle, South)),
                    Vert(2)
                ],
                attributes: vec![SegmentAttribute::CustomMeepleSpot(vec2(0.5, 0.5))]
            },
            SpecialSegment {
                stype: Farm,
                edges: vec![
                    Edge((End, West)),
                    Vert(0),
                    Vert(1),
                    Edge((Full, East)),
                    Edge((Beginning, South)),
                    Vert(3)
                ],
                attributes: vec![SegmentAttribute::CustomMeepleSpot(vec2(0.75, 0.5))]
            },
            Segment {
                stype: Farm,
                edges: vec![Edge((End, South)), Edge((Beginning, West)), Vert(2)]
            }
        ]
    );
    pub static ref EDGE_CITY_RIGHT_CURVE_ROAD: Tile = Tile::new(
        vec![
            vec2(0.35, 0.3),
            vec2(0.65, 0.3),
            vec2(0.45, 0.45),
            vec2(0.55, 0.55)
        ],
        vec![
            Segment {
                stype: City,
                edges: vec![Edge((Full, North)), Vert(1), Vert(0)]
            },
            SpecialSegment {
                stype: Road,
                edges: vec![
                    Edge((Middle, East)),
                    Vert(3),
                    Edge((Middle, South)),
                    Vert(2)
                ],
                attributes: vec![SegmentAttribute::CustomMeepleSpot(vec2(0.5, 0.5))]
            },
            SpecialSegment {
                stype: Farm,
                edges: vec![
                    Edge((Full, West)),
                    Vert(0),
                    Vert(1),
                    Edge((Beginning, East)),
                    Vert(2),
                    Edge((End, South)),
                ],
                attributes: vec![SegmentAttribute::CustomMeepleSpot(vec2(0.25, 0.5))]
            },
            Segment {
                stype: Farm,
                edges: vec![Edge((End, East)), Edge((Beginning, South)), Vert(3)]
            }
        ]
    );
    pub static ref EDGE_CITY_CROSSROADS: Tile = Tile::new(
        vec![
            vec2(0.35, 0.45),
            vec2(0.65, 0.45),
            vec2(0.65, 0.55),
            vec2(0.55, 0.65),
            vec2(0.45, 0.65),
            vec2(0.35, 0.55),
            vec2(0.35, 0.3),
            vec2(0.65, 0.3),
        ],
        vec![
            Segment {
                stype: Village,
                edges: vec![Vert(0), Vert(1), Vert(2), Vert(3), Vert(4), Vert(5)]
            },
            Segment {
                stype: City,
                edges: vec![Edge((Full, North)), Vert(7), Vert(6)]
            },
            SpecialSegment {
                stype: Farm,
                edges: vec![
                    Edge((End, West)),
                    Vert(6),
                    Vert(7),
                    Edge((Beginning, East)),
                    Vert(1),
                    Vert(0)
                ],
                attributes: vec![SegmentAttribute::CustomMeepleSpot(vec2(0.85, 0.315))]
            },
            Segment {
                stype: Road,
                edges: vec![Edge((Middle, East)), Vert(2), Vert(1)]
            },
            Segment {
                stype: Farm,
                edges: vec![
                    Edge((End, East)),
                    Edge((Beginning, South)),
                    Vert(3),
                    Vert(2)
                ]
            },
            Segment {
                stype: Road,
                edges: vec![Edge((Middle, South)), Vert(4), Vert(3)]
            },
            Segment {
                stype: Farm,
                edges: vec![
                    Edge((End, South)),
                    Edge((Beginning, West)),
                    Vert(5),
                    Vert(4)
                ]
            },
            Segment {
                stype: Road,
                edges: vec![Edge((Middle, West)), Vert(0), Vert(5)]
            }
        ]
    );
    pub static ref ADJACENT_EDGE_CITIES: Tile = Tile::new(
        vec![
            vec2(0.35, 0.3),
            vec2(0.65, 0.3),
            vec2(1.0, 0.0),
            vec2(0.7, 0.35),
            vec2(0.7, 0.65),
        ],
        vec![
            Segment {
                stype: City,
                edges: vec![Edge((Full, North)), Vert(1), Vert(0),]
            },
            Segment {
                stype: City,
                edges: vec![Edge((Full, East)), Vert(4), Vert(3),]
            },
            SpecialSegment {
                stype: Farm,
                edges: vec![
                    Edge((Full, South)),
                    Edge((Full, West)),
                    Vert(0),
                    Vert(1),
                    Vert(2),
                    Vert(3),
                    Vert(4),
                ],
                attributes: vec![SegmentAttribute::CustomMeepleSpot(vec2(0.35, 0.65))]
            },
        ]
    );
    pub static ref OPPOSING_EDGE_CITIES: Tile = Tile::new(
        vec![
            vec2(0.35, 0.3),
            vec2(0.65, 0.3),
            vec2(0.35, 0.7),
            vec2(0.65, 0.7)
        ],
        vec![
            Segment {
                stype: City,
                edges: vec![Edge((Full, North)), Vert(1), Vert(0)]
            },
            Segment {
                stype: City,
                edges: vec![Edge((Full, South)), Vert(2), Vert(3)]
            },
            Segment {
                stype: Farm,
                edges: vec![
                    Edge((Full, West)),
                    Vert(0),
                    Vert(1),
                    Edge((Full, East)),
                    Vert(3),
                    Vert(2)
                ]
            }
        ]
    );
    pub static ref BRIDGE_CITY: Tile = Tile::new(
        vec![
            vec2(0.35, 0.3),
            vec2(0.65, 0.3),
            vec2(0.35, 0.7),
            vec2(0.65, 0.7)
        ],
        vec![
            Segment {
                stype: Farm,
                edges: vec![Edge((Full, North)), Vert(1), Vert(0)]
            },
            Segment {
                stype: Farm,
                edges: vec![Edge((Full, South)), Vert(2), Vert(3)]
            },
            Segment {
                stype: City,
                edges: vec![
                    Edge((Full, West)),
                    Vert(0),
                    Vert(1),
                    Edge((Full, East)),
                    Vert(3),
                    Vert(2)
                ]
            }
        ]
    );
    pub static ref FORTIFIED_BRIDGE_CITY: Tile = Tile::new(
        vec![
            vec2(0.35, 0.3),
            vec2(0.65, 0.3),
            vec2(0.35, 0.7),
            vec2(0.65, 0.7)
        ],
        vec![
            Segment {
                stype: Farm,
                edges: vec![Edge((Full, North)), Vert(1), Vert(0)]
            },
            Segment {
                stype: Farm,
                edges: vec![Edge((Full, South)), Vert(2), Vert(3)]
            },
            SpecialSegment {
                stype: City,
                edges: vec![
                    Edge((Full, West)),
                    Vert(0),
                    Vert(1),
                    Edge((Full, East)),
                    Vert(3),
                    Vert(2)
                ],
                attributes: vec![SegmentAttribute::Fortified {
                    shield_location: vec2(0.2, 0.3)
                }]
            }
        ]
    );
    pub static ref THREE_QUARTER_CITY: Tile = Tile::new(
        vec![vec2(0.35, 0.7), vec2(0.65, 0.7),],
        vec![
            SpecialSegment {
                stype: City,
                edges: vec![
                    Edge((Full, West)),
                    Edge((Full, North)),
                    Edge((Full, East)),
                    Vert(1),
                    Vert(0)
                ],
                attributes: vec![SegmentAttribute::CustomMeepleSpot(vec2(0.5, 0.35))]
            },
            Segment {
                stype: Farm,
                edges: vec![Edge((Full, South)), Vert(0), Vert(1)]
            }
        ]
    );
    pub static ref FORTIFIED_THREE_QUARTER_CITY: Tile = Tile::new(
        vec![vec2(0.35, 0.7), vec2(0.65, 0.7),],
        vec![
            SpecialSegment {
                stype: City,
                edges: vec![
                    Edge((Full, West)),
                    Edge((Full, North)),
                    Edge((Full, East)),
                    Vert(1),
                    Vert(0)
                ],
                attributes: vec![
                    SegmentAttribute::Fortified {
                        shield_location: vec2(0.2, 0.2)
                    },
                    SegmentAttribute::CustomMeepleSpot(vec2(0.5, 0.35))
                ]
            },
            Segment {
                stype: Farm,
                edges: vec![Edge((Full, South)), Vert(0), Vert(1)]
            }
        ]
    );
    pub static ref THREE_QUARTER_CITY_ENTRANCE: Tile = Tile::new(
        vec![
            vec2(0.35, 0.7),
            vec2(0.45, 0.7),
            vec2(0.55, 0.7),
            vec2(0.65, 0.7),
        ],
        vec![
            SpecialSegment {
                stype: City,
                edges: vec![
                    Edge((Full, West)),
                    Edge((Full, North)),
                    Edge((Full, East)),
                    Vert(3),
                    Vert(0)
                ],
                attributes: vec![SegmentAttribute::CustomMeepleSpot(vec2(0.5, 0.35))]
            },
            Segment {
                stype: Road,
                edges: vec![Edge((Middle, South)), Vert(1), Vert(2)]
            },
            Segment {
                stype: Farm,
                edges: vec![Edge((Beginning, South)), Vert(2), Vert(3)]
            },
            Segment {
                stype: Farm,
                edges: vec![Edge((End, South)), Vert(0), Vert(1)]
            }
        ]
    );
    pub static ref FORITIFED_THREE_QUARTER_CITY_ENTRANCE: Tile = Tile::new(
        vec![
            vec2(0.35, 0.7),
            vec2(0.45, 0.7),
            vec2(0.55, 0.7),
            vec2(0.65, 0.7),
        ],
        vec![
            SpecialSegment {
                stype: City,
                edges: vec![
                    Edge((Full, West)),
                    Edge((Full, North)),
                    Edge((Full, East)),
                    Vert(3),
                    Vert(0)
                ],
                attributes: vec![
                    SegmentAttribute::Fortified {
                        shield_location: vec2(0.2, 0.2)
                    },
                    SegmentAttribute::CustomMeepleSpot(vec2(0.5, 0.35))
                ]
            },
            Segment {
                stype: Road,
                edges: vec![Edge((Middle, South)), Vert(1), Vert(2)]
            },
            Segment {
                stype: Farm,
                edges: vec![Edge((Beginning, South)), Vert(2), Vert(3)]
            },
            Segment {
                stype: Farm,
                edges: vec![Edge((End, South)), Vert(0), Vert(1)]
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
                stype: Farm,
                edges: vec![Edge((Full, East)), Edge((Full, South))]
            }
        ]
    );
    pub static ref FORTIFIED_CORNER_CITY: Tile = Tile::new(
        vec![],
        vec![
            SpecialSegment {
                stype: City,
                edges: vec![Edge((Full, West)), Edge((Full, North))],
                attributes: vec![SegmentAttribute::Fortified {
                    shield_location: vec2(0.2, 0.2)
                }]
            },
            Segment {
                stype: Farm,
                edges: vec![Edge((Full, East)), Edge((Full, South))]
            }
        ]
    );
    pub static ref CORNER_CITY_CURVE_ROAD: Tile = Tile::new(
        vec![
            vec2(0.45, 0.7),
            vec2(0.7, 0.45),
            vec2(0.77071, 0.55),
            vec2(0.55, 0.77071)
        ],
        vec![
            Segment {
                stype: City,
                edges: vec![Edge((Full, West)), Edge((Full, North))],
            },
            SpecialSegment {
                stype: Road,
                edges: vec![
                    Edge((Middle, South)),
                    Vert(0),
                    Vert(1),
                    Edge((Middle, East)),
                    Vert(2),
                    Vert(3)
                ],
                attributes: vec![SegmentAttribute::CustomMeepleSpot(vec2(0.618, 0.618))]
            },
            SpecialSegment {
                stype: Farm,
                edges: vec![
                    Edge((End, South)),
                    Edge((Beginning, East)),
                    Vert(1),
                    Vert(0),
                ],
                attributes: vec![SegmentAttribute::CustomMeepleSpot(vec2(0.86, 0.32))]
            },
            Segment {
                stype: Farm,
                edges: vec![
                    Edge((End, East)),
                    Edge((Beginning, South)),
                    Vert(3),
                    Vert(2),
                ]
            }
        ]
    );
    pub static ref FORTIFIED_CORNER_CITY_CURVE_ROAD: Tile = Tile::new(
        vec![
            vec2(0.45, 0.7),
            vec2(0.7, 0.45),
            vec2(0.77071, 0.55),
            vec2(0.55, 0.77071)
        ],
        vec![
            SpecialSegment {
                stype: City,
                edges: vec![Edge((Full, West)), Edge((Full, North))],
                attributes: vec![SegmentAttribute::Fortified {
                    shield_location: vec2(0.2, 0.2)
                }]
            },
            SpecialSegment {
                stype: Road,
                edges: vec![
                    Edge((Middle, South)),
                    Vert(0),
                    Vert(1),
                    Edge((Middle, East)),
                    Vert(2),
                    Vert(3)
                ],
                attributes: vec![SegmentAttribute::CustomMeepleSpot(vec2(0.618, 0.618))]
            },
            SpecialSegment {
                stype: Farm,
                edges: vec![
                    Edge((End, South)),
                    Edge((Beginning, East)),
                    Vert(1),
                    Vert(0),
                ],
                attributes: vec![SegmentAttribute::CustomMeepleSpot(vec2(0.86, 0.32))]
            },
            Segment {
                stype: Farm,
                edges: vec![
                    Edge((End, East)),
                    Edge((Beginning, South)),
                    Vert(3),
                    Vert(2),
                ]
            }
        ]
    );
    pub static ref FULL_FORTIFIED_CITY: Tile = Tile::new(
        vec![],
        vec![SpecialSegment {
            stype: City,
            edges: vec![
                Edge((Full, West)),
                Edge((Full, North)),
                Edge((Full, East)),
                Edge((Full, South)),
            ],
            attributes: vec![SegmentAttribute::Fortified {
                shield_location: vec2(0.2, 0.2)
            }]
        }]
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
                stype: Farm,
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
                ],
                attributes: vec![SegmentAttribute::CustomMeepleSpot(vec2(0.85, 0.5))],
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
                stype: Farm,
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
                ],
                attributes: vec![SegmentAttribute::CustomMeepleSpot(vec2(0.85, 0.5))],
            },
            Segment {
                stype: Road,
                edges: vec![Edge((Middle, South)), Vert(6), Vert(5),]
            }
        ]
    );
    pub static ref _DEBUG_EMPTY_FIELD: Tile = Tile::new(
        vec![],
        vec![Segment {
            stype: Farm,
            edges: vec![
                Edge((Full, West)),
                Edge((Full, North)),
                Edge((Full, East)),
                Edge((Full, South))
            ]
        }]
    );
}

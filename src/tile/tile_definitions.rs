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
    use std::vec;

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
                vec2(0.7, 0.1),
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
                vec2(0.35, 0.05),
                vec2(0.9, 0.95),
                vec2(0.6, 0.95),
                vec2(0.6, 0.75),
                vec2(0.75, 0.6),
                vec2(0.9, 0.75),
            ],
            vec![
                Segment {
                    stype: River,
                    edges: vec![
                        Edge((Middle, North)),
                        Vert(0),
                        Vert(1),
                        Vert(2),
                        Vert(3),
                        Vert(4),
                        Vert(5),
                        Vert(6),
                        Vert(7),
                        Vert(8),
                        Vert(9),
                        Vert(10),
                        Vert(11),
                        Vert(12),
                        Vert(13),
                        Vert(14),
                        Vert(15),
                        Vert(16),
                        Vert(17),
                        Vert(18),
                        Vert(19),
                        Vert(20),
                        Vert(21),
                        Vert(22),
                        Vert(23),
                        Vert(24),
                        Vert(25),
                        Vert(26),
                        Vert(27),
                    ]
                },
                Segment {
                    stype: Monastary,
                    edges: vec![Vert(28), Vert(29), Vert(30), Vert(31), Vert(32)]
                },
                Segment {
                    stype: Farm,
                    edges: vec![
                        Edge((End, North)),
                        Edge((Full, East)),
                        Break,
                        Vert(32),
                        Vert(31),
                        Vert(30),
                        Vert(29),
                        Vert(28),
                        Vert(32),
                        Break,
                        Edge((Full, South)),
                        Edge((Full, West)),
                        Edge((Beginning, North)),
                        Vert(27),
                        Vert(26),
                        Vert(25),
                        Vert(24),
                        Vert(23),
                        Vert(22),
                        Vert(21),
                        Vert(20),
                        Vert(19),
                        Vert(18),
                        Vert(17),
                        Vert(16),
                        Vert(15),
                        Vert(14),
                        Vert(13),
                        Vert(12),
                        Vert(11),
                        Vert(10),
                        Vert(9),
                        Vert(8),
                        Vert(7),
                        Vert(6),
                        Vert(5),
                        Vert(4),
                        Vert(3),
                        Vert(2),
                        Vert(1),
                        Vert(0),
                    ]
                }
            ],
            vec![TileAttribute::MiddleSegmentWidth(North, 0.3)]
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
                edges: vec![Edge((Middle, West)), Edge((Middle, East)),]
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
                edges: vec![
                    Vert(0),
                    Vert(1),
                    Vert(2),
                    Vert(3),
                    Vert(4),
                    Vert(5),
                    Vert(6),
                    Vert(7)
                ]
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

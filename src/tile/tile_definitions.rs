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
            Segment {
                stype: Field,
                edges: vec![Edge((End, West)), Vert(0), Vert(1), Edge((Beginning, East))]
            },
            Segment {
                stype: Field,
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
                stype: Field,
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
                stype: Field,
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
                stype: Field,
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
                stype: Field,
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
                stype: Field,
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
                stype: Field,
                edges: vec![
                    Edge((End, North)),
                    Edge((Beginning, East)),
                    Vert(2),
                    Vert(3)
                ]
            },
            Segment {
                stype: Road,
                edges: vec![Edge((Middle, East)), Vert(4), Vert(3)]
            },
            Segment {
                stype: Field,
                edges: vec![
                    Edge((End, East)),
                    Edge((Beginning, South)),
                    Vert(3),
                    Vert(2)
                ]
            },
            Segment {
                stype: Road,
                edges: vec![Edge((Middle, South)), Vert(6), Vert(5)]
            },
            Segment {
                stype: Field,
                edges: vec![
                    Edge((End, South)),
                    Edge((Beginning, West)),
                    Vert(7),
                    Vert(5)
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
            Segment {
                stype: Field,
                edges: vec![
                    Edge((Full, East)),
                    Edge((Full, South)),
                    Edge((Full, West)),
                    Vert(0),
                    Vert(1),
                ]
            },
        ]
    );
    pub static ref EDGE_CITY_ENTRANCE: Tile = Tile::new(
        vec![
            vec2(0.35, 0.3),
            vec2(0.45, 0.3),
            vec2(0.55, 0.3),
            vec2(0.65, 0.3)
        ],
        vec![
            Segment {
                stype: City,
                edges: vec![Edge((Full, North)), Vert(0), Vert(3)]
            },
            Segment {
                stype: Road,
                edges: vec![Edge((Middle, South)), Vert(1), Vert(2)]
            },
            Segment {
                stype: Field,
                edges: vec![Edge((End, South)), Edge((Full, West)), Vert(0), Vert(1)]
            },
            Segment {
                stype: Field,
                edges: vec![
                    Edge((Full, East)),
                    Edge((Beginning, South)),
                    Vert(2),
                    Vert(3)
                ]
            }
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
            Segment {
                stype: Road,
                edges: vec![
                    Edge((Middle, West)),
                    Vert(3),
                    Edge((Middle, South)),
                    Vert(2)
                ]
            },
            Segment {
                stype: Field,
                edges: vec![
                    Edge((End, West)),
                    Vert(0),
                    Vert(1),
                    Edge((Full, East)),
                    Edge((Beginning, South)),
                    Vert(3)
                ]
            },
            Segment {
                stype: Field,
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
            Segment {
                stype: Road,
                edges: vec![
                    Edge((Middle, East)),
                    Vert(3),
                    Edge((Middle, South)),
                    Vert(2)
                ]
            },
            Segment {
                stype: Field,
                edges: vec![
                    Edge((Full, West)),
                    Vert(0),
                    Vert(1),
                    Edge((Beginning, East)),
                    Vert(2),
                    Edge((End, South)),
                ]
            },
            Segment {
                stype: Field,
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
            Segment {
                stype: Field,
                edges: vec![
                    Edge((End, West)),
                    Vert(6),
                    Vert(7),
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
                stype: Field,
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
                edges: vec![Edge((Full, East)), Vert(3), Vert(4),]
            },
            Segment {
                stype: Field,
                edges: vec![
                    Edge((Full, South)),
                    Edge((Full, West)),
                    Vert(0),
                    Vert(1),
                    Vert(2),
                    Vert(3),
                    Vert(4),
                ]
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
                stype: Field,
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
                stype: Field,
                edges: vec![Edge((Full, North)), Vert(1), Vert(0)]
            },
            Segment {
                stype: Field,
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
                stype: Field,
                edges: vec![Edge((Full, North)), Vert(1), Vert(0)]
            },
            Segment {
                stype: Field,
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
            Segment {
                stype: City,
                edges: vec![
                    Edge((Full, West)),
                    Edge((Full, North)),
                    Edge((Full, East)),
                    Vert(1),
                    Vert(0)
                ]
            },
            Segment {
                stype: Field,
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
                attributes: vec![SegmentAttribute::Fortified {
                    shield_location: vec2(0.2, 0.2)
                }]
            },
            Segment {
                stype: Field,
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
            Segment {
                stype: City,
                edges: vec![
                    Edge((Full, West)),
                    Edge((Full, North)),
                    Edge((Full, East)),
                    Vert(3),
                    Vert(0)
                ]
            },
            Segment {
                stype: Road,
                edges: vec![Edge((Middle, South)), Vert(1), Vert(2)]
            },
            Segment {
                stype: Field,
                edges: vec![Edge((Beginning, South)), Vert(2), Vert(3)]
            },
            Segment {
                stype: Field,
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
                attributes: vec![SegmentAttribute::Fortified {
                    shield_location: vec2(0.2, 0.2)
                }]
            },
            Segment {
                stype: Road,
                edges: vec![Edge((Middle, South)), Vert(1), Vert(2)]
            },
            Segment {
                stype: Field,
                edges: vec![Edge((Beginning, South)), Vert(2), Vert(3)]
            },
            Segment {
                stype: Field,
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
                stype: Field,
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
                stype: Field,
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
            Segment {
                stype: Road,
                edges: vec![
                    Edge((Middle, South)),
                    Vert(0),
                    Vert(1),
                    Edge((Middle, East)),
                    Vert(2),
                    Vert(3)
                ]
            },
            Segment {
                stype: Field,
                edges: vec![
                    Edge((End, South)),
                    Edge((Beginning, East)),
                    Vert(1),
                    Vert(0),
                ]
            },
            Segment {
                stype: Field,
                edges: vec![
                    Edge((Beginning, South)),
                    Vert(4),
                    Vert(3),
                    Edge((End, East)),
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
            Segment {
                stype: Road,
                edges: vec![
                    Edge((Middle, South)),
                    Vert(0),
                    Vert(1),
                    Edge((Middle, East)),
                    Vert(2),
                    Vert(3)
                ]
            },
            Segment {
                stype: Field,
                edges: vec![
                    Edge((End, South)),
                    Edge((Beginning, East)),
                    Vert(1),
                    Vert(0),
                ]
            },
            Segment {
                stype: Field,
                edges: vec![
                    Edge((Beginning, South)),
                    Vert(4),
                    Vert(3),
                    Edge((End, East)),
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
    pub static ref _DEBUG_EMPTY_FIELD: Tile = Tile::new(
        vec![],
        vec![Segment {
            stype: Field,
            edges: vec![
                Edge((Full, West)),
                Edge((Full, North)),
                Edge((Full, East)),
                Edge((Full, South))
            ]
        }]
    );
}

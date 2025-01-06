use ggez::glam::vec2;
use lazy_static::lazy_static;

use crate::tile::{Mounts, Segment, SegmentType};

use super::Tile;
use SegmentType::*;

lazy_static! {
    pub static ref STRAIGHT_ROAD: Tile = Tile {
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
        segments: vec![
            Segment {
                stype: Field,
                poly: vec![0, 3, 6, 4],
            },
            Segment {
                stype: Road,
                poly: vec![4, 6, 7, 5],
            },
            Segment {
                stype: Field,
                poly: vec![5, 7, 2, 1],
            },
        ],
        mounts: Mounts {
            north: [0, 0, 0],
            east: [0, 1, 2],
            south: [2, 2, 2],
            west: [2, 1, 0]
        }
    };
    pub static ref L_CURVE_ROAD: Tile = Tile {
        verts: vec![
            vec2(0.0, 0.0),
            vec2(0.0, 1.0),
            vec2(1.0, 1.0),
            vec2(1.0, 0.0),
            vec2(0.0, 0.45),
            vec2(0.0, 0.55),
            vec2(0.45, 0.0),
            vec2(0.55, 0.0),
            vec2(0.45, 0.45),
            vec2(0.55, 0.55),
        ],
        segments: vec![
            Segment {
                stype: Field,
                poly: vec![0, 6, 8, 4],
            },
            Segment {
                stype: Road,
                poly: vec![4, 8, 6, 7, 9, 5],
            },
            Segment {
                stype: Field,
                poly: vec![5, 9, 7, 3, 2, 1],
            },
        ],
        mounts: Mounts {
            north: [0, 1, 2],
            east: [2, 2, 2],
            south: [2, 2, 2],
            west: [2, 1, 0]
        }
    };
    pub static ref CORNER_CITY: Tile = Tile {
        verts: vec![
            vec2(0.0, 0.0),
            vec2(0.0, 1.0),
            vec2(1.0, 1.0),
            vec2(1.0, 0.0),
        ],
        segments: vec![
            Segment {
                stype: City,
                poly: vec![0, 3, 1]
            },
            Segment {
                stype: Field,
                poly: vec![1, 3, 2]
            },
        ],
        mounts: Mounts {
            north: [0, 0, 0],
            east: [1, 1, 1],
            south: [1, 1, 1],
            west: [0, 0, 0]
        }
    };
    pub static ref CITY_ENTRANCE: Tile = Tile {
        verts: vec![
            vec2(0.0, 0.0),
            vec2(0.0, 1.0),
            vec2(1.0, 1.0),
            vec2(1.0, 0.0),
            vec2(0.45, 0.3),
            vec2(0.55, 0.3),
            vec2(0.45, 1.0),
            vec2(0.55, 1.0),
        ],
        segments: vec![
            Segment {
                stype: City,
                poly: vec![0, 3, 5, 4]
            },
            Segment {
                stype: Road,
                poly: vec![4, 5, 7, 6]
            },
            Segment {
                stype: Field,
                poly: vec![0, 4, 6, 1]
            },
            Segment {
                stype: Field,
                poly: vec![5, 3, 2, 7]
            }
        ],
        mounts: Mounts {
            north: [0, 0, 0],
            east: [3, 3, 3],
            south: [3, 1, 2],
            west: [2, 2, 2]
        }
    };
    pub static ref DEAD_END_ROAD: Tile = Tile {
        verts: vec![
            vec2(0.0, 0.0),
            vec2(0.0, 1.0),
            vec2(1.0, 1.0),
            vec2(1.0, 0.0),
            vec2(0.45, 1.0),
            vec2(0.55, 1.0),
            vec2(0.45, 0.45),
            vec2(0.55, 0.45),
        ],
        segments: vec![
            Segment {
                stype: Field,
                poly: vec![0, 3, 2, 5, 7, 6, 4, 1]
            },
            Segment {
                stype: Road,
                poly: vec![4, 6, 7, 5]
            }
        ],
        mounts: Mounts {
            north: [0, 0, 0],
            east: [0, 0, 0],
            south: [0, 1, 0],
            west: [0, 0, 0]
        }
    };
    pub static ref FIELD: Tile = Tile {
        verts: vec![
            vec2(0.0, 0.0),
            vec2(0.0, 1.0),
            vec2(1.0, 1.0),
            vec2(1.0, 0.0),
        ],
        segments: vec![Segment {
            stype: Field,
            poly: vec![0, 1, 2, 3]
        }],
        mounts: Mounts {
            north: [0, 0, 0],
            east: [0, 0, 0],
            south: [0, 0, 0],
            west: [0, 0, 0]
        }
    };
}

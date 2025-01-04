use ggez::glam::vec2;
use lazy_static::lazy_static;

use crate::tile::{Mount, Mounts, Segment, SegmentType};

use super::Tile;

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
                stype: SegmentType::Field,
                poly: vec![0, 3, 6, 4],
                mounts: Mounts {
                    north: Some(Mount::Full),
                    east: Some(Mount::Beginning),
                    south: None,
                    west: Some(Mount::End),
                },
            },
            Segment {
                stype: SegmentType::Road,
                poly: vec![4, 6, 7, 5],
                mounts: Mounts {
                    north: None,
                    east: Some(Mount::Middle),
                    south: None,
                    west: Some(Mount::Middle),
                },
            },
            Segment {
                stype: SegmentType::Field,
                poly: vec![5, 7, 2, 1],
                mounts: Mounts {
                    north: None,
                    east: Some(Mount::End),
                    south: Some(Mount::Full),
                    west: Some(Mount::Beginning),
                },
            },
        ],
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
                stype: SegmentType::Field,
                poly: vec![0, 6, 8, 4],
                mounts: Mounts {
                    north: Some(Mount::Beginning),
                    east: None,
                    south: None,
                    west: Some(Mount::End)
                }
            },
            Segment {
                stype: SegmentType::Field,
                poly: vec![5, 9, 7, 3, 2, 1],
                mounts: Mounts {
                    north: Some(Mount::End),
                    east: Some(Mount::Full),
                    south: Some(Mount::Full),
                    west: Some(Mount::Beginning),
                }
            },
            Segment {
                stype: SegmentType::Road,
                poly: vec![4, 8, 6, 7, 9, 5],
                mounts: Mounts {
                    north: Some(Mount::Middle),
                    east: None,
                    south: None,
                    west: Some(Mount::Middle)
                }
            }
        ]
    };
}

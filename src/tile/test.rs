use crate::tile::{
    tile_definitions::{L_CURVE_ROAD, STRAIGHT_ROAD},
    MountingPair, Orientation,
};

#[test]
fn straight_road_mount_north_south() {
    let result = STRAIGHT_ROAD.validate_mounting(&STRAIGHT_ROAD, Orientation::North);
    assert_eq!(
        result,
        Some(vec![MountingPair {
            from_segment: 0,
            to_segment: 2,
        }])
    )
}

#[test]
fn straight_road_mount_east_west() {
    let result = STRAIGHT_ROAD.validate_mounting(&STRAIGHT_ROAD, Orientation::East);
    assert_eq!(
        result,
        Some(vec![
            MountingPair {
                from_segment: 0,
                to_segment: 0,
            },
            MountingPair {
                from_segment: 1,
                to_segment: 1,
            },
            MountingPair {
                from_segment: 2,
                to_segment: 2,
            },
        ],)
    )
}

#[test]
fn straight_road_curve_road_mount_west() {
    let result = STRAIGHT_ROAD.validate_mounting(&L_CURVE_ROAD, Orientation::West);
    assert_eq!(result, None);
}

#[test]
fn straight_road_curve_road_rotate_mount_west() {
    let mut curve_road = L_CURVE_ROAD.clone();
    curve_road.rotate();
    let result = STRAIGHT_ROAD.validate_mounting(&curve_road, Orientation::West);
    assert_eq!(
        result,
        Some(vec![
            MountingPair {
                from_segment: 2,
                to_segment: 2
            },
            MountingPair {
                from_segment: 1,
                to_segment: 1
            },
            MountingPair {
                from_segment: 0,
                to_segment: 0
            }
        ])
    );
}

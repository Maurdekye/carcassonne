use std::collections::HashMap;

use ggez::GameError;
use slotmap::{DefaultKey, SlotMap};

use crate::{
    pos::Pos,
    tile::{get_tile_library, Orientation, SegmentType, Tile},
};

pub type SegmentIdentifier = (Pos, usize);
pub type GroupIdentifier = DefaultKey;

#[derive(Debug)]
pub struct SegmentGroup {
    pub gtype: SegmentType,
    pub segments: Vec<SegmentIdentifier>,
}

pub struct Game {
    pub library: Vec<Tile>,
    pub placed_tiles: HashMap<Pos, Tile>,
    pub groups: SlotMap<GroupIdentifier, SegmentGroup>,
    pub group_associations: HashMap<SegmentIdentifier, GroupIdentifier>,
}

impl Game {
    pub fn new() -> Game {
        Game {
            library: get_tile_library(),
            placed_tiles: HashMap::new(),
            groups: SlotMap::new(),
            group_associations: HashMap::new(),
        }
    }

    pub fn place_tile(&mut self, tile: Tile, pos: Pos) -> Result<(), GameError> {
        let mut new_group_insertions: HashMap<SegmentIdentifier, Vec<GroupIdentifier>> =
            HashMap::new();
        let mut uninserted_segments: Vec<_> = (0..tile.segments.len()).map(|_| true).collect();
        // dbg!(&self.groups);
        // dbg!(&self.group_associations);

        // evaluate mountings with neighboring tiles
        for (orientation, offset) in Orientation::iter_with_offsets() {
            let adjacent_pos = pos + offset;
            // dbg!(&adjacent_pos);
            let Some(adjacent_tile) = self.placed_tiles.get(&adjacent_pos) else {
                continue;
            };

            // dbg!(&adjacent_tile);

            let Some(mounts) = tile.validate_mounting(adjacent_tile, orientation) else {
                return Err(GameError::CustomError(
                    "Attempt to place invalid tile!".to_string(),
                ));
            };
            // dbg!(&mounts);

            for mount in mounts {
                let seg_id: SegmentIdentifier = (pos, mount.from_segment);
                let adj_seg_id: SegmentIdentifier = (adjacent_pos, mount.to_segment);
                // dbg!(&seg_id);
                // dbg!(&adj_seg_id);
                let group_key = self
                    .group_associations
                    .get(&adj_seg_id)
                    .expect("All placed segments have associated groups");
                new_group_insertions
                    .entry(seg_id)
                    .and_modify(|groups| {
                        if !groups.contains(group_key) {
                            groups.push(*group_key)
                        }
                    })
                    .or_insert(vec![*group_key]);
                // dbg!(&new_group_insertions);
                uninserted_segments[mount.from_segment] = false;
            }
        }
        // dbg!(&new_group_insertions);

        // insert segments into existing connected groups
        let (singletons, coallations): (Vec<_>, Vec<_>) = new_group_insertions
            .into_iter()
            .partition(|(_, additions)| additions.len() == 1);

        // dbg!(&singletons);
        // attach segments to existing groups
        for (seg_id, additions) in singletons {
            self.groups
                .get_mut(additions[0])
                .unwrap()
                .segments
                .push(seg_id);
            self.group_associations.insert(seg_id, additions[0]);
        }

        // dbg!(&coallations);
        // combine disconnected groups that were connected together by the new segment
        let mut rewrites = HashMap::new();
        for (seg_id, additions) in coallations {
            let mut new_segment_list: Vec<_> = additions
                .iter()
                .flat_map(|key| {
                    self.groups
                        .remove(*rewrites.get(key).unwrap_or(key))
                        .unwrap()
                        .segments
                })
                .collect();
            new_segment_list.push(seg_id);
            let new_segment_group = SegmentGroup {
                gtype: tile.segments[seg_id.1].stype,
                segments: new_segment_list,
            };
            let key = self.groups.insert(new_segment_group);
            self.group_associations.extend(
                self.groups
                    .get(key)
                    .unwrap()
                    .segments
                    .iter()
                    .map(|seg_id| (*seg_id, key)),
            );
            rewrites.extend(additions.iter().map(|k| (*k, key)));
        }

        // dbg!(&uninserted_segments);
        // create new groups for disconnected tile segments
        for i in uninserted_segments
            .into_iter()
            .enumerate()
            .filter_map(|(i, x)| x.then_some(i))
        {
            let segment = &tile.segments[i];
            let seg_id = (pos, i);
            let key = self.groups.insert(SegmentGroup {
                gtype: segment.stype,
                segments: vec![seg_id],
            });
            self.group_associations.insert(seg_id, key);
        }

        // put tile on board
        self.placed_tiles.insert(pos, tile);

        // dbg!(&self.groups);
        // dbg!(&self.group_associations);

        Ok(())
    }
}

#[test]
fn test_group_coallating() {
    use crate::tile::tile_definitions::*;
    let mut game = Game::new();
    game.place_tile(STRAIGHT_ROAD.clone(), Pos(0, 0)).unwrap();
    game.place_tile(L_CURVE_ROAD.clone().rotated(), Pos(-1, 0))
        .unwrap();
    game.place_tile(CORNER_CITY.clone().rotated(), Pos(0, -1))
        .unwrap();
    game.place_tile(CITY_ENTRANCE.clone(), Pos(-1, -1)).unwrap();
}

#[test]
fn test_group_coallating_2() {
    use crate::tile::tile_definitions::*;
    let mut game = Game::new();
    game.place_tile(STRAIGHT_ROAD.clone(), Pos(0, 0)).unwrap();
    game.place_tile(DEAD_END_ROAD.clone().rotated(), Pos(2, 0))
        .unwrap();
    game.place_tile(STRAIGHT_ROAD.clone(), Pos(1, 0)).unwrap();
}

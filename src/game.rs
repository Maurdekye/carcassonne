use std::collections::{HashMap, HashSet};

use ggez::GameError;
use slotmap::{DefaultKey, SlotMap};

use crate::{
    pos::Pos,
    tile::{get_tile_library, Orientation, SegmentType, Tile},
};

pub type SegmentIdentifier = (Pos, usize);
pub type GroupIdentifier = DefaultKey;
pub type EdgeIdentifier = (Pos, Orientation);

#[derive(Debug)]
pub struct SegmentGroup {
    pub gtype: SegmentType,
    pub segments: Vec<SegmentIdentifier>,
    pub free_edges: HashSet<EdgeIdentifier>,
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
        let mut closing_edges: HashMap<SegmentIdentifier, Orientation> = HashMap::new();
        let mut opening_edges: HashMap<SegmentIdentifier, Orientation> = HashMap::new();
        // dbg!(&self.groups);
        // dbg!(&self.group_associations);

        // evaluate mountings with neighboring tiles
        for (orientation, offset) in Orientation::iter_with_offsets() {
            let adjacent_pos = pos + offset;
            // dbg!(&adjacent_pos);
            let Some(adjacent_tile) = self.placed_tiles.get(&adjacent_pos) else {
                for seg_index in tile.mounts.by_orientation(orientation) {
                    opening_edges.insert((pos, *seg_index), orientation);
                }
                continue;
            };
            let opposing_orientation = orientation.opposite();
            for seg_index in adjacent_tile.mounts.by_orientation(opposing_orientation) {
                closing_edges.insert((adjacent_pos, *seg_index), opposing_orientation);
            }

            // dbg!(&adjacent_tile);

            let Some(mounts) = tile.validate_mounting(adjacent_tile, orientation) else {
                return Err(GameError::CustomError(
                    "Attempt to place invalid tile!".to_string(),
                ));
            };
            // dbg!(&mounts);

            for mount in mounts {
                let seg_ident: SegmentIdentifier = (pos, mount.from_segment);
                let adj_seg_ident: SegmentIdentifier = (adjacent_pos, mount.to_segment);
                // dbg!(&seg_id);
                // dbg!(&adj_seg_id);
                let group_key = self
                    .group_associations
                    .get(&adj_seg_ident)
                    .expect("All placed segments have associated groups");
                new_group_insertions
                    .entry(seg_ident)
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
        for (seg_ident, additions) in singletons {
            self.groups
                .get_mut(additions[0])
                .unwrap()
                .segments
                .push(seg_ident);
            self.group_associations.insert(seg_ident, additions[0]);
        }

        // dbg!(&coallations);
        // combine disconnected groups that were connected together by the new segment
        let mut rewrites = HashMap::new();
        for (seg_ident, additions) in coallations {
            let (old_segments, old_free_edges): (Vec<_>, Vec<_>) = additions
                .iter()
                .map(|key| {
                    let group = self
                        .groups
                        .remove(*rewrites.get(key).unwrap_or(key))
                        .unwrap();
                    (group.segments, group.free_edges)
                })
                .unzip();
            let new_segment_list = old_segments
                .into_iter()
                .flatten()
                .chain([seg_ident])
                .collect();
            let new_free_edges = old_free_edges.into_iter().flatten().collect();
            let new_segment_group = SegmentGroup {
                gtype: tile.segments[seg_ident.1].stype,
                segments: new_segment_list,
                free_edges: new_free_edges,
            };
            let key = self.groups.insert(new_segment_group);
            self.group_associations.extend(
                self.groups
                    .get(key)
                    .unwrap()
                    .segments
                    .iter()
                    .map(|seg_ident| (*seg_ident, key)),
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
            let seg_ident = (pos, i);
            let key = self.groups.insert(SegmentGroup {
                gtype: segment.stype,
                segments: vec![seg_ident],
                free_edges: HashSet::new(),
            });
            self.group_associations.insert(seg_ident, key);
        }

        // update open and closed edges for groups
        for (seg_ident, orientation) in opening_edges {
            let group = self.group_by_seg_ident(seg_ident).unwrap();
            group.free_edges.insert((seg_ident.0, orientation));
        }
        for (seg_ident, orientation) in closing_edges {
            let group = self.group_by_seg_ident(seg_ident).unwrap();
            group.free_edges.remove(&(seg_ident.0, orientation));
            if group.free_edges.is_empty() {
                println!("{:?} group finished!", group.gtype);
            }
        }

        // put tile on board
        self.placed_tiles.insert(pos, tile);

        // dbg!(&self.groups);
        // dbg!(&self.group_associations);

        Ok(())
    }

    fn group_by_seg_ident(&mut self, seg_ident: SegmentIdentifier) -> Option<&mut SegmentGroup> {
        self.group_associations
            .get(&seg_ident)
            .and_then(|key| self.groups.get_mut(*key))
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

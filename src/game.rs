use std::collections::{HashMap, HashSet};

use ggez::GameError;
use player::Player;
use slotmap::{DefaultKey, SlotMap};

use crate::{
    pos::GridPos,
    tile::{get_tile_library, Orientation, Segment, SegmentType, Tile},
    util::{CollectedBag, HashMapBag},
};

pub mod player {
    use ggez::graphics::Color;

    pub struct Player {
        pub meeples: usize,
        pub color: Color,
        pub score: usize,
    }

    impl Player {
        pub fn new(color: Color) -> Player {
            Player {
                meeples: 7,
                color,
                score: 0,
            }
        }
    }
}

pub type SegmentIndex = usize;
pub type SegmentIdentifier = (GridPos, SegmentIndex);
pub type GroupIdentifier = DefaultKey;
pub type EdgeIdentifier = (GridPos, Orientation);
pub type PlayerIdentifier = DefaultKey;
pub type PlacedMeeple = (SegmentIdentifier, PlayerIdentifier);

#[derive(Debug)]
pub struct SegmentGroup {
    pub gtype: SegmentType,
    pub segments: Vec<SegmentIdentifier>,
    pub free_edges: HashSet<EdgeIdentifier>,
    pub meeples: Vec<PlacedMeeple>,
}

impl SegmentGroup {
    fn score(&self) -> usize {
        let tile_span = self
            .segments
            .iter()
            .map(|(pos, _)| *pos)
            .collect::<HashSet<_>>()
            .len();
        let base_score = match self.gtype {
            SegmentType::City => 2,
            SegmentType::Road => 1,
            _ => todo!(),
        };
        base_score * tile_span
    }
}

pub struct Game {
    pub library: Vec<Tile>,
    pub placed_tiles: HashMap<GridPos, Tile>,
    pub groups: SlotMap<GroupIdentifier, SegmentGroup>,
    pub group_associations: HashMap<SegmentIdentifier, GroupIdentifier>,
    pub players: SlotMap<PlayerIdentifier, Player>,
}

impl Game {
    pub fn new() -> Game {
        Game {
            library: get_tile_library(),
            placed_tiles: HashMap::new(),
            groups: SlotMap::new(),
            group_associations: HashMap::new(),
            players: SlotMap::new(),
        }
    }

    pub fn place_tile(
        &mut self,
        tile: Tile,
        pos: GridPos,
    ) -> Result<Vec<GroupIdentifier>, GameError> {
        let mut new_group_insertions: HashMap<SegmentIdentifier, HashSet<GroupIdentifier>> =
            HashMap::new();
        let mut uninserted_segments: Vec<_> = (0..tile.segments.len()).map(|_| true).collect();
        let mut closing_edges: HashMap<SegmentIdentifier, Vec<Orientation>> = HashMap::new();
        let mut opening_edges: HashMap<SegmentIdentifier, Vec<Orientation>> = HashMap::new();

        // evaluate mountings with neighboring tiles
        for (orientation, offset) in Orientation::iter_with_offsets() {
            let adjacent_pos = pos + offset;
            let Some(adjacent_tile) = self.placed_tiles.get(&adjacent_pos) else {
                for seg_index in tile.mounts.by_orientation(orientation) {
                    opening_edges.place((pos, *seg_index), orientation);
                }
                continue;
            };
            let opposing_orientation = orientation.opposite();
            for seg_index in adjacent_tile.mounts.by_orientation(opposing_orientation) {
                closing_edges.place((adjacent_pos, *seg_index), opposing_orientation);
            }

            let Some(mounts) = tile.validate_mounting(adjacent_tile, orientation) else {
                return Err(GameError::CustomError(
                    "Attempt to place invalid tile!".to_string(),
                ));
            };

            for mount in mounts {
                let seg_ident: SegmentIdentifier = (pos, mount.from_segment);
                let adj_seg_ident: SegmentIdentifier = (adjacent_pos, mount.to_segment);
                let group_key = self
                    .group_associations
                    .get(&adj_seg_ident)
                    .expect("All placed segments have associated groups");
                new_group_insertions.place(seg_ident, *group_key);
                uninserted_segments[mount.from_segment] = false;
            }
        }

        // insert segments into existing connected groups
        let (singletons, coallations): (Vec<_>, Vec<_>) = new_group_insertions
            .into_iter()
            .partition(|(_, additions)| additions.len() == 1);

        // attach segments to existing groups
        for (seg_ident, additions) in singletons {
            let group_key = additions.into_iter().next().unwrap();
            self.groups
                .get_mut(group_key)
                .unwrap()
                .segments
                .push(seg_ident);
            self.group_associations.insert(seg_ident, group_key);
        }

        // combine disconnected groups that were connected together by the new segment
        let mut rewrites = HashMap::new();
        for (seg_ident, additions) in coallations {
            let (old_segments, (old_free_edges, old_meeples)): (Vec<_>, (Vec<_>, Vec<_>)) =
                additions
                    .iter()
                    .map(|key| {
                        let group = self
                            .groups
                            .remove(*rewrites.get(key).unwrap_or(key))
                            .unwrap();
                        (group.segments, (group.free_edges, group.meeples))
                    })
                    .unzip();
            let segments = old_segments
                .into_iter()
                .flatten()
                .chain([seg_ident])
                .collect();
            let free_edges = old_free_edges.into_iter().flatten().collect();
            let meeples = old_meeples.into_iter().flatten().collect();
            let new_segment_group = SegmentGroup {
                gtype: tile.segments[seg_ident.1].stype,
                segments,
                free_edges,
                meeples,
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
                meeples: Vec::new(),
            });
            self.group_associations.insert(seg_ident, key);
        }

        // update open and closed edges for groups
        for (seg_ident, orientations) in opening_edges {
            let group = self.group_by_seg_ident(seg_ident).unwrap();
            group.free_edges.extend(
                orientations
                    .into_iter()
                    .map(|orientation| (seg_ident.0, orientation)),
            );
        }
        let mut closing_groups = Vec::new();
        for (seg_ident, orientations) in closing_edges {
            let (group, group_ident) = self.group_and_key_by_seg_ident(seg_ident).unwrap();
            for orientation in orientations {
                group.free_edges.remove(&(seg_ident.0, orientation));
            }
            if group.free_edges.is_empty() {
                closing_groups.push(group_ident);
            }
        }

        // put tile on board
        self.placed_tiles.insert(pos, tile);

        Ok(closing_groups)
    }

    fn group_by_seg_ident(&mut self, seg_ident: SegmentIdentifier) -> Option<&mut SegmentGroup> {
        self.group_and_key_by_seg_ident(seg_ident).map(|(g, _)| g)
    }

    fn group_and_key_by_seg_ident(
        &mut self,
        seg_ident: SegmentIdentifier,
    ) -> Option<(&mut SegmentGroup, GroupIdentifier)> {
        self.group_associations
            .get(&seg_ident)
            .and_then(|key| self.groups.get_mut(*key).map(|group| (group, *key)))
    }

    pub fn segment_by_ident(&self, (grid_pos, seg_index): SegmentIdentifier) -> Option<&Segment> {
        self.placed_tiles
            .get(&grid_pos)
            .and_then(|tile| tile.segments.get(seg_index))
    }

    pub fn score_group(&mut self, group_ident: GroupIdentifier) {
        let group = self.groups.get_mut(group_ident).unwrap();
        match group.gtype {
            SegmentType::City | SegmentType::Road => {
                // compute individual player scores
                let CollectedBag(meeples_by_player) =
                    group.meeples.iter().map(|&(k, v)| (v, k)).collect();
                let highest_count = meeples_by_player.values().map(Vec::len).max().unwrap();
                let scoring_players: Vec<_> = meeples_by_player
                    .iter()
                    .filter_map(|(player_ident, meeples)| {
                        (meeples.len() == highest_count).then_some(*player_ident)
                    })
                    .collect();
                let group_score = group.score();
                let per_player_score = group_score / scoring_players.len();
                let score_remainder = group_score % scoring_players.len();
                for (i, player_ident) in scoring_players.into_iter().enumerate() {
                    let player = self.players.get_mut(player_ident).unwrap();
                    player.score += per_player_score;
                    if i < score_remainder {
                        player.score += 1;
                    }
                }

                // return and remove meeples
                for (player_ident, meeples) in meeples_by_player {
                    let player = self.players.get_mut(player_ident).unwrap();
                    player.meeples += meeples.len();
                }

                group.meeples.clear();
            }
            _ => {}
        }
    }

    pub fn place_meeple(
        &mut self,
        seg_ident: SegmentIdentifier,
        player_ident: PlayerIdentifier,
    ) -> Result<(), GameError> {
        let player = self.players.get_mut(player_ident).unwrap();
        if player.meeples == 0 {
            return Err(GameError::CustomError(
                "Player has no meeples remaining!".to_string(),
            ));
        }
        player.meeples -= 1;

        let group = self.group_by_seg_ident(seg_ident).unwrap();
        group.meeples.push((seg_ident, player_ident));

        Ok(())
    }
}

#[test]
fn test_group_coallating() {
    use crate::tile::tile_definitions::*;
    let mut game = Game::new();
    game.place_tile(STRAIGHT_ROAD.clone(), GridPos(0, 0))
        .unwrap();
    game.place_tile(L_CURVE_ROAD.clone().rotated(), GridPos(-1, 0))
        .unwrap();
    game.place_tile(CORNER_CITY.clone().rotated(), GridPos(0, -1))
        .unwrap();
    game.place_tile(CITY_ENTRANCE.clone(), GridPos(-1, -1))
        .unwrap();
}

#[test]
fn test_group_coallating_2() {
    use crate::tile::tile_definitions::*;
    let mut game = Game::new();
    game.place_tile(STRAIGHT_ROAD.clone(), GridPos(0, 0))
        .unwrap();
    game.place_tile(DEAD_END_ROAD.clone().rotated(), GridPos(2, 0))
        .unwrap();
    game.place_tile(STRAIGHT_ROAD.clone(), GridPos(1, 0))
        .unwrap();
}

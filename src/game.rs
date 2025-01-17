use std::{
    collections::{HashMap, HashSet, VecDeque},
    convert::identity,
};

use ggez::{glam::Vec2, graphics::Rect, GameError, GameResult};
use player::Player;
use slotmap::{DefaultKey, SlotMap};

use crate::{
    pos::GridPos,
    tile::{
        get_tile_library, GridBorderCoordinate, GridBorderCoordinateOffset, Opposite, Orientation,
        Segment, SegmentBorderPiece, SegmentType, Tile,
    },
    util::{refit_to_rect, Bag, HashMapBag, MapFindExt},
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
    pub outline: Option<Vec<Vec<Vec2>>>,
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
            SegmentType::City if !self.free_edges.is_empty() => 1,
            SegmentType::City => 2,
            SegmentType::Road => 1,
            _ => 0,
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
            let group = self.groups.get_mut(group_key).unwrap();
            group.segments.push(seg_ident);
            group.outline = None;
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
                outline: None,
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
                outline: None,
            });
            self.group_associations.insert(seg_ident, key);
        }

        // update open and closed edges for groups
        for (seg_ident, orientations) in opening_edges {
            let group = self.group_by_seg_ident_mut(seg_ident).unwrap();
            group.free_edges.extend(
                orientations
                    .into_iter()
                    .map(|orientation| (seg_ident.0, orientation)),
            );
        }
        let mut closing_groups = Vec::new();
        for (seg_ident, orientations) in closing_edges {
            let (group, group_ident) = self.group_and_key_by_seg_ident_mut(seg_ident).unwrap();
            for orientation in orientations {
                group.free_edges.remove(&(seg_ident.0, orientation));
            }
            if group.free_edges.is_empty() {
                closing_groups.push(group_ident);
            }
        }

        // put tile on board
        self.placed_tiles.insert(pos, tile);

        // check for completed monastaries
        for adjacent_pos in pos.surrounding() {
            let Some(tile) = self.placed_tiles.get(&adjacent_pos) else {
                continue;
            };
            let Some((seg_index, _)) = tile
                .segments
                .iter()
                .enumerate()
                .find(|(_, seg)| seg.stype == SegmentType::Monastary)
            else {
                continue;
            };
            if adjacent_pos
                .surrounding()
                .all(|monastary_adjacent| self.placed_tiles.contains_key(&monastary_adjacent))
            {
                closing_groups.push(
                    *self
                        .group_associations
                        .get(&(adjacent_pos, seg_index))
                        .unwrap(),
                );
            }
        }

        Ok(closing_groups)
    }

    fn group_by_seg_ident_mut(
        &mut self,
        seg_ident: SegmentIdentifier,
    ) -> Option<&mut SegmentGroup> {
        self.group_and_key_by_seg_ident_mut(seg_ident)
            .map(|(g, _)| g)
    }

    pub fn group_and_key_by_seg_ident(
        &self,
        seg_ident: SegmentIdentifier,
    ) -> Option<(&SegmentGroup, GroupIdentifier)> {
        self.group_associations
            .get(&seg_ident)
            .and_then(|key| self.groups.get(*key).map(|group| (group, *key)))
    }

    fn group_and_key_by_seg_ident_mut(
        &mut self,
        seg_ident: SegmentIdentifier,
    ) -> Option<(&mut SegmentGroup, GroupIdentifier)> {
        self.group_associations
            .get(&seg_ident)
            .and_then(|key| self.groups.get_mut(*key).map(|group| (group, *key)))
    }

    #[allow(unused)]
    pub fn segment_by_ident(&self, (grid_pos, seg_index): SegmentIdentifier) -> Option<&Segment> {
        self.placed_tiles
            .get(&grid_pos)
            .and_then(|tile| tile.segments.get(seg_index))
    }

    pub fn score_group(&mut self, group_ident: GroupIdentifier) {
        let group = self.groups.get(group_ident).unwrap();

        // determine which players are earning score for the group
        let Bag(meeples_by_player) = group.meeples.iter().map(|&(k, v)| (v, k)).collect();
        let Some(highest_count) = meeples_by_player.values().map(Vec::len).max() else {
            // nobody placed any meeples on the group
            return;
        };
        let scoring_players: Vec<_> = meeples_by_player
            .iter()
            .filter_map(|(player_ident, meeples)| {
                (meeples.len() == highest_count).then_some(*player_ident)
            })
            .collect();

        let group_score = match group.gtype {
            SegmentType::City | SegmentType::Road => group.score(),
            SegmentType::Field => {
                let mut cities = HashSet::new();
                for (pos, seg_index) in group.segments.clone() {
                    let tile = self.placed_tiles.get(&pos).unwrap();
                    for adj_seg_index in tile
                        .adjacent_segments(seg_index)
                        .filter_map(|(i, seg)| (seg.stype == SegmentType::City).then_some(i))
                        .collect::<Vec<_>>()
                    {
                        let (city_group, city_group_ident) = self
                            .group_and_key_by_seg_ident((pos, adj_seg_index))
                            .unwrap();
                        if city_group.free_edges.is_empty() {
                            cities.insert(city_group_ident);
                        }
                    }
                }
                cities.len() * 3
            }
            SegmentType::Monastary => {
                group
                    .segments
                    .first()
                    .unwrap()
                    .0
                    .surrounding()
                    .filter(|pos| self.placed_tiles.contains_key(&pos))
                    .count()
                    + 1
            }
            SegmentType::Village => 0,
        };

        for player_ident in scoring_players {
            let player = self.players.get_mut(player_ident).unwrap();
            player.score += group_score;
        }

        // return and remove meeples
        for (player_ident, meeples) in meeples_by_player {
            let player = self.players.get_mut(player_ident).unwrap();
            player.meeples += meeples.len();
        }

        self.groups.get_mut(group_ident).unwrap().meeples.clear();
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

        let group = self.group_by_seg_ident_mut(seg_ident).unwrap();
        group.meeples.push((seg_ident, player_ident));

        Ok(())
    }

    pub fn get_group_outline(&mut self, group_ident: GroupIdentifier) -> Option<&Vec<Vec<Vec2>>> {
        if self.groups.get(group_ident)?.outline.is_none() {
            let outline = self.compute_group_outline(group_ident)?;
            let group = self.groups.get_mut(group_ident)?;
            group.outline = Some(outline);
            group.outline.as_ref()
        } else {
            self.groups.get(group_ident)?.outline.as_ref()
        }
    }

    fn compute_group_outline(&self, group_ident: GroupIdentifier) -> Option<Vec<Vec<Vec2>>> {
        // collect all edges by their grid position
        let group = self.groups.get(group_ident)?;
        let Bag(edges_by_gridpos) = group
            .segments
            .iter()
            .copied()
            .flat_map(|seg_ident| {
                let (tile_pos, seg_index) = seg_ident;
                let tile = self.placed_tiles.get(&tile_pos).unwrap();
                tile.segments[seg_index]
                    .edge_definition
                    .iter()
                    .copied()
                    .filter_map(move |edge| match edge {
                        SegmentBorderPiece::Edge(edge) => Some((tile_pos, edge)),
                        _ => None,
                    })
            })
            .collect();
        // dbg!(&edges_by_gridpos);

        #[derive(Debug, Clone, Copy)]
        enum LinePiece {
            Vert(Vec2),
            BorderCoordinate(GridBorderCoordinate),
        }

        impl PartialEq for LinePiece {
            fn eq(&self, other: &Self) -> bool {
                match (self, other) {
                    (Self::BorderCoordinate(l0), Self::BorderCoordinate(r0)) => l0 == r0,
                    _ => false,
                }
            }
        }

        impl From<LinePiece> for Vec2 {
            fn from(piece: LinePiece) -> Self {
                match piece {
                    LinePiece::Vert(vert) => vert,
                    LinePiece::BorderCoordinate(coord) => coord.into(),
                }
            }
        }

        // collect all lines together by their grid position
        let line_segments_iter = group.segments.iter().copied().flat_map(|seg_ident| {
            // dbg!(&seg_ident);
            let (tile_pos, seg_index) = seg_ident;
            let tile = self.placed_tiles.get(&tile_pos).unwrap();

            let mut pieces: Vec<Option<LinePiece>> = vec![];
            for &edge in &tile.segments[seg_index].edge_definition {
                // dbg!(edge);
                use SegmentBorderPiece::*;
                match edge {
                    Break => {
                        pieces.push(None);
                    }
                    Vert(index) => {
                        pieces.push(Some(LinePiece::Vert(
                            tile.verts[index] + Vec2::from(tile_pos),
                        )));
                    }
                    Edge(edge) => {
                        let (span, orientation) = edge;
                        // dbg!((tile_pos, span.start(), orientation));
                        let start = LinePiece::BorderCoordinate(
                            GridBorderCoordinate::from_tile_edge_vertex(
                                tile_pos,
                                (span.start(), orientation),
                            ),
                        );
                        // dbg!(&start);
                        if pieces.last() != Some(&Some(start)) {
                            pieces.push(Some(start));
                        }

                        // this breaks rustfmt if its a matches! macro
                        if match edges_by_gridpos.get(&(tile_pos + orientation.offset())) {
                            Some(adj_edges) if (adj_edges.contains(&edge.opposite())) => true,
                            _ => false,
                        } {
                            pieces.push(None);
                        }

                        let end = LinePiece::BorderCoordinate(
                            GridBorderCoordinate::from_tile_edge_vertex(
                                tile_pos,
                                (span.end(), orientation),
                            ),
                        );
                        // dbg!(&end);
                        pieces.push(Some(end));
                    }
                }
                // dbg!(&pieces);
            }

            let mut lines: Vec<_> = pieces
                .split(Option::is_none)
                .map(|lines| lines.iter().copied().flatten().collect::<Vec<_>>())
                .collect();

            // dbg!(&lines);

            let closed_loop = lines.len() == 1;
            if !closed_loop {
                let first_line = lines.remove(0);
                let last_line = lines.last_mut().unwrap();
                if first_line.first() == last_line.last() {
                    last_line.extend(first_line.into_iter().skip(1));
                } else {
                    last_line.extend(first_line);
                }
            } else {
                let first_piece = lines[0][0];
                lines[0].push(first_piece);
            }

            // dbg!(&lines);

            lines.into_iter().filter_map(move |line| {
                let closed_loop = closed_loop
                    || !matches!(
                        (line.first(), line.last()),
                        (
                            Some(LinePiece::BorderCoordinate(_)),
                            Some(LinePiece::BorderCoordinate(_))
                        )
                    );
                (line.len() > 1).then_some((tile_pos, (line, closed_loop)))
            })
        });
        let mut closed_loops = Vec::new();
        let mut lines_by_gridpos = HashMap::new();
        for (gridpos, (line, closed_loop)) in line_segments_iter {
            if closed_loop {
                closed_loops.push(line.into());
            } else {
                lines_by_gridpos.place(gridpos, line);
            }
        }
        // dbg!(&closed_loops);
        // dbg!(&lines_by_gridpos);

        // connect all lines together
        let mut polylines: Vec<VecDeque<LinePiece>> = closed_loops;
        let mut current_line: VecDeque<LinePiece> = VecDeque::new();
        loop {
            // dbg!(&current_line);
            match (current_line.front(), current_line.back()) {
                (None, None) => {
                    if let Some(next_line) = lines_by_gridpos
                        .values_mut()
                        .find(|lines| !lines.is_empty())
                        .and_then(|lines| lines.pop())
                    {
                        current_line = next_line.into();
                    } else {
                        break;
                    }
                }
                (
                    Some(LinePiece::BorderCoordinate(start)),
                    Some(LinePiece::BorderCoordinate(end)),
                ) => {
                    // dbg!((&start, &end));
                    if start == end {
                        // segment is completed, add to polylines
                        polylines.push(current_line);
                        current_line = VecDeque::new();
                        // dbg!(&polylines);
                    } else {
                        // locate a segment to attach to the end
                        let search_result = 'line_search: {
                            for adjacent in end.get_adjacent_gridposes() {
                                let Some(lines) = lines_by_gridpos.get_mut(&adjacent) else {
                                    continue;
                                };
                                let Some((i, _)) = lines.iter().enumerate().find(|(_, line)| {
                                    line.first() == Some(&LinePiece::BorderCoordinate(*end))
                                }) else {
                                    continue;
                                };
                                break 'line_search Some(lines.remove(i));
                            }
                            None
                        };
                        // dbg!(&search_result);
                        if let Some(new_line) = search_result {
                            current_line.extend(new_line.into_iter().skip(1));
                            continue;
                        }

                        // locate a segment to attach to the beginning
                        let search_result = 'line_search: {
                            for adjacent in start.get_adjacent_gridposes() {
                                let Some(lines) = lines_by_gridpos.get_mut(&adjacent) else {
                                    continue;
                                };
                                let Some((i, _)) = lines.iter().enumerate().find(|(_, line)| {
                                    line.last() == Some(&LinePiece::BorderCoordinate(*start))
                                }) else {
                                    continue;
                                };
                                break 'line_search Some(lines.remove(i));
                            }
                            None
                        };
                        // dbg!(&search_result);
                        if let Some(new_line) = search_result {
                            current_line = new_line
                                .into_iter()
                                .chain(current_line.into_iter().skip(1))
                                .collect();
                            continue;
                        }

                        // no line segments remaining, just place the incomplete line into polylines (and print a warning)
                        polylines.push(current_line);
                        current_line = VecDeque::new();
                        // dbg!(&polylines);
                        eprintln!("Incomplete line segment created!");
                    }
                }
                _ => unimplemented!("non-border coordinate capped lines"),
            }
        }
        // dbg!(&polylines);

        let final_lines_set = polylines
            .into_iter()
            .map(|polyline| polyline.into_iter().map(Vec2::from).collect())
            .collect();

        // dbg!(&final_lines_set);

        Some(final_lines_set)
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
pub fn test_group_outline_generation() -> GameResult {
    use crate::tile::tile_definitions::CORNER_CITY;
    use crate::tile::SegmentType;
    let mut game = Game::new();
    game.place_tile(CORNER_CITY.clone(), GridPos(1, 1))?;
    game.place_tile(CORNER_CITY.clone().rotated(), GridPos(0, 1))?;
    game.place_tile(CORNER_CITY.clone().rotated().rotated(), GridPos(0, 0))?;
    game.place_tile(
        CORNER_CITY.clone().rotated().rotated().rotated(),
        GridPos(1, 0),
    )?;
    let city_group_ident = game
        .groups
        .iter()
        .find_map(|(group_ident, group)| (group.gtype == SegmentType::City).then_some(group_ident))
        .unwrap();
    let outline = game.compute_group_outline(city_group_ident);
    // dbg!(outline);
    Ok(())
}

#[test]
pub fn test_group_outline_generation_2() -> GameResult {
    use crate::tile::tile_definitions::{L_CURVE_ROAD, STRAIGHT_ROAD};
    use crate::tile::SegmentType;
    let mut game = Game::new();
    game.place_tile(STRAIGHT_ROAD.clone(), GridPos(0, 0))?;
    game.place_tile(L_CURVE_ROAD.clone(), GridPos(1, 0))?;
    game.place_tile(STRAIGHT_ROAD.clone().rotated(), GridPos(1, -1))?;
    let group_ident = game
        .groups
        .iter()
        .map_find(|(group_ident, group)| (group.gtype == SegmentType::Road).then_some(group_ident))
        .unwrap();
    let outline = game.compute_group_outline(group_ident);
    // dbg!(outline);
    Ok(())
}

#[test]
pub fn test_group_outline_generation_3() -> GameResult {
    use crate::tile::tile_definitions::MONASTARY;
    use crate::tile::SegmentType;
    let mut game = Game::new();
    game.place_tile(MONASTARY.clone(), GridPos(0, 0))?;
    let group_ident = game
        .groups
        .iter()
        .map_find(|(group_ident, group)| (group.gtype == SegmentType::Field).then_some(group_ident))
        .unwrap();
    let outline = game.compute_group_outline(group_ident);
    // dbg!(outline);
    Ok(())
}

#[test]
pub fn test_monastary_scoring() -> GameResult {
    use crate::tile::tile_definitions::{MONASTARY, _DEBUG_EMPTY_FIELD};
    let mut game = Game::new();
    game.place_tile(MONASTARY.clone(), GridPos(0, 0))?;
    game.place_tile(_DEBUG_EMPTY_FIELD.clone(), GridPos(0, 1))?;
    game.place_tile(_DEBUG_EMPTY_FIELD.clone(), GridPos(1, 0))?;
    game.place_tile(_DEBUG_EMPTY_FIELD.clone(), GridPos(1, 1))?;
    game.place_tile(_DEBUG_EMPTY_FIELD.clone(), GridPos(1, -1))?;
    game.place_tile(_DEBUG_EMPTY_FIELD.clone(), GridPos(0, -1))?;
    game.place_tile(_DEBUG_EMPTY_FIELD.clone(), GridPos(-1, -1))?;
    game.place_tile(_DEBUG_EMPTY_FIELD.clone(), GridPos(-1, 0))?;
    let closing_tiles = game.place_tile(_DEBUG_EMPTY_FIELD.clone(), GridPos(-1, 1))?;
    assert!(!closing_tiles.is_empty());
    Ok(())
}

use std::{
    collections::{HashMap, HashSet, VecDeque},
    num::NonZero,
};

use discord_sdk::activity::{ActivityBuilder, ActivityKind, Assets, Button, PartyPrivacy};
use ggez::{
    glam::{vec2, Vec2},
    graphics::Color,
    GameError,
};
use log::warn;
use player::{Player, PlayerType};
use serde::{Deserialize, Serialize};
use slotmap::{DefaultKey, SlotMap};

use crate::{
    game_client::NUM_PLAYERS,
    pos::GridPos,
    tile::{
        GridBorderCoordinate, Opposite, Orientation, Segment, SegmentAttribute, SegmentBorderPiece,
        SegmentType, Tile,
    },
    util::{Bag, HashMapBag, MinByF32Key},
    LATEST_RELEASE_LINK,
};
use ggez_no_re::line::Line;

pub mod player {
    use std::{net::IpAddr, time::Duration};

    use ggez::graphics::Color;
    use serde::{Deserialize, Serialize};

    #[derive(Copy, Clone, Debug, Serialize, Deserialize)]
    pub enum ConnectionState {
        Disconnected,
        Connected { latency: Option<Duration> },
    }

    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub enum PlayerType {
        Local,
        MultiplayerHost {
            username: String,
        },
        MultiplayerClient {
            username: String,
            address: IpAddr,
            connection_state: ConnectionState,
        },
    }

    impl PlayerType {
        pub fn from_details(username: String, address: Option<IpAddr>) -> PlayerType {
            match address {
                Some(address) => PlayerType::MultiplayerClient {
                    username,
                    address,
                    connection_state: ConnectionState::Connected { latency: None },
                },
                None => PlayerType::MultiplayerHost { username },
            }
        }

        pub fn matches_address(&self, address: Option<IpAddr>) -> bool {
            match (&self, address) {
                (PlayerType::MultiplayerHost { .. }, None) => true,
                (PlayerType::MultiplayerClient { address, .. }, Some(check_address)) => {
                    address == &check_address
                }
                _ => false,
            }
        }
    }

    impl PartialEq for PlayerType {
        fn eq(&self, other: &Self) -> bool {
            match (self, other) {
                (
                    Self::MultiplayerClient {
                        address: l_address, ..
                    },
                    Self::MultiplayerClient {
                        address: r_address, ..
                    },
                ) => l_address == r_address,
                _ => core::mem::discriminant(self) == core::mem::discriminant(other),
            }
        }
    }

    #[derive(Clone, Serialize, Deserialize, Debug)]
    pub struct Player {
        pub meeples: usize,
        pub score: usize,
        pub color: Color,
        pub ptype: PlayerType,
    }

    impl Player {
        pub fn new(color: Color) -> Player {
            Player::new_inner(color, PlayerType::Local)
        }

        pub fn new_inner(color: Color, ptype: PlayerType) -> Player {
            Player {
                meeples: 7,
                score: 0,
                color,
                ptype,
            }
        }
    }
}

pub mod debug_game_configs;

pub type SegmentIndex = usize;
pub type SegmentIdentifier = (GridPos, SegmentIndex);
pub type GroupIdentifier = DefaultKey;
pub type EdgeIdentifier = (GridPos, Orientation);
pub type PlayerIdentifier = DefaultKey;
pub type PlacedMeeple = (SegmentIdentifier, PlayerIdentifier);

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScoringDetails {
    pub score: usize,
    pub owners: Vec<(PlayerIdentifier, Color)>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ShapeDetails {
    pub outline: Vec<Line>,
    pub popup_location: Vec2,
}

impl From<Vec<Line>> for ShapeDetails {
    fn from(outline: Vec<Line>) -> Self {
        let all_verts: Vec<_> = outline.iter().flatten().collect();
        let top_edge = all_verts.iter().min_by_f32_key(|v| v.y).unwrap().y;
        let middle: f32 = all_verts.iter().map(|v| v.x).sum();
        let middle = middle / all_verts.len() as f32;
        let popup_location = vec2(middle, top_edge);
        ShapeDetails {
            outline,
            popup_location,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SegmentGroup {
    pub gtype: SegmentType,
    pub segments: Vec<SegmentIdentifier>,
    pub free_edges: HashSet<EdgeIdentifier>,
    pub meeples: Vec<PlacedMeeple>,
    pub scoring_details: Option<ScoringDetails>,
    pub shape_details: Option<ShapeDetails>,
}

#[allow(clippy::type_complexity)]
impl SegmentGroup {
    fn compute_owners(
        &self,
    ) -> Option<(
        HashMap<PlayerIdentifier, Vec<(GridPos, usize)>>,
        Vec<PlayerIdentifier>,
    )> {
        let Bag(meeples_by_player) = self.meeples.iter().map(|&(k, v)| (v, k)).collect();
        let Some(highest_count) = meeples_by_player.values().map(Vec::len).max() else {
            // nobody placed any meeples on the group
            return None;
        };
        let scoring_players: Vec<_> = meeples_by_player
            .iter()
            .filter_map(|(player_ident, meeples)| {
                (meeples.len() == highest_count).then_some(*player_ident)
            })
            .collect();
        Some((meeples_by_player, scoring_players))
    }
}

pub struct ScoringResult {
    pub meeple_location: Vec2,
    pub meeple_color: Color,
    pub score: usize,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Game {
    pub local_player: PlayerType,
    pub library: Vec<Tile>,
    pub placed_tiles: HashMap<GridPos, Tile>,
    pub groups: SlotMap<GroupIdentifier, SegmentGroup>,
    pub group_associations: HashMap<SegmentIdentifier, GroupIdentifier>,
    pub players: SlotMap<PlayerIdentifier, Player>,
    pub valid_placements: Vec<GridPos>,
}

impl Game {
    pub fn new() -> Game {
        Game::new_with_library(Tile::default_library())
    }

    pub fn new_with_library(library: Vec<Tile>) -> Game {
        Game::new_inner(library, PlayerType::Local)
    }

    pub fn new_inner(library: Vec<Tile>, local_player: PlayerType) -> Game {
        Game {
            local_player,
            library,
            placed_tiles: HashMap::new(),
            groups: SlotMap::new(),
            group_associations: HashMap::new(),
            players: SlotMap::new(),
            valid_placements: Vec::new(),
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
            group.shape_details = None;
            group.scoring_details = None;
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
                scoring_details: None,
                shape_details: None,
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
                scoring_details: None,
                shape_details: None,
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
        for adjacent_pos in pos.surrounding().chain([pos]) {
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

        // update valid placements
        self.valid_placements.retain(|p| *p != pos);
        self.valid_placements.extend(
            pos.adjacent()
                .filter(|pos| !self.placed_tiles.contains_key(pos)),
        );

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

    pub fn segment_by_ident(&self, (grid_pos, seg_index): SegmentIdentifier) -> Option<&Segment> {
        self.placed_tiles
            .get(&grid_pos)
            .and_then(|tile| tile.segments.get(seg_index))
    }

    pub fn score_group(&mut self, group_ident: GroupIdentifier) -> Vec<ScoringResult> {
        let mut scoring_result = Vec::new();
        let group = self.groups.get(group_ident).unwrap();

        // determine which players are earning score for the group
        let (meeples_by_player, scoring_players) = group.compute_owners().unwrap_or_default();

        let group_score = self.compute_group_score(group);

        for player_ident in &scoring_players {
            let player = self.players.get_mut(*player_ident).unwrap();
            player.score += group_score;
        }

        // return and remove meeples
        for (player_ident, meeples) in meeples_by_player {
            let first_meeple = meeples.first().unwrap();
            let player = self.players.get_mut(player_ident).unwrap();
            player.meeples += meeples.len();
            let color = player.color;
            scoring_result.push(ScoringResult {
                meeple_location: self.segment_by_ident(*first_meeple).unwrap().meeple_spot
                    + Vec2::from(first_meeple.0),
                meeple_color: color,
                score: if scoring_players.contains(&player_ident) {
                    group_score
                } else {
                    0
                },
            });
        }

        // update group scoring details
        let group = self.groups.get_mut(group_ident).unwrap();
        group.meeples.clear();
        group.scoring_details = Some(ScoringDetails {
            score: group_score,
            owners: scoring_players
                .into_iter()
                .map(|id| (id, self.players.get(id).unwrap().color))
                .collect(),
        });

        // if scoring a city, invalidate the scoring details of all connected farms
        if group.gtype == SegmentType::City {
            for (pos, seg_index) in group.segments.clone() {
                let tile = self.placed_tiles.get(&pos).unwrap();
                for adj_seg_index in tile
                    .adjacent_segments(seg_index)
                    .filter_map(|(i, seg)| (seg.stype == SegmentType::Farm).then_some(i))
                    .collect::<Vec<_>>()
                {
                    let farm_group = self.group_by_seg_ident_mut((pos, adj_seg_index)).unwrap();
                    farm_group.scoring_details = None;
                }
            }
        }

        scoring_result
    }

    fn compute_group_score(&self, group: &SegmentGroup) -> usize {
        match group.gtype {
            SegmentType::City | SegmentType::Road => {
                let Bag(tile_scores) = group
                    .segments
                    .iter()
                    .map(|seg_ident| {
                        let (pos, _) = seg_ident;
                        if self
                            .segment_by_ident(*seg_ident)
                            .unwrap()
                            .attributes
                            .iter()
                            .any(|a| matches!(a, SegmentAttribute::Fortified { .. }))
                        {
                            (pos, 2)
                        } else {
                            (pos, 1)
                        }
                    })
                    .collect();
                let tile_span: usize = tile_scores
                    .values()
                    .flat_map(|scores| scores.iter().max())
                    .sum();
                let base_score = match group.gtype {
                    SegmentType::City if !group.free_edges.is_empty() => 1,
                    SegmentType::City => 2,
                    SegmentType::Road => 1,
                    _ => unimplemented!("unimplemented segment type"),
                };
                base_score * tile_span
            }
            SegmentType::Farm => {
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
                    .filter(|pos| self.placed_tiles.contains_key(pos))
                    .count()
                    + 1
            }
            SegmentType::Village | SegmentType::River => 0,
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

        let group = self.group_by_seg_ident_mut(seg_ident).unwrap();
        group.meeples.push((seg_ident, player_ident));

        Ok(())
    }

    pub fn get_group_scoring_details(
        &mut self,
        group_ident: GroupIdentifier,
    ) -> Option<&ScoringDetails> {
        let group = self.groups.get(group_ident)?;
        if group.scoring_details.is_none() {
            let scoring_details = self.compute_group_scoring_details(group);
            let group = self.groups.get_mut(group_ident)?;
            group.scoring_details = Some(scoring_details);
        }
        self.groups
            .get(group_ident)
            .unwrap()
            .scoring_details
            .as_ref()
    }

    fn compute_group_scoring_details(&self, group: &SegmentGroup) -> ScoringDetails {
        let group_score = self.compute_group_score(group);
        let (_, scoring_players) = group.compute_owners().unwrap_or_default();
        ScoringDetails {
            score: group_score,
            owners: scoring_players
                .into_iter()
                .map(|id| (id, self.players.get(id).unwrap().color))
                .collect(),
        }
    }

    pub fn get_group_shape_details(
        &mut self,
        group_ident: GroupIdentifier,
    ) -> Option<&ShapeDetails> {
        let group = self.groups.get(group_ident)?;
        if group.shape_details.is_none() {
            let outline = self.compute_group_outline(group);
            let group = self.groups.get_mut(group_ident)?;
            group.shape_details = Some(outline.into());
        }
        self.groups.get(group_ident).unwrap().shape_details.as_ref()
    }

    fn compute_group_outline(&self, group: &SegmentGroup) -> Vec<Line> {
        // collect all edges by their grid position
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
        // sdbg!&edges_by_gridpos);

        #[derive(Clone, Copy)]
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

        impl std::fmt::Debug for LinePiece {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    Self::Vert(Vec2 { x, y }) => write!(f, "({x}, {y})"),
                    Self::BorderCoordinate(coord) => write!(f, "{coord:?}"),
                }
            }
        }

        // collect all lines together by their grid position
        let line_segments_iter = group.segments.iter().copied().flat_map(|seg_ident| {
            // sdgb!(&seg_ident);
            let (tile_pos, seg_index) = seg_ident;
            let tile = self.placed_tiles.get(&tile_pos).unwrap();

            let mut pieces: Vec<Option<LinePiece>> = vec![];
            for &edge in &tile.segments[seg_index].edge_definition {
                // sdgb!(edge);
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
                        // sdgb!((tile_pos, span.start(), orientation));
                        let start = LinePiece::BorderCoordinate(
                            GridBorderCoordinate::from_tile_edge_vertex(
                                tile_pos,
                                tile.encode_edge_vertex(span.start(), orientation),
                            ),
                        );
                        // sdgb!(&start);
                        if pieces.last() != Some(&Some(start)) {
                            pieces.push(Some(start));
                        }

                        // this breaks rustfmt if its a matches! macro
                        #[allow(clippy::match_like_matches_macro)]
                        if match edges_by_gridpos.get(&(tile_pos + orientation.offset())) {
                            Some(adj_edges) if (adj_edges.contains(&edge.opposite())) => true,
                            _ => false,
                        } {
                            pieces.push(None);
                        }

                        let end = LinePiece::BorderCoordinate(
                            GridBorderCoordinate::from_tile_edge_vertex(
                                tile_pos,
                                tile.encode_edge_vertex(span.end(), orientation),
                            ),
                        );
                        // sdgb!(&end);
                        pieces.push(Some(end));
                    }
                }
                // sdgb!(&pieces);
            }

            let mut lines: Vec<_> = pieces
                .split(Option::is_none)
                .map(|lines| lines.iter().copied().flatten().collect::<Vec<_>>())
                .collect();

            // sdgb!(&lines);

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

            // sdgb!(&lines);

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
        // sdgb!(&closed_loops);
        // for (GridPos(x, y), len) in lines_by_gridpos.iter().map(|(p, l)| (p, l.len())) {
        //     // sdgb!(((x, y), len));
        // }

        // connect all lines together
        let mut polylines: Vec<VecDeque<LinePiece>> = closed_loops;
        let mut current_line: VecDeque<LinePiece> = VecDeque::new();
        loop {
            // sdbg!(&lines_by_gridpos)
            // sdgb!(&current_line);
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
                    // sdgb!(&start);
                    // sdgb!(&end);
                    if start == end {
                        // segment is completed, add to polylines
                        polylines.push(current_line);
                        current_line = VecDeque::new();
                        // sdgb!(&polylines);
                    } else {
                        // locate a segment to attach to the end
                        let search_result = 'line_search: {
                            for adjacent in end.get_adjacent_gridposes() {
                                // sdgb!(&adjacent);
                                let Some(lines) = lines_by_gridpos.get_mut(&adjacent) else {
                                    continue;
                                };
                                // sdgb!(&lines);
                                let Some((i, _)) = lines.iter().enumerate().find(|(_, line)| {
                                    line.first() == Some(&LinePiece::BorderCoordinate(*end))
                                }) else {
                                    continue;
                                };
                                break 'line_search Some(lines.remove(i));
                            }
                            None
                        };
                        // sdgb!(&search_result);
                        if let Some(new_line) = search_result {
                            current_line.extend(new_line.into_iter().skip(1));
                            continue;
                        }

                        // locate a segment to attach to the beginning
                        let search_result = 'line_search: {
                            for adjacent in start.get_adjacent_gridposes() {
                                // sdgb!(&adjacent);
                                let Some(lines) = lines_by_gridpos.get_mut(&adjacent) else {
                                    continue;
                                };
                                // sdgb!(&lines);
                                let Some((i, _)) = lines.iter().enumerate().find(|(_, line)| {
                                    line.last() == Some(&LinePiece::BorderCoordinate(*start))
                                }) else {
                                    continue;
                                };
                                break 'line_search Some(lines.remove(i));
                            }
                            None
                        };
                        // sdgb!(&search_result);
                        if let Some(new_line) = search_result {
                            current_line = new_line
                                .into_iter()
                                .chain(current_line.into_iter().skip(1))
                                .collect();
                            continue;
                        }

                        // no line segments remaining, just place the incomplete line into polylines (and print a warning)
                        warn!("Incomplete line segment created!");
                        warn!("Incomplete segment: {current_line:?}");
                        polylines.push(current_line);
                        current_line = VecDeque::new();
                        // sdgb!(&polylines);
                    }
                }
                _ => unimplemented!("non-border coordinate capped lines"),
            }
        }
        // sdgb!(&polylines);

        polylines
            .into_iter()
            .map(|polyline| polyline.into_iter().map(Vec2::from).collect())
            .collect()
    }

    pub fn is_valid_tile_position(&self, tile: &Tile, pos: GridPos) -> bool {
        let mut is_adjacent_tile = false;
        for (orientation, offset) in Orientation::iter_with_offsets() {
            let adjacent_pos = pos + offset;
            let Some(adjacent_tile) = self.placed_tiles.get(&adjacent_pos) else {
                continue;
            };
            is_adjacent_tile = true;
            if tile.validate_mounting(adjacent_tile, orientation).is_none() {
                return false;
            }
        }
        is_adjacent_tile
    }

    pub fn placeable_positions(&self, tile: &Tile) -> Vec<GridPos> {
        self.valid_placements
            .iter()
            .cloned()
            .filter(|pos| {
                let mut tile = tile.clone();
                (0..4).any(|_| {
                    tile.rotate_clockwise();
                    self.is_valid_tile_position(&tile, *pos)
                })
            })
            .collect()
    }

    pub fn draw_placeable_tile(&mut self) -> Option<(Tile, Vec<GridPos>)> {
        for _ in 0..self.library.len() {
            let Some(next_tile) = self.library.pop() else {
                break;
            };
            let placeable_positions = self.placeable_positions(&next_tile);
            if !placeable_positions.is_empty() {
                return Some((next_tile, placeable_positions));
            } else {
                self.library.insert(0, next_tile);
            }
        }
        None
    }

    pub fn discord_presence(&self) -> ActivityBuilder {
        ActivityBuilder::new()
            .details(format!("{} tiles remaining", self.library.len()))
            .state(if matches!(self.local_player, PlayerType::Local) {
                "In a local game"
            } else {
                "In an online game"
            })
            .kind(ActivityKind::Playing)
            .instance(true)
            .party(
                "null",
                Some(NonZero::<u32>::try_from(self.players.len() as u32).unwrap()),
                Some(NonZero::<u32>::try_from(NUM_PLAYERS as u32).unwrap()),
                PartyPrivacy::Private,
            )
            .assets(Assets::default().large("starting-tile", None::<String>))
            .button(Button {
                label: "Download".into(),
                url: LATEST_RELEASE_LINK.into(),
            })
    }
}

#[cfg(test)]
mod test {
    use ggez::{graphics::Color, GameResult};

    use crate::{
        game::{debug_game_configs::river_test, Game},
        pos::GridPos,
        tile::{
            tile_definitions::{
                rivers_1::MONASTARY_POND, CROSSROADS, CURVE_ROAD, MONASTARY, STARTING_TILE,
                STRAIGHT_ROAD,
            },
            SegmentType,
        },
    };

    use super::player::Player;

    #[test]
    fn test_group_coallating() {
        use crate::tile::tile_definitions::*;
        let mut game = Game::new();
        game.place_tile(STRAIGHT_ROAD.clone(), GridPos(0, 0))
            .unwrap();
        game.place_tile(CURVE_ROAD.clone().rotated(), GridPos(-1, 0))
            .unwrap();
        game.place_tile(CORNER_CITY.clone().rotated(), GridPos(0, -1))
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
        let group = game
            .groups
            .values()
            .find(|group| group.gtype == SegmentType::City)
            .unwrap();
        let outline = game.compute_group_outline(group);
        dbg!(outline);
        Ok(())
    }

    #[test]
    pub fn test_group_outline_generation_2() -> GameResult {
        let mut game = Game::new();
        game.place_tile(STRAIGHT_ROAD.clone(), GridPos(0, 0))?;
        game.place_tile(CURVE_ROAD.clone(), GridPos(1, 0))?;
        game.place_tile(STRAIGHT_ROAD.clone().rotated(), GridPos(1, -1))?;
        let group = game
            .groups
            .values()
            .find(|group| group.gtype == SegmentType::Road)
            .unwrap();
        let outline = game.compute_group_outline(group);
        dbg!(outline);
        Ok(())
    }

    #[test]
    pub fn test_group_outline_generation_3() -> GameResult {
        let mut game = Game::new();
        game.place_tile(MONASTARY.clone(), GridPos(0, 0))?;
        let group = game
            .groups
            .values()
            .find(|group| group.gtype == SegmentType::Farm)
            .unwrap();
        let outline = game.compute_group_outline(group);
        dbg!(outline);
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

    #[test]
    pub fn test_scoring_effects() -> GameResult {
        let mut game = Game::new_with_library(vec![CROSSROADS.clone(), CROSSROADS.clone()]);
        let player_ident = game.players.insert(Player::new(Color::RED));
        game.place_tile(CROSSROADS.clone(), GridPos(0, 0))?;
        game.place_meeple((GridPos(0, 0), 2), player_ident)?;
        Ok(())
    }

    #[test]
    pub fn test_valid_tile_placement() -> GameResult {
        let mut game = Game::new_with_library(vec![CROSSROADS.clone(), CROSSROADS.clone()]);
        game.place_tile(STARTING_TILE.clone(), GridPos(0, 0))?;
        assert!(!game.is_valid_tile_position(&STARTING_TILE, GridPos(0, -1)));
        Ok(())
    }

    #[test]
    pub fn test_river_outline_generation() -> GameResult {
        use crate::tile::SegmentType;
        let mut game = river_test()?;
        game.place_tile(MONASTARY_POND.clone().rotated(), GridPos(0, 1))?;
        let group = game
            .groups
            .values()
            .find(|group| group.gtype == SegmentType::Farm && group.segments.len() == 2)
            .unwrap();
        let outline = game.compute_group_outline(group);
        dbg!(outline);
        Ok(())
    }
}

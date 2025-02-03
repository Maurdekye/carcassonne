use super::{player::Player, Game};
use ggez::{graphics::Color, GameError, GameResult};

use crate::{
    game_client::PLAYER_COLORS,
    pos::GridPos,
    tile::{
        tile_definitions::{
            ADJACENT_EDGE_CITIES, BRIDGE_CITY, CORNER_CITY, CROSSROADS, CURVE_ROAD,
            FORITIFED_THREE_QUARTER_CITY_ENTRANCE, FORTIFIED_CORNER_CITY, FOUR_WAY_CROSSROADS,
            OPPOSING_EDGE_CITIES, ROAD_MONASTARY, STARTING_TILE, STRAIGHT_ROAD, THREE_QUARTER_CITY,
        },
        Tile,
    },
    DebugGameConfiguration,
};

impl DebugGameConfiguration {
    pub fn get_game(&self) -> Result<Game, GameError> {
        use DebugGameConfiguration::*;
        match self {
            MeeplePlacement => meeple_locations(),
            MultipleSegmentsPerTileScoring => multiple_segments_per_tile_scoring(),
            MultiplePlayerOwnership => multiple_player_ownership(),
            RotationTest => rotation_test(),
            GroupCoallation => group_coallation(),
        }
    }
}
pub fn meeple_locations() -> Result<Game, GameError> {
    let library: Vec<Tile> = Tile::default_library_tallies()
        .into_iter()
        .map(|(tile, _)| tile)
        .cloned()
        .collect();
    let width = library.len().isqrt().max(1);
    let height = library.len() / width;
    let mut this = Game::new_with_library(vec![STARTING_TILE.clone()]);
    let player_ident = this.players.insert(Player::new(Color::BLACK));
    this.players.get_mut(player_ident).unwrap().meeples =
        library.iter().map(|tile| tile.segments.len()).sum();
    for (i, tile) in library.into_iter().enumerate() {
        let pos = GridPos(
            ((i % width) as i32 - (width as i32 / 2)) * 2,
            ((i / width) as i32 - (height as i32 / 2)) * 2,
        );
        let segments = tile.segments.len();
        this.place_tile(tile, pos)?;
        for seg_index in 0..segments {
            this.place_meeple((pos, seg_index), player_ident)?;
        }
    }
    Ok(this)
}

pub fn multiple_segments_per_tile_scoring() -> Result<Game, GameError> {
    let mut this = Game::new_with_library(vec![
        BRIDGE_CITY.clone(),
        BRIDGE_CITY.clone(),
        CORNER_CITY.clone().rotated().rotated().rotated(),
        CORNER_CITY.clone().rotated().rotated().rotated(),
        CURVE_ROAD.clone().rotated().rotated().rotated(),
    ]);

    let player_ident = this.players.insert(Player::new(Color::BLACK));

    // opposing edge cities

    this.place_tile(OPPOSING_EDGE_CITIES.clone().rotated(), GridPos(-3, 0))?;
    this.place_meeple((GridPos(-3, 0), 0), player_ident)?;
    this.place_tile(CORNER_CITY.clone().rotated(), GridPos(-4, 0))?;
    this.place_tile(CORNER_CITY.clone(), GridPos(-2, 0))?;
    this.place_tile(CORNER_CITY.clone().rotated().rotated(), GridPos(-4, -1))?;
    this.place_tile(
        CORNER_CITY.clone().rotated().rotated().rotated(),
        GridPos(-2, -1),
    )?;

    this.place_tile(OPPOSING_EDGE_CITIES.clone().rotated(), GridPos(-3, -3))?;
    this.place_meeple((GridPos(-3, -3), 0), player_ident)?;
    this.place_tile(FORTIFIED_CORNER_CITY.clone().rotated(), GridPos(-4, -3))?;
    this.place_tile(CORNER_CITY.clone(), GridPos(-2, -3))?;
    this.place_tile(CORNER_CITY.clone().rotated().rotated(), GridPos(-4, -4))?;
    this.place_tile(
        CORNER_CITY.clone().rotated().rotated().rotated(),
        GridPos(-2, -4),
    )?;

    // adjacent edge cities

    this.place_tile(ADJACENT_EDGE_CITIES.clone(), GridPos(0, 0))?;
    this.place_meeple((GridPos(0, 0), 0), player_ident)?;
    this.place_tile(CORNER_CITY.clone(), GridPos(1, 0))?;
    this.place_tile(CORNER_CITY.clone().rotated().rotated(), GridPos(0, -1))?;

    this.place_tile(ADJACENT_EDGE_CITIES.clone(), GridPos(0, -3))?;
    this.place_meeple((GridPos(0, -3), 0), player_ident)?;
    this.place_tile(FORTIFIED_CORNER_CITY.clone(), GridPos(1, -3))?;
    this.place_tile(CORNER_CITY.clone().rotated().rotated(), GridPos(0, -4))?;

    // crossroads

    this.place_tile(CROSSROADS.clone().rotated().rotated(), GridPos(3, 0))?;
    this.place_meeple((GridPos(3, 0), 4), player_ident)?;
    this.place_tile(CURVE_ROAD.clone(), GridPos(4, 0))?;
    this.place_tile(CURVE_ROAD.clone().rotated().rotated(), GridPos(3, -1))?;

    this.players.get_mut(player_ident).unwrap().meeples = 0;

    Ok(this)
}

pub fn multiple_player_ownership() -> Result<Game, GameError> {
    let mut this = Game::new_with_library(vec![STARTING_TILE.clone()]);

    let players = PLAYER_COLORS.map(|color| this.players.insert(Player::new(color)));

    fn put_city(game: &mut Game, origin: GridPos) -> GameResult<()> {
        game.place_tile(
            CORNER_CITY.clone().rotated().rotated(),
            origin + GridPos(0, -1),
        )?;
        game.place_tile(
            CORNER_CITY.clone().rotated().rotated().rotated(),
            origin + GridPos(1, -1),
        )?;
        game.place_tile(THREE_QUARTER_CITY.clone().rotated(), origin + GridPos(0, 0))?;
        game.place_tile(
            THREE_QUARTER_CITY.clone().rotated().rotated().rotated(),
            origin + GridPos(1, 0),
        )?;
        game.place_tile(CORNER_CITY.clone().rotated(), origin + GridPos(0, 1))?;
        game.place_tile(CORNER_CITY.clone(), origin + GridPos(1, 1))?;
        Ok(())
    }

    put_city(&mut this, GridPos(-3, 0))?;

    put_city(&mut this, GridPos(0, 0))?;
    this.place_meeple((GridPos(0, 0), 0), players[0])?;

    put_city(&mut this, GridPos(3, 0))?;
    this.place_meeple((GridPos(3, 0), 0), players[0])?;
    this.place_meeple((GridPos(3, 1), 0), players[1])?;

    put_city(&mut this, GridPos(6, 0))?;
    this.place_meeple((GridPos(6, 0), 0), players[0])?;
    this.place_meeple((GridPos(6, 1), 0), players[1])?;
    this.place_meeple((GridPos(7, 0), 0), players[2])?;

    put_city(&mut this, GridPos(9, 0))?;
    this.place_meeple((GridPos(9, 0), 0), players[0])?;
    this.place_meeple((GridPos(9, 1), 0), players[1])?;
    this.place_meeple((GridPos(10, 0), 0), players[2])?;
    this.place_meeple((GridPos(10, 1), 0), players[3])?;

    put_city(&mut this, GridPos(12, 0))?;
    this.place_meeple((GridPos(12, 0), 0), players[0])?;
    this.place_meeple((GridPos(12, 1), 0), players[1])?;
    this.place_meeple((GridPos(13, 0), 0), players[2])?;
    this.place_meeple((GridPos(13, 1), 0), players[3])?;
    this.place_meeple((GridPos(13, -1), 0), players[4])?;

    Ok(this)
}

pub fn rotation_test() -> GameResult<Game> {
    let mut this = Game::new_with_library(vec![STARTING_TILE.clone()]);
    this.players.insert(Player::new(Color::BLUE));

    fn spin(mut tile: Tile, rotations: usize) -> Tile {
        for _ in 0..rotations {
            tile.rotate_clockwise();
        }
        tile
    }

    for x in 0..=4 {
        let num_rotations = 10usize.pow(x as u32);
        this.place_tile(
            spin(STARTING_TILE.clone(), num_rotations),
            GridPos(x * 2, 0),
        )?;
        this.place_tile(
            spin(FOUR_WAY_CROSSROADS.clone(), num_rotations),
            GridPos(x * 2, 2),
        )?;
        this.place_tile(
            spin(FORITIFED_THREE_QUARTER_CITY_ENTRANCE.clone(), num_rotations),
            GridPos(x * 2, 4),
        )?;
        this.place_tile(
            spin(ROAD_MONASTARY.clone(), num_rotations),
            GridPos(x * 2, 6),
        )?;
    }

    Ok(this)
}

fn group_coallation() -> GameResult<Game> {
    let mut this = Game::new_with_library(vec![STRAIGHT_ROAD.clone()]);
    this.players.insert(Player::new(Color::BLUE));

    this.place_tile(CURVE_ROAD.clone(), GridPos(0, 0))?;
    this.place_tile(STRAIGHT_ROAD.clone(), GridPos(-1, 0))?;
    this.place_tile(CURVE_ROAD.clone().rotated(), GridPos(-2, 0))?;
    this.place_tile(CURVE_ROAD.clone().rotated().rotated(), GridPos(-2, -1))?;
    this.place_tile(
        CURVE_ROAD.clone().rotated().rotated().rotated(),
        GridPos(0, -1),
    )?;

    Ok(this)
}

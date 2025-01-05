// use std::io::{stdout, Write};

use std::collections::HashMap;

use ggez::{
    conf::{WindowMode, WindowSetup},
    event::{self, EventHandler},
    graphics::{Canvas, Color, DrawMode, DrawParam, Mesh},
    input::keyboard::KeyCode,
    Context, ContextBuilder, GameError, GameResult,
};
use pos::Pos;
use slotmap::{DefaultKey, SlotMap};
use tile::{get_tile_library, tile_definitions::STRAIGHT_ROAD, Orientation, SegmentType, Tile};

pub mod pos;
mod tile;
mod util;

const GRID_SIZE: f32 = 0.1;

type SegmentIdentifier = (Pos, usize);

#[derive(Debug)]
struct SegmentGroup {
    gtype: SegmentType,
    segments: Vec<SegmentIdentifier>,
}

struct Game {
    library: Vec<Tile>,
    placed_tiles: HashMap<Pos, Tile>,
    selected_square: Option<Pos>,
    held_tile: Option<Tile>,
    last_selected_square: Option<Pos>,
    placement_is_valid: bool,
    groups: SlotMap<DefaultKey, SegmentGroup>,
    group_associations: HashMap<SegmentIdentifier, DefaultKey>,
}

impl Game {
    fn new() -> Self {
        let mut this = Self {
            library: get_tile_library(),
            placed_tiles: HashMap::new(),
            selected_square: None,
            held_tile: None,
            last_selected_square: None,
            placement_is_valid: false,
            groups: SlotMap::new(),
            group_associations: HashMap::new(),
        };
        this.place_tile(STRAIGHT_ROAD.clone(), Pos(5, 5)).unwrap();
        this
    }

    fn place_tile(&mut self, tile: Tile, pos: Pos) -> Result<(), GameError> {
        let mut new_group_insertions: HashMap<SegmentIdentifier, Vec<DefaultKey>> = HashMap::new();
        let mut uninserted_segments: Vec<_> = (0..tile.segments.len()).map(|_| true).collect();

        // evaluate mountings with neighboring tiles
        for (orientation, offset) in Orientation::iter_with_offsets() {
            let adjacent_pos = pos + offset;
            let Some(adjacent_tile) = self.placed_tiles.get(&adjacent_pos) else {
                continue;
            };

            let Some(mounts) = tile.validate_mounting(adjacent_tile, orientation) else {
                return Err(GameError::CustomError(
                    "Attempt to place invalid tile!".to_string(),
                ));
            };

            for mount in mounts {
                let seg_id: SegmentIdentifier = (pos, mount.from_segment);
                let adj_seg_id: SegmentIdentifier = (adjacent_pos, mount.to_segment);
                let group_key = self
                    .group_associations
                    .get(&adj_seg_id)
                    .expect("All placed segments have associated groups");
                new_group_insertions
                    .entry(seg_id)
                    .and_modify(|groups| groups.push(*group_key))
                    .or_insert(vec![*group_key]);
                uninserted_segments[mount.from_segment] = false;
            }
        }

        // insert segments into existing connected groups
        for (seg_id, additions) in new_group_insertions {
            #[allow(clippy::comparison_chain)]
            if additions.len() == 1 {
                self.groups
                    .get_mut(additions[0])
                    .unwrap()
                    .segments
                    .push(seg_id);
            } else if additions.len() > 1 {
                let mut new_segment_list: Vec<_> = additions
                    .iter()
                    .flat_map(|key| self.groups.remove(*key).unwrap().segments)
                    .collect();
                new_segment_list.push(seg_id);
                let new_segment_group = SegmentGroup {
                    gtype: tile.segments[seg_id.1].stype,
                    segments: new_segment_list,
                };
                let key = self.groups.insert(new_segment_group);
                for seg_id in &self.groups.get(key).unwrap().segments {
                    self.group_associations.insert(*seg_id, key);
                }
            } else {
                panic!("entry in insertions map with no values??");
            }
        }

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

        self.placed_tiles.insert(pos, tile);

        Ok(())
    }

    fn reevaluate_selected_square(&mut self) {
        self.placement_is_valid = false;

        let Some(selected_square) = &self.selected_square else {
            return;
        };

        if self.placed_tiles.contains_key(selected_square) {
            return;
        }

        if let Some(held_tile) = &self.held_tile {
            let mut is_adjacent_tile = false;
            for (orientation, offset) in Orientation::iter_with_offsets() {
                let adjacent_pos = *selected_square + offset;
                let Some(adjacent_tile) = self.placed_tiles.get(&adjacent_pos) else {
                    continue;
                };
                is_adjacent_tile = true;
                if held_tile
                    .validate_mounting(adjacent_tile, orientation)
                    .is_none()
                {
                    return;
                }
            }
            if !is_adjacent_tile {
                return;
            }
        } else {
            return;
        }

        self.placement_is_valid = true;
    }
}

impl EventHandler<GameError> for Game {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        let mouse = ctx.mouse.position();
        let grid_pos = Pos::from_screen_pos(mouse, ctx);
        self.selected_square = Some(grid_pos);
        if self.selected_square != self.last_selected_square {
            self.reevaluate_selected_square();
            self.last_selected_square = self.selected_square;
        }

        if let Some(tile) = &mut self.held_tile {
            if ctx.keyboard.is_key_just_pressed(KeyCode::R) {
                tile.rotate();
                self.reevaluate_selected_square();
            }
        }

        if self.held_tile.is_some() {
            if ctx.mouse.button_just_pressed(event::MouseButton::Left) && self.placement_is_valid {
                let tile = self.held_tile.take().unwrap();
                self.place_tile(tile, grid_pos)?;
                // dbg!(&self.groups);
                self.reevaluate_selected_square();
            }
        } else {
            self.held_tile = self.library.pop();
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas = Canvas::from_frame(ctx, Color::WHITE);

        for (pos, tile) in &self.placed_tiles {
            tile.render(ctx, &mut canvas, pos.rect(ctx))?;
        }

        if let Some(pos) = self.selected_square {
            let rect = pos.rect(ctx);
            let cursor_color = if !self.placement_is_valid {
                Color::RED
            } else {
                Color::GREEN
            };
            if !self.placed_tiles.contains_key(&pos) {
                if let Some(tile) = &self.held_tile {
                    tile.render(ctx, &mut canvas, rect)?;
                }
            }
            canvas.draw(
                &Mesh::new_rectangle(ctx, DrawMode::stroke(2.0), rect, cursor_color)?,
                DrawParam::default(),
            )
        }

        ctx.gfx
            .set_window_title(&format!("Carcassone: {:.2} fps", ctx.time.fps()));

        canvas.finish(ctx)
    }
}

fn main() -> GameResult {
    let (ctx, event_loop) = ContextBuilder::new("carcassone", "maurdekye")
        .window_mode(WindowMode::default().dimensions(800.0, 800.0))
        .window_setup(WindowSetup::default().title("Carcassone"))
        .build()?;
    let game = Game::new();
    event::run(ctx, event_loop, game);
}

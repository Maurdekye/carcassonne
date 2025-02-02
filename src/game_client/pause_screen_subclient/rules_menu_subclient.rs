use ggez::{
    glam::{vec2, Vec2},
    graphics::{Canvas, Color, DrawMode, Mesh, Rect, Text},
    input::keyboard::KeyCode,
    Context, GameError,
};
use std::{
    cell::RefCell,
    rc::Rc,
    sync::mpsc::{channel, Receiver, Sender},
};

use crate::{
    colors::PANEL_COLOR,
    sub_event_handler::SubEventHandler,
    ui_manager::{Button, ButtonBounds, ButtonState, UIManager},
    util::{AnchorPoint, DrawableWihParamsExt, RectExt, TextExt, Vec2ToRectExt},
};

use super::PauseScreenEvent;

const NUM_PAGES: usize = 11;

#[derive(Clone)]
pub enum RulesMenuEvent {
    PauseScreenEvent(PauseScreenEvent),
    PreviousPage,
    NextPage,
}

pub struct RulesMenuSubclient {
    parent_channel: Sender<PauseScreenEvent>,
    event_sender: Sender<RulesMenuEvent>,
    event_receiver: Receiver<RulesMenuEvent>,
    ui: UIManager<RulesMenuEvent, RulesMenuEvent>,
    prev_page: Rc<RefCell<Button<RulesMenuEvent>>>,
    next_page: Rc<RefCell<Button<RulesMenuEvent>>>,
    page: usize,
}

impl RulesMenuSubclient {
    pub fn new(parent_channel: Sender<PauseScreenEvent>) -> Self {
        let (event_sender, event_receiver) = channel();
        let ui_sender = event_sender.clone();
        let (ui, [_, prev_page, next_page]) = UIManager::new_and_rc_buttons(
            ui_sender,
            [
                Button::new(
                    ButtonBounds::absolute(Rect::new(55.0, 20.0, 50.0, 30.0)),
                    Text::new("<").size(24.0),
                    RulesMenuEvent::PauseScreenEvent(PauseScreenEvent::MainMenu),
                ),
                Button::new(
                    ButtonBounds {
                        relative: Rect::new(0.5, 1.0, 0.0, 0.0),
                        absolute: Rect::new(-160.0, -80.0, 50.0, 30.0),
                    },
                    Text::new("<-"),
                    RulesMenuEvent::PreviousPage,
                ),
                Button::new(
                    ButtonBounds {
                        relative: Rect::new(0.5, 1.0, 0.0, 0.0),
                        absolute: Rect::new(130.0, -80.0, 50.0, 30.0),
                    },
                    Text::new("->"),
                    RulesMenuEvent::NextPage,
                ),
            ],
        );
        Self {
            parent_channel,
            event_sender,
            event_receiver,
            ui,
            next_page,
            prev_page,
            page: 0,
        }
    }

    fn handle_event(&mut self, _ctx: &mut Context, event: RulesMenuEvent) -> Result<(), GameError> {
        match event {
            RulesMenuEvent::PauseScreenEvent(event) => self.parent_channel.send(event).unwrap(),
            RulesMenuEvent::PreviousPage => {
                self.page = ((self.page as i32 - 1) % NUM_PAGES as i32) as usize
            }
            RulesMenuEvent::NextPage => self.page = (self.page + 1) % NUM_PAGES,
        }
        Ok(())
    }
}

impl SubEventHandler<GameError> for RulesMenuSubclient {
    fn update(&mut self, ctx: &mut Context) -> Result<(), GameError> {
        self.ui.update(ctx)?;

        if ctx.keyboard.is_key_just_pressed(KeyCode::Escape) {
            self.event_sender
                .send(RulesMenuEvent::PauseScreenEvent(PauseScreenEvent::MainMenu))
                .unwrap();
        }

        while let Ok(event) = self.event_receiver.try_recv() {
            self.handle_event(ctx, event)?;
        }

        self.next_page.borrow_mut().state = ButtonState::disabled_if(self.page == NUM_PAGES - 1);
        self.prev_page.borrow_mut().state = ButtonState::disabled_if(self.page == 0);

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context, canvas: &mut Canvas) -> Result<(), GameError> {
        let res: Vec2 = ctx.gfx.drawable_size().into();

        let panel_origin = vec2(100.0, 100.0);
        let panel = {
            let dims = res - vec2(200.0, 200.0);
            Rect::new(panel_origin.x, panel_origin.y, dims.x, dims.y)
        };

        Mesh::new_rectangle(ctx, DrawMode::fill(), panel, PANEL_COLOR)?.draw(canvas);

        match self.page {
            0 => {
                Text::new("Rules")
                    .size(56.0)
                    .anchored_by(ctx, panel_origin + vec2(10.0, 10.0), AnchorPoint::NorthWest)?
                    .draw(canvas);

                {
                    Text::new(
                        "\
Carcassonne is a game about placing tiles and 
scoring groups. The aim of the game is to amass 
the greatest score by the end of the game.
Turns proceed from player to player in two phases:
  1. Place a tile on the board
  2. Optionally place a meeple on an unclaimed 
    segment of that tile
Play proceeds from player to player until all 
tiles are exhausted, or none of the remaining tiles 
can be played.",
                    )
                    .draw_into_rect(
                        ctx,
                        canvas,
                        Color::WHITE,
                        32.0,
                        (panel_origin + vec2(10.0, 80.0))
                            .to(panel.bottom_right() - vec2(10.0, 10.0)),
                    )?;
                }
            }
            1 => {
                Text::new("Tile Placement")
                    .size(36.0)
                    .anchored_by(ctx, panel_origin + vec2(10.0, 10.0), AnchorPoint::NorthWest)?
                    .draw(canvas);

                {
                    Text::new(
                        "\
When placing a tile, you must abide by the 
following rules:
 * You must place the tile adjacent to an existing,
    already-placed tile on the board
 * The groups at the edges of all adjacent tiles
    must line up appropriately: roads must connect
    to roads, cities to cities, farms to farms,
    etc.
Upon successfully placing a tile on the board, 
play proceeds to the meeple placement phase.",
                    )
                    .draw_into_rect(
                        ctx,
                        canvas,
                        Color::WHITE,
                        32.0,
                        (panel_origin + vec2(10.0, 60.0))
                            .to(panel.bottom_right() - vec2(10.0, 10.0)),
                    )?;
                }
            }
            2 => {
                Text::new("Meeple Placement")
                    .size(36.0)
                    .anchored_by(ctx, panel_origin + vec2(10.0, 10.0), AnchorPoint::NorthWest)?
                    .draw(canvas);

                {
                    Text::new(
                        "\
All players begin with 7 meeples in their stock.
If you have remaining meeples in your stock, you
may optionally choose to select a segment on the
tile you have just placed in order to place a
meeple down. Meeple placement follows one rule:
 * You may not place a meeple on a segment
    connected to a group that is already
    claimed by another existing meeple.
After placing your meeple, the game proceeds to
the scoring phase, where any newly closed groups
with meeples placed on them are scored appropriately.
If you don't wish to, you may choose not to place
a meeple, and simply proceed directly the scoring 
phase.",
                    )
                    .draw_into_rect(
                        ctx,
                        canvas,
                        Color::WHITE,
                        32.0,
                        (panel_origin + vec2(10.0, 60.0))
                            .to(panel.bottom_right() - vec2(10.0, 10.0)),
                    )?;
                }
            }
            3 => {
                Text::new("Scoring")
                    .size(36.0)
                    .anchored_by(ctx, panel_origin + vec2(10.0, 10.0), AnchorPoint::NorthWest)?
                    .draw(canvas);

                {
                    Text::new(
                        "\
After deciding whether or not to place a meeple,
scoring occurs. Any groups with meeples placed on them
that were closed off by the placement of the last tile
are tallied up and scored. Scoring varies depending
on the type of group. There are four different types of
groups in the game: Roads, Cities, Farms, and Monastaries.
                    ",
                    )
                    .draw_into_rect(
                        ctx,
                        canvas,
                        Color::WHITE,
                        32.0,
                        (panel_origin + vec2(10.0, 60.0))
                            .to(panel.bottom_right() - vec2(10.0, 10.0)),
                    )?;
                }
            }
            4 => {
                Text::new("Road & City Scoring")
                    .size(36.0)
                    .anchored_by(ctx, panel_origin + vec2(10.0, 10.0), AnchorPoint::NorthWest)?
                    .draw(canvas);

                {
                    Text::new(
                        "\
After a road or city is closed off on all tile edges,
meeples are removed from the group and scored based on
how many tiles are contained within the group.
 * Roads: 1 point per tile
 * Cities: 2 points per tile
    * If the tile is Fortified, with a blue shield icon on
       the city, then the tile is worth 4 points when
       scored, as opposed to 2.
All removed meeples are returned to the player's individual
stocks after scoring.",
                    )
                    .draw_into_rect(
                        ctx,
                        canvas,
                        Color::WHITE,
                        32.0,
                        (panel_origin + vec2(10.0, 60.0))
                            .to(panel.bottom_right() - vec2(10.0, 10.0)),
                    )?;
                }
            }
            5 => {
                Text::new("Monastary Scoring")
                    .size(36.0)
                    .anchored_by(ctx, panel_origin + vec2(10.0, 10.0), AnchorPoint::NorthWest)?
                    .draw(canvas);

                {
                    Text::new(
                        "\
Monastaries are unique, in that they are always a
one-tile group. However, they are not scored like
cities or roads. In order to complete a monastary
and score it, you must surround it with tiles in
all 8 cardinal directions. Once the last tile is placed,
the monastary is scored like a normal group worth 9
points, and the meeple placed on it is returned
to the owning player.",
                    )
                    .draw_into_rect(
                        ctx,
                        canvas,
                        Color::WHITE,
                        32.0,
                        (panel_origin + vec2(10.0, 60.0))
                            .to(panel.bottom_right() - vec2(10.0, 10.0)),
                    )?;
                }
            }
            6 => {
                Text::new("Farm Scoring")
                    .size(36.0)
                    .anchored_by(ctx, panel_origin + vec2(10.0, 10.0), AnchorPoint::NorthWest)?
                    .draw(canvas);

                {
                    Text::new(
                        "\
Unlike other group types, Farms are not scored
upon the group being closed; they are only scored
at the end of the game, after all tiles have been
exhausted. This means that all meeples placed on
farms remain there for the rest of the game.
Farms are not scored based on the number of tiles
in the farm's group. Instead, scoring is based on
the number of completed city groups that the farm
is touching. For each one, the owner of the farm
receives 3 points.",
                    )
                    .draw_into_rect(
                        ctx,
                        canvas,
                        Color::WHITE,
                        32.0,
                        (panel_origin + vec2(10.0, 60.0))
                            .to(panel.bottom_right() - vec2(10.0, 10.0)),
                    )?;
                }
            }
            7 => {
                Text::new("End of the Game")
                    .size(36.0)
                    .anchored_by(ctx, panel_origin + vec2(10.0, 10.0), AnchorPoint::NorthWest)?
                    .draw(canvas);

                {
                    Text::new(
                        "\
After the scoring phase finishes, play proceeds
to the next player, who continues by placing a
tile of their own. 
Once all tiles in the library have been exhausted,
or none of the remaining tiles can be placed, the
game ends immediately afterwards, and all remaining
placed meeples on the board are scored.
 * Farms are scored as described on the prior page.
 * Roads are scored as normal, as though they were
    complete.
 * Cities are scored as normal, with the caveat
    that each city tile is only worth half the
    points of a completed city: 1 point for each
    normal city tile, and 2 points for each 
    fortified city tile.
 * Monastaries are scored based on the number of
    tiles surrounding them in the 8 cardinal 
    directions; the owner is awarded 1 point per 
    tile surrounding the monastary, plus 1 for
    the monastary tile itself.",
                    )
                    .draw_into_rect(
                        ctx,
                        canvas,
                        Color::WHITE,
                        32.0,
                        (panel_origin + vec2(10.0, 60.0))
                            .to(panel.bottom_right() - vec2(10.0, 10.0)),
                    )?;
                }
            }
            8 => {
                Text::new("Scoring Co-owned Groups")
                    .size(36.0)
                    .anchored_by(ctx, panel_origin + vec2(10.0, 10.0), AnchorPoint::NorthWest)?
                    .draw(canvas);

                {
                    Text::new(
                        "\
While it isn't possible to place a second meeple directly
on a group in which a meeple is already placed,
sometimes existing groups with meeples already placed
on them may end up connected together such that the
ownership of the combined group is shared between
multiple players. In this scenario, scoring of the
group depends on the number of meeples each player has
on the group:
 * Whichever player has the most meeples on the group
    is awarded the full score for the group, and all
    other players receive 0 points. 
 * If there is a tie between who has the most meeples
    in the group, then all players tying for ownership
    are awarded the full score for the group. As 
    before, all other players, if any, receive 0 points.
All meeples on the group are returned to their respective
players' stocks after scoring as normal, regardless 
of whether or not they scored any points from the group.",
                    )
                    .draw_into_rect(
                        ctx,
                        canvas,
                        Color::WHITE,
                        32.0,
                        (panel_origin + vec2(10.0, 60.0))
                            .to(panel.bottom_right() - vec2(10.0, 10.0)),
                    )?;
                }
            }
            9 => {
                Text::new("Tips & Strategies")
                    .size(36.0)
                    .anchored_by(ctx, panel_origin + vec2(10.0, 10.0), AnchorPoint::NorthWest)?
                    .draw(canvas);

                {
                    Text::new(
                        "\
 * Because farms are only scored at the very
    end of the game, be cautious when placing meeples
    on them, as they will remain out of your stock
    for the rest of the game, limiting your ability
    to score further groups.
 * Some groups can be scored immediately on the same
    turn that they're closed; if a group is unclaimed
    on the turn that it is closed, the player closing
    the group may place a meeple on that group and 
    immediately retrieve it, awarding themselves the
    points for it without sacrificing the meeple from
    their stock. For this reason, it is generally a
    good idea to keep at least one meeple leftover in
    your stock, so that you can score these unclaimed,
    leftover groups.",
                    )
                    .draw_into_rect(
                        ctx,
                        canvas,
                        Color::WHITE,
                        32.0,
                        (panel_origin + vec2(10.0, 60.0))
                            .to(panel.bottom_right() - vec2(10.0, 10.0)),
                    )?;
                }
            }
            10 => {
                Text::new("Tips & Strategies cont.")
                    .size(36.0)
                    .anchored_by(ctx, panel_origin + vec2(10.0, 10.0), AnchorPoint::NorthWest)?
                    .draw(canvas);

                {
                    Text::new(
                        "\
 * Tiles can be played to your own benefit by building
    up your own claimed groups, but they can also be 
    played aggressively to the detriment of your
    opponents. If you can't benefit yourself with a
    tile, try placing it near an opponent's claimed
    group to make it more difficult for a tile to fit
    into the space needed to close it.
 * Co-ownership isn't always incidental; you can
    strategize around it and try to intentionally
    invoke it yourself. If a player is building a
    particularly large city, or has ownership of a
    very lucrative farm, try placing a city or
    farm tile nearby, but not adjacent, such that
    you can place a meeple in the isolated group, and
    later connect that group to your opponent's and 
    share the score.",
                    )
                    .draw_into_rect(
                        ctx,
                        canvas,
                        Color::WHITE,
                        32.0,
                        (panel_origin + vec2(10.0, 60.0))
                            .to(panel.bottom_right() - vec2(10.0, 10.0)),
                    )?;
                }
            }
            _ => {}
        }

        self.ui.draw(ctx, canvas)?;

        Ok(())
    }
}

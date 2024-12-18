use std::io::{stdout, Write};

use ggez::{
    conf::{WindowMode, WindowSetup},
    event::{self, EventHandler},
    graphics::{Canvas, Color, DrawMode, DrawParam, Mesh, Rect},
    mint::Point2,
    Context, ContextBuilder, GameError, GameResult,
};

const GRID_SIZE: f32 = 0.1;

#[derive(Clone, Copy, Debug)]
struct Pos(i32, i32);

#[derive(Default)]
struct Game {
    selected_square: Option<Pos>,
}

fn screen_pos_to_grid_pos(screen_pos: Point2<f32>, ctx: &Context) -> Pos {
    let res = ctx.gfx.window().inner_size();
    let uv = Point2 {
        x: screen_pos.x / res.width as f32,
        y: screen_pos.y / res.height as f32,
    };
    Pos((uv.x / GRID_SIZE) as i32, (uv.y / GRID_SIZE) as i32)
}

fn grid_pos_to_screen_pos(grid_pos: Pos, ctx: &Context) -> Point2<f32> {
    let res = ctx.gfx.window().inner_size();
    Point2 {
        x: (grid_pos.0 as f32 * GRID_SIZE) * res.width as f32,
        y: (grid_pos.1 as f32 * GRID_SIZE) * res.height as f32,
    }
}

impl EventHandler<GameError> for Game {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        let mouse = ctx.mouse.position();
        let grid_pos = screen_pos_to_grid_pos(mouse, ctx);
        self.selected_square = Some(grid_pos);
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas = Canvas::from_frame(ctx, Color::WHITE);
        let resolution = ctx.gfx.window().inner_size();

        if let Some(pos) = self.selected_square {
            let near_corner = grid_pos_to_screen_pos(pos, ctx);
            let width = resolution.width as f32 * GRID_SIZE;
            let height = resolution.height as f32 * GRID_SIZE;
            print!("{pos:?} {near_corner:?}      \r");
            stdout().flush().unwrap();
            let rect = Rect::new(near_corner.x, near_corner.y, width, height);
            canvas.draw(
                &Mesh::new_rectangle(ctx, DrawMode::fill(), rect, Color::GREEN)?,
                DrawParam::default(),
            );
        }

        canvas.finish(ctx)
    }
}

fn main() -> GameResult {
    let (ctx, event_loop) = ContextBuilder::new("carcassone", "maurdekye")
        .window_mode(WindowMode::default().dimensions(800.0, 800.0))
        .window_setup(WindowSetup::default().title("Carcassone"))
        .build()?;
    let game = Game::default();
    event::run(ctx, event_loop, game);
}

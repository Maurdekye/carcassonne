use ggez::{graphics::Canvas, Context};

pub trait SubEventHandler<E> {
    fn update(&mut self, ctx: &mut Context) -> Result<(), E>;
    fn draw(&mut self, ctx: &mut Context, canvas: &mut Canvas) -> Result<(), E>;
}

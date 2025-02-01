use ggez::{graphics::Canvas, Context};

pub trait SubEventHandler<E> {
    fn update(&mut self, ctx: &mut Context) -> Result<(), E>;
    fn draw(&mut self, ctx: &mut Context, canvas: &mut Canvas) -> Result<(), E>;
    fn mouse_wheel_event(&mut self, _ctx: &mut Context, _x: f32, _y: f32) -> Result<(), E> {
        Ok(())
    }
}

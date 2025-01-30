use std::collections::{hash_map::Entry, HashMap, HashSet};

use ggez::{
    glam::{vec2, Vec2},
    graphics::{Canvas, Color, DrawParam, Drawable, Rect, Text},
    Context, GameError,
};

pub fn refit_to_rect(vec: Vec2, rect: Rect) -> Vec2 {
    vec2(vec.x * rect.w + rect.x, vec.y * rect.h + rect.y)
}

pub fn point_in_polygon(point: Vec2, polygon: &[Vec2]) -> bool {
    let mut crossings = 0;
    for (i, j) in (0..polygon.len()).zip((1..polygon.len()).chain([0])) {
        let a = polygon[i];
        let b = polygon[j];
        if (a.y > point.y) != (b.y > point.y) {
            let slope = (b.x - a.x) / (b.y - a.y);
            let x_intersect = slope * (point.y - a.y) + a.x;

            if point.x < x_intersect {
                crossings += 1;
            }
        }
    }
    crossings % 2 == 1
}

pub trait HashMapBag<K, V> {
    fn place(&mut self, key: K, value: V) -> usize;
}

impl<K, V> HashMapBag<K, V> for HashMap<K, Vec<V>>
where
    K: std::hash::Hash + Eq,
{
    fn place(&mut self, key: K, value: V) -> usize {
        match self.entry(key) {
            Entry::Occupied(occupied_entry) => {
                let list = occupied_entry.into_mut();
                list.push(value);
                list.len()
            }
            Entry::Vacant(vacant_entry) => {
                let key = vacant_entry.into_key();
                self.insert(key, vec![value]);
                1
            }
        }
    }
}

impl<K, V> HashMapBag<K, V> for HashMap<K, HashSet<V>>
where
    K: std::hash::Hash + Eq,
    V: std::hash::Hash + Eq,
{
    fn place(&mut self, key: K, value: V) -> usize {
        match self.entry(key) {
            Entry::Occupied(occupied_entry) => {
                let list = occupied_entry.into_mut();
                list.insert(value);
                list.len()
            }
            Entry::Vacant(vacant_entry) => {
                let key = vacant_entry.into_key();
                self.insert(key, HashSet::from([value]));
                1
            }
        }
    }
}

pub struct Bag<K, V>(pub HashMap<K, Vec<V>>);

impl<K, V> FromIterator<(K, V)> for Bag<K, V>
where
    K: std::hash::Hash + Eq,
{
    fn from_iter<T: IntoIterator<Item = (K, V)>>(iter: T) -> Self {
        let mut bag = HashMap::new();
        for (k, v) in iter {
            bag.place(k, v);
        }
        Bag(bag)
    }
}

pub trait MapFindExt
where
    Self: Iterator,
{
    #[allow(unused)]
    fn map_find<F, O>(self, f: F) -> Option<O>
    where
        Self: Sized,
        F: FnMut(Self::Item) -> Option<O>;
}

impl<I> MapFindExt for I
where
    I: Iterator,
{
    fn map_find<F, O>(self, f: F) -> Option<O>
    where
        Self: Sized,
        F: FnMut(Self::Item) -> Option<O>,
    {
        self.filter_map(f).next()
    }
}

pub trait RotateExt: Sized {
    fn rotate_(&mut self);
    #[allow(unused)]
    fn rotated(mut self) -> Self {
        self.rotate_();
        self
    }
}

impl RotateExt for Vec2 {
    fn rotate_(&mut self) {
        *self = vec2(1.0 - self.y, self.x);
    }
}

pub fn color_mul(color: Color, factor: f32) -> Color {
    Color::from((
        (color.r * factor).clamp(0.0, 1.0),
        (color.g * factor).clamp(0.0, 1.0),
        (color.b * factor).clamp(0.0, 1.0),
    ))
}

#[allow(unused)]
#[derive(Clone, Copy)]
pub enum AnchorPoint {
    NorthWest,
    NorthCenter,
    NorthEast,
    SouthWest,
    SouthEast,
    Center,
}

pub trait TextExt: Sized {
    fn pos(&self, pos: Vec2) -> DrawableWihParams<'_, Self>;
    fn centered_on<'a>(
        &'a self,
        ctx: &Context,
        pos: Vec2,
    ) -> Result<DrawableWihParams<'a, Self>, GameError>;
    fn anchored_by<'a>(
        &'a self,
        ctx: &Context,
        pos: Vec2,
        anchor: AnchorPoint,
    ) -> Result<DrawableWihParams<'a, Self>, GameError>;
    fn size(self, size: f32) -> Self;
}

impl TextExt for Text {
    fn centered_on<'a>(
        &'a self,
        ctx: &Context,
        pos: Vec2,
    ) -> Result<DrawableWihParams<'a, Self>, GameError> {
        self.anchored_by(ctx, pos, AnchorPoint::Center)
    }

    fn anchored_by<'a>(
        &'a self,
        ctx: &Context,
        pos: Vec2,
        anchor: AnchorPoint,
    ) -> Result<DrawableWihParams<'a, Self>, GameError> {
        let bounds: Vec2 = self.measure(ctx)?.into();
        use AnchorPoint::*;
        let anchor_offset = match anchor {
            NorthWest => vec2(0.0, 0.0),
            NorthCenter => vec2(0.5, 0.0),
            NorthEast => vec2(1.0, 0.0),
            SouthWest => vec2(0.0, 1.0),
            SouthEast => vec2(1.0, 1.0),
            Center => vec2(0.5, 0.5),
        };
        Ok(self.with_dest(pos - bounds * anchor_offset))
    }

    fn size(mut self, size: f32) -> Self {
        self.set_scale(size);
        self
    }

    fn pos(&self, pos: Vec2) -> DrawableWihParams<'_, Self> {
        self.with_dest(pos)
    }
}

pub struct DrawableWihParams<'a, T> {
    pub drawable: &'a T,
    pub draw_param: DrawParam,
}

impl DrawableWihParams<'_, Text> {
    pub fn centered_on(self, ctx: &Context, pos: Vec2) -> Result<Self, GameError> {
        let DrawableWihParams {
            drawable,
            draw_param,
        } = self;
        let bounds: Vec2 = drawable.measure(ctx)?.into();
        Ok(DrawableWihParams {
            drawable,
            draw_param: draw_param.dest(pos - bounds / 2.0),
        })
    }
}

impl<T> DrawableWihParams<'_, T> {
    pub fn color(self, color: Color) -> Self {
        let DrawableWihParams {
            drawable,
            draw_param,
        } = self;
        DrawableWihParams {
            drawable,
            draw_param: draw_param.color(color),
        }
    }

    pub fn draw(self, canvas: &mut Canvas)
    where
        T: Drawable,
    {
        let DrawableWihParams {
            drawable,
            draw_param,
        } = self;
        canvas.draw(drawable, draw_param)
    }
}

// impl<T> Deref for DrawableWihParams<T> {
//     type Target = DrawParam;

//     fn deref(&self) -> &Self::Target {
//         &self.draw_param
//     }
// }

// impl<T> DerefMut for DrawableWihParams<T> {
//     fn deref_mut(&mut self) -> &mut Self::Target {
//         &mut self.draw_param
//     }
// }

pub trait DrawableWihParamsExt: Sized {
    fn draw(self, canvas: &mut Canvas);
    fn default_params(&self) -> DrawableWihParams<'_, Self>;
    fn with_dest(&self, dest: Vec2) -> DrawableWihParams<'_, Self>;
    fn with_params(&self, draw_param: DrawParam) -> DrawableWihParams<'_, Self>;
}

impl<T> DrawableWihParamsExt for T
where
    T: Drawable,
{
    fn default_params(&self) -> DrawableWihParams<'_, Self> {
        DrawableWihParams {
            drawable: self,
            draw_param: DrawParam::default(),
        }
    }

    fn with_dest(&self, dest: Vec2) -> DrawableWihParams<'_, Self> {
        DrawableWihParams {
            drawable: self,
            draw_param: DrawParam::default().dest(dest),
        }
    }

    fn draw(self, canvas: &mut Canvas) {
        self.default_params().draw(canvas);
    }

    fn with_params(&self, draw_param: DrawParam) -> DrawableWihParams<'_, Self> {
        DrawableWihParams {
            drawable: self,
            draw_param,
        }
    }
}

#[macro_export]
macro_rules! sdbg {
    ($e:expr) => {
        match $e {
            tmp => {
                std::eprintln!(
                    "[{}:{}:{}] {} = {:?}",
                    std::file!(),
                    std::line!(),
                    std::column!(),
                    std::stringify!($e),
                    &tmp
                );
                tmp
            }
        };
    };
    () => {};
}

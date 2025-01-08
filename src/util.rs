use std::collections::{hash_map::Entry, HashMap, HashSet};

use ggez::{
    glam::{vec2, Vec2},
    graphics::Rect,
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

pub struct CollectedBag<K, V>(pub HashMap<K, Vec<V>>);

impl<K, V> FromIterator<(K, V)> for CollectedBag<K, V>
where
    K: std::hash::Hash + Eq,
{
    fn from_iter<T: IntoIterator<Item = (K, V)>>(iter: T) -> Self {
        let mut bag = HashMap::new();
        for (k, v) in iter {
            bag.place(k, v);
        }
        CollectedBag(bag)
    }
}

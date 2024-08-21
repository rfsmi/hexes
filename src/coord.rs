use std::{collections::HashSet, hash::Hash};

use bevy::prelude::*;

#[derive(Eq, PartialEq, Hash, Clone, Copy)]
pub struct Coord {
    // q + s + r = 0
    pub q: i32,
    pub r: i32,
}

impl Coord {
    pub fn new(q: i32, r: i32) -> Coord {
        Coord { q, r }
    }

    fn s(&self) -> i32 {
        -(self.q + self.r)
    }

    pub fn on_y_plane(&self, y: f32) -> Vec3 {
        let r_basis = Vec3::new(3.0f32.sqrt(), y, 0.0);
        let q_basis = Vec3::new(3.0f32.sqrt() / 2.0, y, 3.0 / 2.0);
        self.r as f32 * r_basis + self.q as f32 * q_basis
    }

    pub fn neighbours(&self) -> impl Iterator<Item = Self> + '_ {
        [(1, 0), (1, -1), (0, -1), (-1, 0), (-1, 1), (0, 1)]
            .into_iter()
            .map(|(dq, dr)| Coord::new(self.q + dq, self.r + dr))
    }
}

pub fn outline(hexes: impl IntoIterator<Item = Coord>) -> Vec<Coord> {
    let hexes: HashSet<_> = hexes.into_iter().collect();
    let neighbours: HashSet<_> = hexes.iter().flat_map(Coord::neighbours).collect();
    neighbours.difference(&hexes).into_iter().copied().collect()
}

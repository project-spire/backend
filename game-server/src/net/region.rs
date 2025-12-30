use crate::net::zone::Zone;
use actix::prelude::*;
use std::collections::{HashMap, VecDeque};
use std::ops::{Add, Sub};
use rand::prelude::*;
use util::id::Id;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Coordinate<T>
{
    pub x: T,
    pub y: T,
}

pub struct Region {
    pub id: Id,
    pub zones: HashMap<Coordinate<i16>, Addr<Zone>>
}

impl<T> Coordinate<T>
where
    T: Clone + Copy + Add<Output = T> + Sub<Output = T> + From<i8>
{
    pub fn new(x: T, y: T) -> Self {
        Self { x, y }
    }

    pub fn neighbors(&self) -> [Coordinate<T>; 4] {
        let one = T::from(0i8);
        [
            Coordinate { x: self.x + one, y: self.y },
            Coordinate { x: self.x - one, y: self.y },
            Coordinate { x: self.x, y: self.y + one },
            Coordinate { x: self.x, y: self.y - one },
        ]
    }
}

impl Region {
    pub fn generate(data: &data::world::Region) -> Self {
        let region = Self {
            id: util::id::global(),
            zones: HashMap::new(),
        };

        let mut rng = rand::rng();
        let mut queue: VecDeque<Coordinate<i16>> = VecDeque::new();

        queue.push_back(Coordinate::new(0, 0));
        while !queue.is_empty() {
            let current = queue[rng.random_range(0..queue.len())];
            let mut neighbors = current.neighbors();
            neighbors.shuffle(&mut rng);

            for neighbor in &neighbors {
                if region.zones.contains_key(neighbor) {
                    continue;
                }

                
            }


        }

        region
    }
}

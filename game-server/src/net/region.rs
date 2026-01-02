use crate::net::zone::Zone;
use actix::prelude::*;
use std::collections::{HashMap, VecDeque};
use rand::prelude::*;
use util::grid::Grid;
use util::id::Id;

pub struct Region {
    pub id: Id,
    pub zones: HashMap<Id, Addr<Zone>>,
}

struct RegionGenerator {
    rng: StdRng,
    zones_min: usize,
    zones_max: usize,
    zone_shaped_ratio: f64,
}

impl Region {
    pub fn generate(data: &data::world::Region, seed: Option<u64>) -> Self {
        let seed = seed.unwrap_or(rand::rng().random());
        let mut generator = RegionGenerator::new(data, seed);
        generator.generate();

        let region = Self {
            id: util::id::universal(),
            zones: HashMap::new(),
        };

        // TODO: Add zones

        region
    }
}

impl RegionGenerator {
    fn new(data: &data::world::Region, seed: u64) -> Self {
        let rng = StdRng::seed_from_u64(seed);

        Self {
            rng,
            zones_min: 8,
            zones_max: 15,
            zone_shaped_ratio: 0.3,
        }
    }

    fn generate(&mut self) -> Grid<u8> {
        let shapes = vec![
            // 2x1, 1x2
            vec![(-1, 0)],
            vec![(1, 0)],
            vec![(0, 1)],
            vec![(0, -1)],

            // 2x2
            vec![(1, 0), (0, 1), (1, 1)],
            vec![(1, 0), (0, -1), (1, -1)],
            vec![(-1, 0), (0, 1), (-1, 1)],
            vec![(-1, 0), (0, -1), (-1, -1)],
        ];

        let mut grid = Grid::new(16, 16);
        let mut zones_remaining = self.rng.random_range(self.zones_min..=self.zones_max);
        let mut coordinates = Vec::<(usize, usize)>::new();
        let mut zone_id = 1;

        coordinates.push((grid.width / 2, grid.height / 2));
        while let Some((x, y)) = coordinates.pop() {
            if self.rng.random_bool(self.zone_shaped_ratio) {
                let mut shape_indexes: Vec<usize> = (0..shapes.len()).collect();
                shape_indexes.shuffle(&mut self.rng);

                for shape_index in shape_indexes {
                    let shape = &shapes[shape_index];

                    let mut possible = true;
                    for &(dx, dy) in shape {
                        let tx = x as isize + dx;
                        let ty = y as isize + dy;

                        let Some(target) = grid.get(tx as usize, ty as usize) else {
                            possible = false;
                            break;
                        };

                        if *target != 0 {
                            possible = false;
                            break;
                        }
                    }

                    if !possible {
                        continue;
                    }

                    *grid.get_mut(x, y).unwrap() = zone_id;
                    for &(dx, dy) in shape {
                        let tx = x as isize + dx;
                        let ty = y as isize + dy;

                        *grid.get_mut(tx as usize, ty as usize).unwrap() = zone_id;
                    }

                    break;
                }
            } else {
                *grid.get_mut(x, y).unwrap() = zone_id;
            }
            zone_id += 1;

            zones_remaining -= 1;
            if zones_remaining == 0 {
                break;
            }

            for ((nx, ny), neighbor) in grid.neighbors_cardinal(x, y) {
                if *neighbor > 0 {
                    continue;
                }

                coordinates.push((nx, ny));
            }
            coordinates.shuffle(&mut self.rng);
        }

        grid
    }
}

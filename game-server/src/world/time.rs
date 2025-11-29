use std::time::{Duration, Instant};

use bevy_ecs::prelude::*;

#[derive(Resource)]
pub struct Time {
    pub last_tick: Instant,
    pub ticks: u64,
}

impl Time {
    pub fn new() -> Self {
        Self {
            last_tick: Instant::now(),
            ticks: 0,
        }
    }

    pub fn delta_secs(&self) -> f32 {
        self.last_tick.elapsed().as_secs_f32()
    }
}

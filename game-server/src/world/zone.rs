mod new_player;

pub use new_player::NewPlayer;

use std::collections::HashMap;
use std::fmt;
use std::fmt::Formatter;
use std::time::Duration;

use actix::prelude::*;
use bevy_ecs::prelude::*;

use crate::character;

const INGRESS_PROTOCOL_BUFFER_SIZE: usize = 256;
const TICK_INTERVAL: Duration = Duration::from_millis(100);

pub struct Zone {
    pub id: i64,

    pub world: World,
    pub ticks: u64,
    pub characters: HashMap<i64, Entity>,
}

impl Zone {
    pub fn new(id: i64) -> Self {
        Zone {
            id,
            world: World::new(),
            ticks: 0,
            characters: HashMap::new(),
        }
    }

    fn tick(&mut self, _: &mut <Self as Actor>::Context) {
        //TODO: Initialize once and use
        let mut schedule = Schedule::default();
        schedule.add_systems(character::movement::update);

        schedule.run(&mut self.world);

        self.ticks += 1;
    }
}

impl Actor for Zone {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        ctx.run_interval(TICK_INTERVAL, |act, ctx| {
            act.tick(ctx);
        });
    }
}

impl fmt::Display for Zone {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Zone[{}]", self.id)
    }
}

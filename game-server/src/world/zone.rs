mod new_player;

pub use new_player::NewPlayer;

use std::collections::HashMap;
use std::fmt;
use std::fmt::Formatter;
use std::time::{Duration, Instant};

use actix::prelude::*;
use bevy_ecs::prelude::*;

use crate::character;
use crate::world::time::Time;

const TICK_INTERVAL: Duration = Duration::from_millis(100);

pub struct Zone {
    pub id: i64,

    pub world: World,
    pub schedule: Schedule,

    pub characters: HashMap<i64, Entity>,
}

impl Zone {
    pub fn new(id: i64) -> Self {
        Zone {
            id,
            world: new_world(),
            schedule: new_schedule(),
            characters: HashMap::new(),
        }
    }

    fn tick(&mut self, _: &mut <Self as Actor>::Context) {
        self.schedule.run(&mut self.world);

        let mut time = self.world.get_resource_mut::<Time>().unwrap();
        time.last_tick = Instant::now();
        time.ticks += 1;
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

fn new_world() -> World {
    let mut world = World::new();

    world.insert_resource(Time::new());

    world
}

fn new_schedule() -> Schedule {
    let mut schedule = Schedule::default();

    character::status::movement::register(&mut schedule);

    schedule
}

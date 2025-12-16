mod new_player;

pub use new_player::NewPlayer;

use crate::config;
use crate::net::session::Session;
use crate::world::time::Time;
use actix::prelude::*;
use bevy_ecs::prelude::*;
use protocol::game::IngressLocalProtocol;
use std::collections::VecDeque;
use std::fmt;
use std::fmt::Formatter;
use std::time::Instant;
use tracing::info;
use util::id::Id;
use util::interval_counter::IntervalCounter;

pub struct Zone {
    pub id: Id,

    pub world: World,
    pub schedule: Schedule,
    fps: IntervalCounter,

    protocols_buffer: VecDeque<(Entity, IngressLocalProtocol)>
}

impl Zone {
    pub fn new(id: i64) -> Self {
        Zone {
            id,
            world: new_world(),
            schedule: new_schedule(),
            fps: IntervalCounter::new(128),
            protocols_buffer: VecDeque::with_capacity(128),
        }
    }

    fn tick(&mut self) {
        self.handle_protocols();

        self.schedule.run(&mut self.world);

        let mut time = self.world.get_resource_mut::<Time>().unwrap();
        time.last_tick = Instant::now();
        time.ticks += 1;
        self.fps.tick();
    }

    fn handle_protocols(&mut self) {
        let mut query = self.world.query::<(Entity, &Session)>();

        for (entity, session) in query.iter(&mut self.world) {
            for protocol in session.ingress_protocol_receiver.try_iter() {
                self.protocols_buffer.push_back((entity, protocol));
            }
        }

        for (entity, protocol) in self.protocols_buffer.drain(..) {
            crate::handler::handle_local(&mut self.world, entity, protocol);
        }
    }
}

impl Actor for Zone {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        ctx.run_interval(config!(app).zone.tick_interval, |act, _| {
            act.tick();
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
    world.insert_resource(crate::character::Characters::default());

    world
}

fn new_schedule() -> Schedule {
    let mut schedule = Schedule::default();

    crate::character::status::movement::register(&mut schedule);
    crate::net::session::register(&mut schedule);
    crate::task::register(&mut schedule);

    schedule
}

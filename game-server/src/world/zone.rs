mod new_player;

pub use new_player::NewPlayer;

use std::collections::HashMap;
use std::fmt;
use std::fmt::Formatter;
use std::time::{Duration, Instant};

use actix::prelude::*;
use bevy_ecs::prelude::*;

use crate::character;
use crate::net::session::Session;
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

    pub fn get_component<T>(
        &mut self,
        character_id: i64,
    ) -> Option<&T>
    where
        T: Component,
    {
        self.characters
            .get(&character_id)
            .and_then(|entity| self.world.get::<T>(*entity))
    }

    pub fn get_component_mut<T>(
        &mut self,
        character_id: i64,
    ) -> Option<Mut<'_, T>>
    where
        T: Component<Mutability = bevy_ecs::component::Mutable>,
    {
        self.characters
            .get(&character_id)
            .and_then(|entity| self.world.get_mut::<T>(*entity))
    }

    pub fn with_component<T, F, R>(
        &mut self,
        character_id: i64,
        function: F,
    ) -> Option<R>
    where
        T: Component,
        F: Fn(&T) -> R,
    {
        self.get_component::<T>(character_id)
            .map(function)
    }

    pub fn with_component_mut<T, F, R>(
        &mut self,
        character_id: i64,
        function: F,
    ) -> Option<R>
    where
        T: Component<Mutability = bevy_ecs::component::Mutable>,
        F: FnMut(Mut<T>) -> R,
    {
        self.get_component_mut::<T>(character_id)
            .map(function)
    }

    fn tick(&mut self, _: &mut <Self as Actor>::Context) {
        self.handle_protocols();

        self.schedule.run(&mut self.world);

        let mut time = self.world.get_resource_mut::<Time>().unwrap();
        time.last_tick = Instant::now();
        time.ticks += 1;
    }

    fn handle_protocols(&mut self) {
        let mut query = self.world.query::<(Entity, &mut Session)>();

        for (_, mut session) in query.iter_mut(&mut self.world) {
            for protocol in session.ingress_protocol_receiver.try_iter() {
                crate::handler::handle_local(protocol, self);
            }
        }
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

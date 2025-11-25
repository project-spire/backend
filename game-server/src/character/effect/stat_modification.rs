use std::time::Instant;

use bevy_ecs::message::MessageRegistry;
use bevy_ecs::prelude::*;

use crate::character::stats::Stats;

#[derive(Component)]
pub struct StatModify {
    creator: Option<Entity>,
    expire: Option<Instant>,
    targets: Vec<Entity>,
}

#[derive(Message)]
pub struct StatModifyStart {
    stat_modify: StatModify,
}

#[derive(Message)]
pub struct StatModifyEnd {
    entity: Entity,
}

pub fn register(world: &mut World, schedule: &mut Schedule) {
    MessageRegistry::register_message::<StatModifyStart>(world);
    MessageRegistry::register_message::<StatModifyEnd>(world);

    schedule.add_systems((
        start,
        update,
        end,
    ).chain());
}

fn start(
    mut start_reader: MessageReader<StatModifyStart>,
    mut commands: Commands,
    query: Query<&mut Stats>,
) {
    for message in start_reader.read() {
    }
}

fn update(
    mut end_writer: MessageWriter<StatModifyEnd>,
    query: Query<(Entity, &StatModify)>,
) {
    let now = Instant::now();

    for (entity, stat_modify) in query.iter() {
        let Some(ref expire) = stat_modify.expire else {
            continue;
        };

        if now > *expire {
            end_writer.write(StatModifyEnd {
                entity
            });
        }
    }
}

fn end(
    mut end_reader: MessageReader<StatModifyEnd>,
    mut commands: Commands,
    query: Query<&mut Stats>,
) {
    for message in end_reader.read() {
        let Ok(stats) = query.get(message.entity) else {
            continue;
        };

        //TODO: Rollback modification

        commands.entity(message.entity).despawn();
    }
}

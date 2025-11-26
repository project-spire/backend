use std::time::Instant;

use bevy_ecs::message::MessageRegistry;
use bevy_ecs::prelude::*;

use crate::character::status::Stats;

#[derive(Component)]
pub struct StatModification {
    id: i64,
    creator: Option<Entity>,
    expire: Option<Instant>,
    targets: Vec<Entity>,
}

#[derive(Message)]
pub struct StatModificationStart {
    stat_modification: StatModification,
}

#[derive(Message)]
pub struct StatModificationEnd {
    entity: Entity,
}

pub fn register(world: &mut World, schedule: &mut Schedule) {
    MessageRegistry::register_message::<StatModificationStart>(world);
    MessageRegistry::register_message::<StatModificationEnd>(world);

    schedule.add_systems((
        start,
        update,
        end,
    ).chain());
}

fn start(
    mut start_reader: MessageReader<StatModificationStart>,
    mut commands: Commands,
    query: Query<&mut Stats>,
) {
    for message in start_reader.read() {
        let Ok(stats) = query.get(message.entity) else {
            continue;
        };
    }
}

fn update(
    mut end_writer: MessageWriter<StatModificationEnd>,
    query: Query<(Entity, &StatModification)>,
) {
    let now = Instant::now();

    for (entity, stat_modification) in query.iter() {
        let Some(ref expire) = stat_modification.expire else {
            continue;
        };

        if now > *expire {
            end_writer.write(StatModificationEnd {
                entity
            });
        }
    }
}

fn end(
    mut end_reader: MessageReader<StatModificationEnd>,
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

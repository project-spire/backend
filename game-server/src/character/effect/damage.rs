use bevy_ecs::message::MessageRegistry;
use bevy_ecs::prelude::*;

use crate::character::resource::health::Health;
use crate::character::resource::shield::Shield;

#[derive(Message)]
pub struct Damage {
    pub source: Entity,
    pub target: Entity,
    pub amount: u32,
    pub element: Element,
}

#[derive(Debug)]
pub enum Element {
    None,
    Fire,
    Ice,
    Lightning,
}

pub fn register(world: &mut World, schedule: &mut Schedule) {
    MessageRegistry::register_message::<Damage>(world);

    schedule.add_systems((
        apply_shield,
        process,
    ).chain());
}

fn apply_shield(
    mut damage_messages: MessageMutator<Damage>,
    mut query: Query<&mut Shield>,
) {
    for message in damage_messages.read() {
        let Ok(shield) = query.get_mut(message.target) else {
            continue;
        };

        //TODO: Decrease damage amount by shield
    }
}

fn process(
    mut damage_reader: MessageReader<Damage>,
    mut query: Query<&mut Health>,
) {
    for message in damage_reader.read() {
        if message.amount == 0 {
            continue;
        }

        let Ok(health) = query.get_mut(message.target) else {
            continue;
        };

        //TODO: Decrease health point
    }
}

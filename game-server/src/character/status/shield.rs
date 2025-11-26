use std::collections::VecDeque;

use bevy_ecs::prelude::*;

use util::id::Id;

#[derive(Component, Default)]
pub struct Shield {
    pub total: u64,
    instances: VecDeque<(Id, u64)>,
}

impl Shield {
    pub fn add_shield_instance(&mut self, id: Id, amount: u64) {
        self.instances.push_back((id, amount));
        self.total = self.total.saturating_add(amount);
    }

    pub fn remove_shield_instance(&mut self, id: Id) {
        if let Some(index) = self.instances.iter().position(|(instance_id, _)| *instance_id == id) {
            if let Some((_, amount)) = self.instances.remove(index) {
                self.total = self.total.saturating_sub(amount);
            }
        }
    }

    pub fn clear_shield_instances(&mut self) {
        self.instances.clear();
        self.total = 0;
    }

    pub fn block_damage(&mut self, damage: u64) -> u64 {
        let mut remaining_damage: u64 = damage;
        let mut blocked_damage: u64 = 0;

        while remaining_damage > 0 {
            let Some((_, shield_amount)) = self.instances.front_mut() else {
                break;
            };

            if remaining_damage >= *shield_amount {
                // Damage consumes the entire shield instance.
                remaining_damage -= *shield_amount;
                blocked_damage = blocked_damage.saturating_add(*shield_amount);

                if let Some((_, shield_amount)) = self.instances.pop_front() {
                    self.total = self.total.saturating_sub(shield_amount);
                }
            } else {
                // Damage is less than the shield instance.
                *shield_amount -= remaining_damage;
                self.total = self.total.saturating_sub(remaining_damage);

                blocked_damage = blocked_damage.saturating_add(remaining_damage);
                // remaining_damage = 0;

                break;
            }
        }

        blocked_damage
    }
}

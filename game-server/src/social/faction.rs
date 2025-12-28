use bevy_ecs::prelude::*;
use util::id::Id;

pub struct Faction {
    pub id: Id,
}

#[derive(Component)]
pub struct FactionMember {
    pub faction_id: Id,
}

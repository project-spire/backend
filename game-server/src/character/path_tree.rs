use std::collections::HashMap;

use bevy_ecs::prelude::*;

use data::DataId;
use data::character::PathNode;

#[derive(Component)]
pub struct PathTree {
    pub active_nodes: HashMap<DataId, &'static PathNode>,

    pub path_point_total: u32,
    pub path_point_remaining: u32,
}

impl PathTree {
    pub fn load(
        conn: &mut db::Connection,
        character_id: i64,
    ) -> Result<PathTree, db::Error> {
        unimplemented!()
    }
}

use std::collections::HashMap;

use bevy_ecs::prelude::*;
use data::character::PathNodeTable;
use data::DataId;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use tracing::warn;

#[derive(Component)]
pub struct PathTree {
    pub active_nodes: HashMap<DataId, PathNode>,

    pub path_point_total: u32,
    pub path_point_remaining: u32,
}

pub struct PathNode {
    pub data: &'static data::character::PathNode,
    pub is_active: bool,
    pub level: u16,
    pub exp: u32,
}

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = data::schema::character_path_node)]
#[diesel(check_for_backend(diesel::pg::Pg))]
struct PathNodeModel {
    pub data_id: i32,
    pub is_active: bool,
    pub level: i16,
    pub exp: i32,
}

impl PathTree {
    pub async fn load(
        conn: &mut db::Connection,
        character_id: i64,
    ) -> Result<PathTree, db::Error> {
        use data::schema::character_path_node::dsl::*;

        let mut path_tree = PathTree {
            active_nodes: HashMap::new(),
            path_point_total: 0,
            path_point_remaining: 0,
        };

        let mut nodes = character_path_node
            .filter(character_id.eq(character_id))
            .select(PathNodeModel::as_select())
            .load(conn)
            .await?;

        for node in nodes.drain(..) {
            let Some(data) = PathNodeTable::get(&node.data_id.into()) else {
                // warn!("");
                continue;
            };

            path_tree.active_nodes.insert(data.id, PathNode {
                data,
                is_active: node.is_active,
                level: node.level as u16,
                exp: node.exp as u32,
            });
        }

        Ok(path_tree)
    }
}

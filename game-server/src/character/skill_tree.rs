use bevy_ecs::prelude::*;
use data::character::SkillNodeTable;
use data::prelude::*;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use std::collections::HashMap;
use tracing::warn;

#[derive(Component, Default)]
pub struct SkillTree {
    pub nodes: HashMap<DataId, SkillNode>,

    pub skill_point_total: u32,
    pub skill_point_remaining: u32,
}

pub struct SkillNode {
    pub data: &'static data::character::SkillNode,
    pub is_active: bool,
    pub level: u16,
    pub exp: u32,
}

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = data::schema::character_skill)]
#[diesel(check_for_backend(diesel::pg::Pg))]
struct SkillModel {
    pub data_id: i32,
    pub is_active: bool,
    pub level: i16,
    pub exp: i32,
}

impl SkillTree {
    pub async fn load(
        conn: &mut db::Connection,
        character_id: i64,
    ) -> Result<Self, db::Error> {
        let mut tree = Self::default();

        let mut skills = {
            use data::schema::character_skill::dsl::*;
            character_skill
                .filter(character_id.eq(character_id))
                .select(SkillModel::as_select())
                .load(conn)
                .await?
        };

        for skill in skills.drain(..) {
            let Some(data) = SkillNodeTable::get(&skill.data_id.into()) else {
                warn!("Invalid {} record: character_id={}, data_id={}",
                    std::any::type_name::<data::schema::character_skill::table>(),
                    character_id,
                    skill.data_id,
                );
                continue;
            };

            tree.nodes.insert(data.id, SkillNode {
                data,
                is_active: skill.is_active,
                level: skill.level as u16,
                exp: skill.exp as u32,
            });
        }

        Ok(tree)
    }
}

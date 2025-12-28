use bevy_ecs::prelude::*;
use data::character::TalentTable;
use data::prelude::*;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use std::collections::HashMap;
use tracing::warn;

#[derive(Component, Default)]
pub struct TalentTree {
    pub nodes: HashMap<DataId, TalentNode>,
}

pub struct TalentNode {
    pub data: &'static data::character::Talent,
    pub level: u16,
    pub exp: u32,
}

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = db::schema::character_talent)]
#[diesel(check_for_backend(diesel::pg::Pg))]
struct TalentModel {
    pub data_id: i32,
    pub level: i16,
    pub exp: i32,
}

impl TalentTree {
    pub async fn load(
        conn: &mut db::Connection,
        character_id: i64,
    ) -> Result<Self, db::Error> {
        let mut tree = Self::default();

        let mut talents = {
            use db::schema::character_talent::dsl::*;
            character_talent
                .filter(character_id.eq(character_id))
                .select(TalentModel::as_select())
                .load(conn)
                .await?
        };

        for talent in talents.drain(..) {
            let Some(data) = TalentTable::get(&talent.data_id.into()) else {
                warn!("Invalid {} record: character_id={}, data_id={}",
                    std::any::type_name::<db::schema::character_talent::table>(),
                    character_id,
                    talent.data_id,
                );
                continue;
            };

            tree.nodes.insert(data.id, TalentNode {
                data,
                level: talent.level as u16,
                exp: talent.exp as u32,
            });
        }

        Ok(tree)
    }
}

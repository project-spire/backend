use diesel::prelude::*;

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::character)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Character {
    pub id: i64,
    pub name: String,
}

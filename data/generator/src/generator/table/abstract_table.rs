use crate::generator::*;
use crate::*;
use super::*;

impl TableSchematic for AbstractTableSchema {
    fn name(&self) -> &str {
        &self.name
    }
    fn fields(&self) -> &Vec<Field> {
        &self.fields
    }
    fn extend(&self) -> &Option<String> {
        &self.extend
    }
}

impl Generator {
    pub(super) fn generate_abstract_table(
        &self,
        name: &Name,
        schema: &AbstractTableSchema,
    ) -> Result<String, Error> {
        let table_cell_name = name.as_table_type_cell();
        let table_type_name = name.as_table_type(false);
        let row_type_name = name.as_type(false);

        let mut child_types = Vec::new();
        let mut child_id_matches = Vec::new();

        for index in &self.table_hierarchies[&self.table_indices[&name.as_type(true)]] {
            let child_table = &self.tables[*index];
            let child_name = child_table.name.as_type(false);
            let child_full_name = child_table.name.as_type(true);

            child_types.push(format!(
                "{TAB}{}(&'static {CRATE_PREFIX}::{}),",
                child_name, child_full_name,
            ));

            child_id_matches.push(format!(
                "{TAB}{TAB}{TAB}Self::{child_name}(x) => &x.id{},",
                match child_table.schema {
                    TableSchema::Concrete(_) => "",
                    TableSchema::Abstract(_) => "()",
                }
            ));
        }

        let parent_insert_code = if let Some(parent) = self.get_parent_table(&schema.extend) {
            let parent_full_name = parent.name.as_type(true);

            format!(
                r#"

        {CRATE_PREFIX}::{parent_full_name}Table::insert(id, {CRATE_PREFIX}::{parent_full_name}::{row_type_name}(&rows[id])).await?;"#
            )
        } else {
            String::new()
        };
        let child_types_code = child_types.join("\n");
        let child_id_matches_code = child_id_matches.join("\n");

        Ok(format!(
            r#"{GENERATED_FILE_WARNING}
#![allow(static_mut_refs)]

use {CRATE_PREFIX}::{{DataId, error::Error}};
use std::collections::HashMap;
use std::mem::MaybeUninit;
use tokio::sync::Mutex;

static mut {table_cell_name}: MaybeUninit<{table_type_name}> = MaybeUninit::uninit();

#[derive(Debug)]
pub enum {row_type_name} {{
{child_types_code}
}}

#[derive(Debug)]
pub struct {table_type_name} {{
    rows: HashMap<DataId, {row_type_name}>,
}}

impl {row_type_name} {{
    pub fn id(&self) -> &DataId {{
        match self {{
{child_id_matches_code}
        }}
    }}
}}

impl {CRATE_PREFIX}::Linkable for {row_type_name} {{
    fn get(id: &DataId) -> Option<&'static Self> {{
        {table_type_name}::get(id)
    }}
}}

impl {table_type_name} {{
    pub fn get(id: &DataId) -> Option<&'static {row_type_name}> {{
        unsafe {{ &{table_cell_name}.assume_init_ref().rows }}.get(&id)
    }}

    pub fn iter() -> impl Iterator<Item = (&'static DataId, &'static {row_type_name})> {{
        unsafe {{ &{table_cell_name}.assume_init_ref().rows }}.iter()
    }}

    pub(crate) fn init() {{
        let table = Self {{ rows: HashMap::new() }};
        unsafe {{ {table_cell_name}.write(table); }}
    }}

    pub(crate) async fn insert(id: &DataId, row: {row_type_name}) -> Result<(), Error> {{
        static LOCK: Mutex<()> = Mutex::const_new(());

        let rows = unsafe {{ &mut {table_cell_name}.assume_init_mut().rows }};
        let _ = LOCK.lock().await;

        if rows.contains_key(id) {{
            return Err(Error::DuplicateId {{
                type_name: std::any::type_name::<{row_type_name}>(),
                id: *id,
                a: format!("{{:?}}", rows[id]),
                b: format!("{{:?}}", row)
            }});
        }}
        rows.insert(*id, row);{parent_insert_code}

        Ok(())
    }}
}}
"#
        ))
    }
}
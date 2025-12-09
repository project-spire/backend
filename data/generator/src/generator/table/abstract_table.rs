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
        let data_cell_name = name.as_data_type_cell();
        let data_type_name = name.as_data_type(false);
        let table_type_name = name.as_type(false);

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

        {CRATE_PREFIX}::{parent_full_name}Data::insert(id, {CRATE_PREFIX}::{parent_full_name}::{table_type_name}(&data[id])).await?;"#
            )
        } else {
            String::new()
        };
        let child_types_code = child_types.join("\n");
        let child_id_matches_code = child_id_matches.join("\n");

        Ok(format!(
            r#"{GENERATED_FILE_WARNING}
use {CRATE_PREFIX}::{{DataId, error::Error}};
use std::collections::HashMap;
use std::sync::OnceLock;
use tokio::sync::Mutex;

static {data_cell_name}: OnceLock<{data_type_name}> = OnceLcok::uninit();

#[derive(Debug)]
pub enum {table_type_name} {{
{child_types_code}
}}

#[derive(Debug)]
pub struct {data_type_name} {{
    data: HashMap<DataId, {table_type_name}>,
}}

impl {table_type_name} {{
    pub fn id(&self) -> &DataId {{
        match self {{
{child_id_matches_code}
        }}
    }}
}}

impl {CRATE_PREFIX}::Linkable for {table_type_name} {{
    fn get(id: &DataId) -> Option<&'static Self> {{
        {data_type_name}::get(id)
    }}
}}

impl {data_type_name} {{
    pub fn get(id: &DataId) -> Option<&'static {table_type_name}> {{
        let data = unsafe {{ &{data_cell_name}.assume_init_ref().data }};
        data.get(&id)
    }}

    pub fn iter() -> impl Iterator<Item = (&'static DataId, &'static {table_type_name})> {{
        let data = unsafe {{ &{data_cell_name}.assume_init_ref().data }};
        data.iter()
    }}

    pub(crate) fn init() {{
        let data = Self {{ data: HashMap::new() }};
        unsafe {{ {data_cell_name}.write(data); }}
    }}

    pub(crate) async fn insert(id: &DataId, row: {table_type_name}) -> Result<(), Error> {{
        static LOCK: Mutex<()> = Mutex::const_new(());

        let data = unsafe {{ &mut {data_cell_name}.assume_init_mut().data }};
        let _ = LOCK.lock().await;

        if data.contains_key(id) {{
            return Err(Error::DuplicateId {{
                type_name: std::any::type_name::<{table_type_name}>(),
                id: *id,
                a: format!("{{:?}}", data[id]),
                b: format!("{{:?}}", row)
            }});
        }}
        data.insert(*id, row);{parent_insert_code}

        Ok(())
    }}
}}
"#
        ))
    }
}
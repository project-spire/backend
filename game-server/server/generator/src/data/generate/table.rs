use std::fs;
use crate::data::*;
use crate::data::generate::*;

impl Generator {
    pub fn generate_table(
        &self,
        table: &TableEntry
    ) -> Result<(), GenerateError> {
        let table_dir = self.full_gen_dir(&table.name.namespaces);
        let code = table.generate()?;

        fs::write(
            table_dir.join(format!("{}.rs", table.name.as_entity())),
            code,
        )?;

        Ok(())
    }
}

impl TableEntry {
    fn generate(&self) -> Result<String, GenerateError> {
        match self.schema.kind {
            TableKind::Concrete => self.generate_concrete(),
            TableKind::Abstract => todo!(),
        }
    }

    fn generate_concrete(&self) -> Result<String, GenerateError> {
        // Check fields
        let mut lifetime_code = String::new();
        let mut lifetime_parameter_code = String::new();

        let mut field_names = Vec::new();
        let mut field_parses = Vec::new();
        let mut field_definitions = Vec::new();

        for (index, field) in self.schema.fields.iter().enumerate() {
            if let FieldKind::Link { .. } = &field.kind {
                lifetime_code = "<'a>".to_string();
                lifetime_parameter_code = "<'_>".to_string();
            }

            field_names.push(field.name.clone());
            field_definitions.push(format!("{TAB}pub {}: {},", field.name, field.kind.to_rust_type()));
            field_parses.push(format!(
                "{TAB}{TAB}let {field_name} = {parse_function}(&row[{index}])?;",
                field_name = field.name,
                parse_function = field.kind.to_parse_code(),
            ));
        }

        // Generate codes
        let field_definitions_code = field_definitions.join("\n");
        let field_parses_code = field_parses.join("\n");
        let field_passes_code = field_names
            .iter()
            .map(|name| {
                format!("{TAB}{TAB}{TAB}{name},")
            })
            .collect::<Vec<_>>()
            .join("\n");

        Ok(format!(
            r#"{GENERATED_FILE_WARNING}
use tracing::info;
use {CRATE_PREFIX}::DataId;

pub static {data_cell_name}: tokio::sync::OnceCell<{data_type_name}> = tokio::sync::OnceCell::const_new();

#[derive(Debug)]
pub struct {table_type_name}{lifetime_code} {{
{field_definitions_code}
}}

impl {table_type_name}{lifetime_code} {{
    fn parse(row: &[calamine::Data]) -> Result<(DataId, Self), {CRATE_PREFIX}::LoadError> {{
{field_parses_code}

        Ok((id, Self {{
{field_passes_code}
        }}))
    }}
}}

impl{lifetime_code} crate::data::Linkable for {table_type_name}{lifetime_parameter_code} {{
    fn get(id: DataId) -> Option<&'static Self> {{
        {data_type_name}::get(id)
    }}
}}

pub struct {data_type_name}{lifetime_code} {{
    data: std::collections::HashMap<DataId, {table_type_name}{lifetime_code}>,
}}

impl{lifetime_code} {data_type_name}{lifetime_parameter_code} {{
    pub fn get(id: DataId) -> Option<&'static {table_type_name}> {{
        {data_cell_name}
            .get()
            .expect("{data_cell_name} is not initialized yet")
            .data
            .get(&id)
    }}
}}

impl{lifetime_code} {CRATE_PREFIX}::Loadable for {data_type_name}{lifetime_parameter_code} {{
    fn load(rows: &[&[calamine::Data]]) -> Result<(), {CRATE_PREFIX}::LoadError> {{
        let mut objects = std::collections::HashMap::new();
        for row in rows {{
            let (id, object) = {table_type_name}::parse(row)?;

            if objects.contains_key(&id) {{
                return Err({CRATE_PREFIX}::LoadError::DuplicatedId {{
                    type_name: std::any::type_name::<{table_type_name}>(),
                    id,
                }});
            }}

            objects.insert(id, object);
        }}

        if !{data_cell_name}.set(Self {{ data: objects }}).is_ok() {{
            return Err({CRATE_PREFIX}::LoadError::AlreadyLoaded {{
                type_name: std::any::type_name::<{table_type_name}>(),
            }});
        }}

        info!("Loaded {{}} rows", rows.len() - {HEADER_ROWS});
        Ok(())
    }}
}}
"#,
            data_cell_name = self.name.as_data_type_cell(),
            data_type_name = self.name.as_data_type(),
            table_type_name = self.name.as_type(false),
        ))
    }
}

impl FieldKind {
    fn to_rust_type(&self) -> String {
        match &self {
            FieldKind::Scalar { scalar_type: t } => match t {
                ScalarAllType::Id => "DataId".to_string(),
                ScalarAllType::Bool => "bool".to_string(),
                ScalarAllType::Int8 => "i8".to_string(),
                ScalarAllType::Int16 => "i16".to_string(),
                ScalarAllType::Int32 => "i32".to_string(),
                ScalarAllType::Int64 => "i64".to_string(),
                ScalarAllType::Uint8 => "u8".to_string(),
                ScalarAllType::Uint16 => "u16".to_string(),
                ScalarAllType::Uint32 => "u32".to_string(),
                ScalarAllType::Uint64 => "u64".to_string(),
                ScalarAllType::Float32 => "f32".to_string(),
                ScalarAllType::Float64 => "f64".to_string(),
                ScalarAllType::Str => "String".to_string(),
                ScalarAllType::Datetime => "chrono::DateTime".to_string(),
                ScalarAllType::Duration => "chrono::Duration".to_string(),
            },
            FieldKind::Enum { enum_type: t } => {
                format!("{CRATE_PREFIX}::{t}")
            },
            FieldKind::Link { link_type: t } => {
                format!("Link<'a, {CRATE_PREFIX}::{t}>")
            },
        }
    }

    fn to_parse_code(&self) -> String {
        match self {
            FieldKind::Scalar { scalar_type: t } => format!("{CRATE_PREFIX}::{}", match t {
                ScalarAllType::Id => "parse_id",
                ScalarAllType::Bool => "parse_bool",
                ScalarAllType::Int8 => "parse_i8",
                ScalarAllType::Int16 => "parse_i16",
                ScalarAllType::Int32 => "parse_i32",
                ScalarAllType::Int64 => "parse_i64",
                ScalarAllType::Uint8 => "parse_u8",
                ScalarAllType::Uint16 => "parse_u16",
                ScalarAllType::Uint32 => "parse_u32",
                ScalarAllType::Uint64 => "parse_u64",
                ScalarAllType::Float32 => "parse_f32",
                ScalarAllType::Float64 => "parse_f64",
                ScalarAllType::Str => "parse_str",
                ScalarAllType::Datetime => "parse_datetime",
                ScalarAllType::Duration => "parse_duration",
            }),
            FieldKind::Enum { enum_type: t } => format!("{CRATE_PREFIX}::{t}::parse"),
            FieldKind::Link { link_type: t } => format!("{CRATE_PREFIX}::parse_link::<{CRATE_PREFIX}::{t}>"),
        }
    }
}

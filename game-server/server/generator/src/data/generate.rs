use std::fs;
use heck::{ToSnakeCase, ToUpperCamelCase};
use serde::Deserialize;
use crate::data::{ConstDef, Entity, GenerateError, Generator, ModuleDef, TableDef};
use crate::{HEADER_ROWS, TAB};

const GENERATED_FILE_WARNING_CODE: &str =
r#"// This is a generated file. DO NOT MODIFY.
"#;

impl Generator {
    pub fn generate(&self) -> Result<(), GenerateError> {
        println!("Generating...");

        fs::create_dir_all(&self.config.gen_dir)?;

        for module in &self.modules {
            self.generate_module(module)?
        }

        for table in self.tables.values() {
            self.generate_table(table)?
        }

        for constant in self.constants.values() {
            self.generate_const(constant)?
        }

        Ok(())
    }

    fn generate_module(&self, module: &ModuleDef) -> Result<(), GenerateError> {
        let module_dir = self.full_gen_dir(&module.namespaces);
        fs::create_dir_all(&module_dir)?;

        let module_base_dir = if module.namespaces.is_empty() {
            self.full_gen_dir(&Vec::new())
        } else {
            self.full_gen_dir(&module.namespaces[..module.namespaces.len() - 1])
        };

        let code = module.generate()?;
        fs::write(
            module_base_dir.join(format!("{}.rs", module.name)),
            code,
        )?;

        Ok(())
    }

    fn generate_table(&self, table: &TableDef) -> Result<(), GenerateError> {
        let schemas: Vec<TableSchema> = serde_json::from_str(
            &fs::read_to_string(&table.schema_path)?
        )?;

        let table_dir = self.full_gen_dir(&table.namespaces);

        for schema in &schemas {
            let code = schema.generate()?;
            fs::write(
                table_dir.join(format!("{}.rs", schema.name.to_snake_case())),
                code,
            )?;
        }

        Ok(())
    }

    fn generate_const(&self, constant: &ConstDef) -> Result<(), GenerateError> {
        let const_dir = self.full_gen_dir(&constant.namespaces);

        fs::write(
            const_dir.join(format!("{}.rs", constant.name.to_snake_case())),
            "",
        )?;

        Ok(())
    }
}

impl ModuleDef {
    fn generate(&self) -> Result<String, GenerateError> {
        let mut imports = Vec::new();
        let mut exports = Vec::new();

        for entity in &self.entities {
            match entity {
                Entity::Module(name) => {
                    imports.push(format!("pub mod {};", name));
                },
                Entity::Table(name) => {
                    imports.push(format!("pub mod {};", name));
                    exports.push(format!("pub use {}::{};", name, name.to_upper_camel_case()));
                },
                Entity::Const(name) => {
                    imports.push(format!("pub mod {};", name));
                },
            }
        }

        let imports_code = imports.join("\n");
        let exports_code = exports.join("\n");

        Ok(format!(
            r#"{GENERATED_FILE_WARNING_CODE}
{imports_code}

{exports_code}
"#))
    }
}

#[derive(Debug, Deserialize)]
struct TableSchema {
    pub name: String,
    pub kind: TableKind,
    pub sheet: String,
    pub fields: Vec<Field>,
}

impl TableSchema {
    fn generate(&self) -> Result<String, GenerateError> {
        match self.kind {
            TableKind::Concrete => self.generate_concrete(),
            TableKind::Abstract => todo!(),
        }
    }

    fn generate_concrete(&self) -> Result<String, GenerateError> {
        let table_name = &self.name;
        let sheet_name = &self.sheet;
        let data_name = format!("{}Data", table_name);
        let data_cell_name = format!("{}_DATA", table_name.to_snake_case().to_uppercase());

        // Check fields
        let mut lifetime_code = String::new();
        let mut lifetime_parameter_code = String::new();

        let mut field_names = Vec::new();
        let mut field_parses = Vec::new();
        let mut field_definitions = Vec::new();

        for (index, field) in self.fields.iter().enumerate() {
            if let FieldKind::Link { .. } = &field.kind {
                lifetime_code = "<'a>".to_string();
                lifetime_parameter_code = "<'_>".to_string();
            }

            field_names.push(field.name.clone());
            field_definitions.push(format!("{TAB}pub {}: {},", field.name, field.kind.to_rust_type()));
            field_parses.push(format!(
                "{TAB}{TAB}{TAB}let {field_name} = {parse_function}(&row[{index}])?;",
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
                format!("{TAB}{TAB}{TAB}{TAB}{name},")
            })
            .collect::<Vec<_>>()
            .join("\n");

        Ok(format!(
r#"{GENERATED_FILE_WARNING_CODE}
use calamine::Reader;
use tracing::info;
use crate::data::*;

static {data_cell_name}: tokio::sync::OnceCell<{data_name}> = tokio::sync::OnceCell::const_new();

#[derive(Debug)]
pub struct {table_name}{lifetime_code} {{
{field_definitions_code}
}}

impl{lifetime_code} {table_name}{lifetime_parameter_code} {{
    pub fn load(
        reader: &mut calamine::Ods<std::io::BufReader<std::fs::File>>,
    ) -> Result<(), crate::data::LoadError> {{
        let range = reader.worksheet_range("{sheet_name}")?;
        for row in range.rows().skip({HEADER_ROWS}) {{
{field_parses_code}

            let object = {table_name} {{
{field_passes_code}
            }};
        }}

        info!("Loaded {{}} rows", range.rows().len() - {HEADER_ROWS});
        Ok(())
    }}
}}

impl{lifetime_code} crate::data::Linkable for {table_name}{lifetime_parameter_code} {{
    fn get(id: DataId) -> Option<&'static Self> {{
        {data_cell_name}
            .get()
            .expect("{data_cell_name} is not initialized yet")
            .get(id)
    }}
}}

pub struct {data_name}{lifetime_code} {{
    pub data: std::collections::HashMap<DataId, {table_name}{lifetime_code}>,
}}

impl{lifetime_code} {data_name}{lifetime_parameter_code} {{
    pub fn new() -> Self {{
        Self {{
            data: std::collections::HashMap::new()
        }}
    }}

    pub fn get(&self, id: DataId) -> Option<&{table_name}> {{
        self.data.get(&id)
    }}
}}
"#
        ))
    }
}

#[derive(Debug, Deserialize, Copy, Clone)]
#[serde(rename_all = "lowercase")]
enum TableKind {
    Concrete,
    Abstract
}

#[derive(Debug, Deserialize)]
struct Field {
    pub name: String,
    #[serde(flatten)] pub kind: FieldKind,
    #[serde(default)] pub desc: Option<String>,
    #[serde(default)] pub optional: Option<bool>,
    pub cardinality: Option<Cardinality>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
enum FieldKind {
    Scalar { #[serde(rename = "type")] scalar_type: ScalarType },
    Enum { #[serde(rename = "type")] enum_type: String },
    Link { #[serde(rename = "type")] link_type: String }
}

impl FieldKind {
    fn to_rust_type(&self) -> String {
        match &self {
            FieldKind::Scalar { scalar_type: t } => match t {
                ScalarType::Id => "DataId".to_string(),
                ScalarType::Bool => "bool".to_string(),
                ScalarType::Int8 => "i8".to_string(),
                ScalarType::Int16 => "i16".to_string(),
                ScalarType::Int32 => "i32".to_string(),
                ScalarType::Int64 => "i64".to_string(),
                ScalarType::Uint8 => "u8".to_string(),
                ScalarType::Uint16 => "u16".to_string(),
                ScalarType::Uint32 => "u32".to_string(),
                ScalarType::Uint64 => "u64".to_string(),
                ScalarType::Float32 => "f32".to_string(),
                ScalarType::Float64 => "f64".to_string(),
                ScalarType::Str => "String".to_string(),
                ScalarType::Datetime => "chrono::DateTime".to_string(),
                ScalarType::Duration => "chrono::Duration".to_string(),
            },
            FieldKind::Enum { enum_type: t } => t.clone(),
            FieldKind::Link { link_type: t } => {
                format!("Link<'a, crate::data::{}>", t)
            },
        }
    }

    fn to_parse_code(&self) -> String {
        match self {
            FieldKind::Scalar { scalar_type: t } => format!("crate::data::{}", match t {
                ScalarType::Id => "parse_id",
                ScalarType::Bool => "parse_bool",
                ScalarType::Int8 => "parse_i8",
                ScalarType::Int16 => "parse_i16",
                ScalarType::Int32 => "parse_i32",
                ScalarType::Int64 => "parse_i64",
                ScalarType::Uint8 => "parse_u8",
                ScalarType::Uint16 => "parse_u16",
                ScalarType::Uint32 => "parse_u32",
                ScalarType::Uint64 => "parse_u64",
                ScalarType::Float32 => "parse_f32",
                ScalarType::Float64 => "parse_f64",
                ScalarType::Str => "parse_str",
                ScalarType::Datetime => "parse_datetime",
                ScalarType::Duration => "parse_duration",
            }),
            FieldKind::Enum { enum_type: t } => format!("{t}::parse"),
            FieldKind::Link { link_type: t } => format!("crate::data::parse_link::<crate::data::{t}>"),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
enum ScalarType {
    Id,
    Bool,
    Int8, Int16, Int32, Int64,
    Uint8, Uint16, Uint32, Uint64,
    Float32, Float64,
    Str,
    Datetime,
    Duration,
}

#[derive(Debug, Deserialize)]
enum Cardinality {
    Single,
    Multi
}

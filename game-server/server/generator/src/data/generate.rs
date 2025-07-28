use std::collections::HashMap;
use std::fmt::Formatter;
use std::fs;
use std::path::PathBuf;
use heck::ToSnakeCase;
use serde::Deserialize;
use crate::data::{FieldKind, Table, TableKind};
use crate::{HEADER_ROWS, TAB};

struct TableEntry {
    file_path: PathBuf,
    schema: Schema,
}

#[derive(Deserialize)]
struct DataEntry {
    file: String,
    schema: String,
}

#[derive(Deserialize)]
struct Schema {
    tables: Vec<Table>
}

#[derive(Debug)]
pub enum GenerateError {
    IO(std::io::Error),
    Json(serde_json::Error),
    InvalidSchema(String),
    DuplicatedTableName { table_name: String },
    UnknownTableName { table_name: String },
}

impl std::fmt::Display for GenerateError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            GenerateError::IO(e) => {
                write!(f, "{e}")
            }
            GenerateError::Json(e) => {
                write!(f, "{e}")
            },
            GenerateError::InvalidSchema(s) => {
                write!(f, "{s}")
            },
            GenerateError::DuplicatedTableName { table_name } => {
                write!(f, "Duplicated table name {}", table_name)
            },
            GenerateError::UnknownTableName {table_name} => {
                write!(f, "Unknown table name {}", table_name)
            },
        }
    }
}

impl From<std::io::Error> for GenerateError {
    fn from(value: std::io::Error) -> Self {
        GenerateError::IO(value)
    }
}

impl From<serde_json::Error> for GenerateError {
    fn from(value: serde_json::Error) -> Self {
        GenerateError::Json(value)
    }
}

impl std::error::Error for GenerateError {}

pub fn generate_code(data_dir: &PathBuf, out_dir: &PathBuf) -> Result<(), GenerateError> {
    println!("cargo:rerun-if-changed={}", data_dir.join("data.json").display());
    let data_entries: Vec<DataEntry> = serde_json::from_str(
        &fs::read_to_string(&data_dir.join("data.json"))?
    )?;

    // Collect table entries
    let mut table_entries = Vec::new();
    for entry in data_entries {
        let file_path = data_dir.join(entry.file);
        let schema_path = data_dir.join(entry.schema);

        println!("cargo:rerun-if-changed={}", file_path.display());
        println!("cargo:rerun-if-changed={}", schema_path.display());

        let schema_str = fs::read_to_string(&schema_path)?;
        println!("{}", &schema_str);
        let schema: Schema = serde_json::from_str(&schema_str)?;

        if schema.tables.is_empty() {
            return Err(GenerateError::InvalidSchema(
                format!("Table of schema {} is empty", schema_path.display())));
        }

        table_entries.push(TableEntry { file_path, schema });
    }

    // Register types
    let mut table_types = HashMap::new();
    for entry in &table_entries {
        register_table_types(&entry.schema.tables, &mut table_types)?;
    }

    let mut imports = Vec::new();
    let mut exports = Vec::new();

    // Generate table codes & Collect imports/exports
    for entry in &table_entries {
        for table in &entry.schema.tables {
            let mod_name = table.name.to_snake_case();
            let path = out_dir.join(format!("{}.rs", &mod_name));
            println!("{}", path.display());
            let code = generate_table_code(&table, &table_types)?;
            fs::write(path, code)?;

            imports.push(format!("pub mod {mod_name};"));
            exports.push(format!(
                "pub use {}::{{{}, {}}};",
                mod_name,
                table.name,
                format!("{}Data", table.name),
            ));
        }
    }

    // Generate module code
    generate_module_code(&out_dir, &imports, &exports)?;

    Ok(())
}

fn register_table_types(
    tables: &Vec<Table>,
    table_types: &mut HashMap<String, TableKind>,
) -> Result<(), GenerateError> {
    for table in tables {
        if table_types.contains_key(&table.name) {
            return Err(GenerateError::DuplicatedTableName { table_name: table.name.clone() });
        }

        table_types.insert(table.name.clone(), table.kind);
    }

    Ok(())
}

fn generate_module_code(
    out_dir: &PathBuf,
    imports: &Vec<String>,
    exports: &Vec<String>,
) -> Result<(), GenerateError> {
    let imports_code = imports.join("\n");
    let exports_code = exports.join("\n");

    let code = format!(
        r#"// Generated file
{imports_code}

{exports_code}
"#
    );

    fs::write(out_dir.join("data.rs"), code)?;

    Ok(())
}

fn generate_table_code(
    table: &Table,
    table_types: &HashMap<String, TableKind>,
) -> Result<String, GenerateError> {
    match table.kind {
        TableKind::Concrete => generate_concrete_table_code(table, table_types),
        TableKind::Abstract => generate_abstract_table_code(table, table_types),
    }
}

fn generate_concrete_table_code(
    table: &Table,
    table_types: &HashMap<String, TableKind>,
) -> Result<String, GenerateError> {
    let table_name = &table.name;
    let sheet_name = &table.sheet;
    let data_name = format!("{}Data", table_name);
    let data_cell_name = format!("{}_DATA", table_name.to_snake_case().to_uppercase());

    // Check fields
    let mut has_link = false;
    let mut lifetime_code = String::new();
    let mut lifetime_parameter_code = String::new();

    let mut imports = Vec::new();
    let mut field_names = Vec::new();
    let mut field_parses = Vec::new();
    let mut field_definitions = Vec::new();

    for (index, field) in table.fields.iter().enumerate() {
        if let FieldKind::Link { link_type } = &field.kind {
            if !table_types.contains_key(link_type) {
                println!("{:?}", table_types);
                return Err(GenerateError::UnknownTableName { table_name: table_name.clone() });
            }

            has_link = true;
            lifetime_code = "<'a>".to_string();
            lifetime_parameter_code = "<'_>".to_string();

            imports.push(format!("use crate::data::{};", link_type));
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
    let imports_code = imports.join("\n");
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
        r#"// Generated file
use calamine::{{open_workbook, Reader}};
use tracing::info;
use crate::data::*;
{imports_code}

static {data_cell_name}: tokio::sync::OnceCell<{data_name}> = tokio::sync::OnceCell::const_new();

#[derive(Debug, serde::Deserialize)]
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

fn generate_abstract_table_code(
    table: &Table,
    table_types: &HashMap<String, TableKind>,
) -> Result<String, GenerateError> {
    todo!()
}

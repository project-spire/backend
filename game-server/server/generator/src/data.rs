use serde::Deserialize;
use std::fmt::Formatter;
use std::fs;
use std::path::PathBuf;

#[derive(Debug)]
pub struct Config {
    pub gen_dir: PathBuf,
    pub data_dir: PathBuf,
    pub dry_run: bool,
}

impl Config {
    pub fn generate(self) -> Result<(), GenerateError> {
        fs::create_dir_all(&self.gen_dir)?;

        let base_module_path = self.data_dir.join("mod.json");
        collect_modules(&self, &base_module_path)?;


        Ok(())
    }
}

#[derive(Debug)]
pub enum GenerateError {
    IO(std::io::Error),
    Json(serde_json::Error),
    InvalidFile(String),
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
            GenerateError::InvalidFile(s) => {
                write!(f, "Invalid file: {s}")
            }
            GenerateError::InvalidSchema(s) => {
                write!(f, "Invalid schema: {s}")
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
                format!("Link<'a, {}>", t)
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
            FieldKind::Link { link_type: t } => format!("crate::data::parse_link::<{t}>"),
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

struct Module {
    namespaces: Vec<String>,
    entities: Vec<Entity>,
}

#[derive(Debug, Deserialize)]
struct Table {
    pub name: String,
    pub kind: TableKind,
    pub sheet: String,
    pub fields: Vec<Field>,
}

#[derive(Debug, Deserialize)]
enum EntityEntry {
    ModuleEntry {
        #[serde(rename = "mod")] file_path: String
    },
    TableEntry {
        #[serde(rename = "table")] file_path: String,
        #[serde(rename = "schema")] schema_path: String
    },
    ConstEntry {
        #[serde(rename = "const")] file_path: String
    },
}

fn collect_modules(
    config: &Config,
    module_path: &PathBuf,
    mut namespaces: Vec<String>,
) -> Result<(), GenerateError> {
    println!("cargo:rerun-if-changed={}", module_path.display());
    let entity_entries: Vec<EntityEntry> = serde_json::from_str(
        &fs::read_to_string(&module_path)?
    )?;

    for entity_path in &entity_entries {
        let file_name = entity_path.file_name().unwrap().to_str().unwrap();
        let components: Vec<&str> = file_name.split('.').collect();
        if components.len() != 3 || components[2] != "json" {
            return Err(GenerateError::InvalidFile(file_name.to_string()));
        }
        let (namespace, entity_type) = (components[0], components[1]);

        if entity_type == "mod" {
            let mut namespaces = namespaces.clone();
            namespaces.push(namespace.to_owned());
            collect_modules(config, module_path, namespaces)?;
        } else if entity_type == "table" {
            let table_entries: Vec<TableEntry> = serde_json::from_str(
                &fs::read_to_string(entity_path)?
            )?;

        } else if entity_type == "const" {
            todo!()
        } else {
            return Err(GenerateError::InvalidFile(file_name.to_string()));
        }
    }


    Ok(())
}

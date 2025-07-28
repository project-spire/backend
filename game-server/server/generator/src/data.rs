mod generate;

pub use generate::generate_code;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Table {
    pub name: String,
    pub kind: TableKind,
    pub sheet: String,
    pub fields: Vec<Field>,
}

#[derive(Debug, Deserialize, Copy, Clone)]
#[serde(rename_all = "lowercase")]
pub enum TableKind {
    Concrete,
    Abstract
}

#[derive(Debug, Deserialize)]
pub struct Field {
    pub name: String,
    #[serde(flatten)] pub kind: FieldKind,
    #[serde(default)] pub desc: Option<String>,
    #[serde(default)] pub optional: Option<bool>,
    pub cardinality: Option<Cardinality>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum FieldKind {
    Scalar { #[serde(rename = "type")] scalar_type: ScalarType },
    Enum { #[serde(rename = "type")] enum_type: String },
    Link { #[serde(rename = "type")] link_type: String }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ScalarType {
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
pub enum Cardinality {
    Single,
    Multi
}

impl FieldKind {
    pub fn to_rust_type(&self) -> String {
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
    
    pub fn to_parse_code(&self) -> String {
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

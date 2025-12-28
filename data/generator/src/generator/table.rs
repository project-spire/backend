mod abstract_table;
mod concrete_table;

use crate::generator::*;
use crate::*;
use heck::ToSnakeCase;
use std::collections::VecDeque;
use std::fs;
use std::path::Path;

const TUPLE_TYPES_MAX_COUNT: usize = 4;
const fn generate_default() -> bool { true }

#[derive(Debug)]
pub struct TableEntry {
    pub name: Name,
    pub schema: TableSchema,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum TableSchema {
    Concrete(ConcreteTableSchema),
    Abstract(AbstractTableSchema),
}

#[derive(Debug, Deserialize)]
pub struct ConcreteTableSchema {
    #[serde(default = "generate_default")]
    pub enabled: bool,
    pub name: String,
    pub workbook: String,
    pub sheet: String,
    pub fields: Vec<Field>,
    pub extend: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AbstractTableSchema {
    #[serde(default = "generate_default")]
    pub enabled: bool,
    pub name: String,
    pub fields: Vec<Field>,
    pub extend: Option<String>,
}

pub trait TableSchematic {
    fn name(&self) -> &str;
    fn fields(&self) -> &Vec<Field>;
    fn extend(&self) -> &Option<String>;
}

#[derive(Debug, Clone, Deserialize)]
pub struct Field {
    pub name: String,
    pub target: Target,
    #[serde(flatten)]
    pub kind: FieldKind,
    #[serde(default)]
    pub optional: bool,
    #[serde(default)]
    pub multi: bool,
    #[serde(default)]
    pub constraints: Vec<Constraint>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum FieldKind {
    Scalar {
        #[serde(rename = "type")]
        scalar_type: ScalarAllType,
    },
    Enum {
        #[serde(rename = "type")]
        enum_type: String,
    },
    Link {
        #[serde(rename = "type")]
        link_type: String,
    },
    Tuple {
        types: Vec<FieldKind>,
    },
    Union {
        #[serde(rename = "type")]
        union_type: String
    },
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ScalarAllType {
    Id,
    Bool,
    Int8,
    Int16,
    Int32,
    Int64,
    Uint8,
    Uint16,
    Uint32,
    Uint64,
    Float32,
    Float64,
    String,
    Datetime,
    Duration,
    Json,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Constraint {
    Unique,
    Max(i32),
    Min(i32),
}

impl Generator {
    pub fn collect_table(
        &mut self,
        module: &mut ModuleEntry,
        file: &Path,
        name: Name,
    ) -> Result<(), Error> {
        println!("Collecting table `{}`", file.display());

        let type_name = name.as_type(true);
        self.register_type(&type_name)?;

        let schema: TableSchema = serde_json::from_str(&fs::read_to_string(file)?)?;
        if !match &schema {
            TableSchema::Concrete(schema) => schema.enabled,
            TableSchema::Abstract(schema) => schema.enabled,
        } {
            return Ok(());
        }

        self.tables.push(TableEntry { name, schema });
        module
            .entries
            .push(EntityEntry::TableIndex(self.tables.len() - 1));

        self.table_indices.insert(type_name, self.tables.len() - 1);

        Ok(())
    }
    
    pub fn generate_table(
        &self,
        table: &TableEntry,
        writer: &mut dyn Write,
    ) -> Result<(), Error> {
        println!("Generating table `{}`", table.name.name);
        
        match &table.schema {
            TableSchema::Concrete(schema) => {
                self.generate_concrete_table(&table.name, schema, writer)?;
            }
            TableSchema::Abstract(schema) => {
                self.generate_abstract_table(&table.name, schema, writer)?;
            }
        };

        Ok(())
    }

    /// Get all fields of table including its parents
    pub fn get_table_all_fields(&self, schema: &dyn TableSchematic) -> Result<Vec<Field>, Error> {
        let mut fields = VecDeque::from(schema.fields().clone());
        let mut extend = schema.extend().clone();

        while let Some(parent) = extend.take() {
            let parent_index = self
                .table_indices
                .get(&parent)
                .ok_or_else(|| Error::Inheritance(schema.name().to_owned(), parent))?;
            let parent_table = &self.tables[*parent_index];
            let parent_schema = parent_table.schema.schematic();

            for field in parent_schema.fields().iter().rev() {
                fields.push_front(field.clone());
            }

            extend = parent_schema.extend().clone();
        }

        Ok(Vec::from(fields))
    }
}

impl TableSchema {
    pub fn schematic(&self) -> &dyn TableSchematic {
        match self {
            TableSchema::Concrete(c) => c,
            TableSchema::Abstract(a) => a,
        }
    }
}

impl FieldKind {
    fn to_rust_type(&self) -> String {
        match &self {
            FieldKind::Scalar { scalar_type: t } => match t {
                ScalarAllType::Id => "DataId",
                ScalarAllType::Bool => "bool",
                ScalarAllType::Int8 => "i8",
                ScalarAllType::Int16 => "i16",
                ScalarAllType::Int32 => "i32",
                ScalarAllType::Int64 => "i64",
                ScalarAllType::Uint8 => "u8",
                ScalarAllType::Uint16 => "u16",
                ScalarAllType::Uint32 => "u32",
                ScalarAllType::Uint64 => "u64",
                ScalarAllType::Float32 => "f32",
                ScalarAllType::Float64 => "f64",
                ScalarAllType::String => "String",
                ScalarAllType::Datetime => "chrono::DateTime",
                ScalarAllType::Duration => "chrono::Duration",
                ScalarAllType::Json => "serde_json::Value",
            }.to_string(),
            FieldKind::Enum { enum_type } => {
                format!("crate::{enum_type}")
            }
            FieldKind::Link { link_type } => {
                format!("Link<crate::{link_type}>")
            }
            FieldKind::Tuple { types } => {
                let type_strings = to_tuple_type_strings(types);
                format!("({})", type_strings.join(", "))
            }
            FieldKind::Union { union_type } => {
                format!("crate::{union_type}")
            }
        }
    }

    fn has_link(&self) -> bool {
        match &self {
            FieldKind::Scalar { .. } => false,
            FieldKind::Enum { .. } => false,
            FieldKind::Link { .. } => true,
            FieldKind::Tuple { types } => {
                let mut has_link = false;
                for t in types {
                    match t {
                        FieldKind::Scalar { .. } => continue,
                        FieldKind::Enum { .. } => continue,
                        FieldKind::Link { .. } => {
                            has_link = true;
                            break;
                        }
                        FieldKind::Tuple { .. } => panic!("Nested tuples are not supported"),
                        FieldKind::Union { .. } => continue,
                    }
                }

                has_link
            },
            FieldKind::Union { .. } => false,
        }
    }
}

fn to_tuple_type_strings(fields: &[FieldKind]) -> Vec<String> {
    let mut type_strings = Vec::new();
    for t in fields {
        if let FieldKind::Tuple { .. } = t {
            panic!("Nested tuples are not supported");
        }

        type_strings.push(t.to_rust_type())
    }

    if type_strings.len() < 2 {
        panic!("Tuples must have at least two fields");
    }
    if type_strings.len() > TUPLE_TYPES_MAX_COUNT {
        panic!(
            "Tuples with more than {} types are not supported",
            TUPLE_TYPES_MAX_COUNT
        );
    }

    type_strings
}

impl Name {
    pub fn as_table_type(&self, with_namespace: bool) -> String {
        format!("{}Table", self.as_type(with_namespace))
    }

    pub fn as_table_type_cell(&self) -> String {
        format!("{}_TABLE", self.name.to_snake_case().to_uppercase())
    }
}

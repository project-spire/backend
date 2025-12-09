mod abstract_table;
mod concrete_table;

use crate::generator::*;
use crate::*;
use std::collections::VecDeque;
use std::fs;

const TUPLE_TYPES_MAX_COUNT: usize = 4;

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
    pub name: String,
    pub workbook: String,
    pub sheet: String,
    pub fields: Vec<Field>,
    pub extend: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AbstractTableSchema {
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
    pub optional: Option<bool>,
    #[serde(default)]
    pub multi: Option<bool>,
    #[serde(default)]
    pub constraints: Option<Vec<Constraint>>,
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
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Constraint {
    Unique,
    Max(i32),
    Min(i32),
}

impl Generator {
    pub fn generate_table(&self, table: &TableEntry) -> Result<(), Error> {
        let table_file = self
            .full_gen_dir(&table.name.parent_namespace())
            .join(format!("{}.rs", table.name.as_entity()));
        self.log(&format!("Generating table `{}`", table_file.display()));

        let code = match &table.schema {
            TableSchema::Concrete(schema) => self.generate_concrete_table(&table.name, schema)?,
            TableSchema::Abstract(schema) => self.generate_abstract_table(&table.name, schema)?,
        };

        fs::write(table_file, code)?;

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
            }
            .to_string(),
            FieldKind::Enum { enum_type } => {
                format!("{CRATE_PREFIX}::{enum_type}")
            }
            FieldKind::Link { link_type } => {
                format!("Link<{CRATE_PREFIX}::{link_type}>")
            }
            FieldKind::Tuple { types } => {
                let type_strings = to_tuple_type_strings(types);
                format!("({})", type_strings.join(", "))
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
                    }
                }

                has_link
            }
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

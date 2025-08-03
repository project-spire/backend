mod collect;
mod generate;

use heck::{ToUpperCamelCase, ToSnakeCase};
use std::collections::{HashMap, HashSet};
use std::fmt::Formatter;
use std::path::PathBuf;
use serde::Deserialize;

const HEADER_ROWS: usize = 1;

#[derive(Debug)]
pub struct Config {
    pub base_module_path: PathBuf,
    pub data_dir: PathBuf,
    pub gen_dir: PathBuf,
    pub dry_run: bool,
}

impl Config {
    pub fn generate(self) -> Result<(), GenerateError> {
        let mut generator = Generator::new(self);
        generator.collect()?;
        generator.generate()?;

        Ok(())
    }
}

#[derive(Debug)]
pub struct Generator {
    config: Config,
    
    base_module_index: Option<usize>,
    modules: Vec<ModuleEntry>,
    tables: Vec<TableEntry>,
    enumerations: Vec<EnumerationEntry>,
    constants: Vec<ConstantEntry>,

    type_names: HashSet<String>,
    table_indices: HashMap<String, usize>,
    dependency_levels: Vec<Vec<usize>>,
}

impl Generator {
    pub fn new(config: Config) -> Self {
        Self {
            config,

            base_module_index: None,
            modules: Vec::new(),
            tables: Vec::new(),
            enumerations: Vec::new(),
            constants: Vec::new(),

            type_names: HashSet::new(),
            table_indices: HashMap::new(),
            dependency_levels: Vec::new(),
        }
    }

    pub fn full_data_path(&self, namespaces: &[String], file_name: &str) -> PathBuf {
        self.config.data_dir.join(namespaces.join(".")).join(file_name)
    }

    pub fn full_gen_dir(&self, namespaces: &[String]) -> PathBuf {
        self.config.gen_dir.join(namespaces.join("/"))
    }

    pub fn full_gen_path(&self, namespaces: &[String], file_name: &str) -> PathBuf {
        self.full_gen_dir(namespaces).join(file_name)
    }
}

#[derive(Debug)]
pub enum GenerateError {
    IO(std::io::Error),
    Json(serde_json::Error),
    InvalidFileName(String),
    NamespaceCollision(String),
    UnknownType(String),
    CircularDependency,
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
            GenerateError::InvalidFileName(s) => {
                write!(f, "Invalid file name: {s}")
            },
            GenerateError::NamespaceCollision(s) => {
                write!(f, "Namespace collision: {s}")
            },
            GenerateError::UnknownType(s) => {
                write!(f, "Unknown type: {s}")
            },
            GenerateError::CircularDependency => {
                write!(f, "Circular dependency! TODO: Show subgraph.")
            }
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

#[derive(Debug)]
pub struct ModuleEntry {
    name: Name,
    entries: Vec<EntityEntry>,
}

impl ModuleEntry {
    pub fn new(name: Name) -> Self {
        Self {
            name,
            entries: Vec::new(),
        }
    }
}

#[derive(Debug)]
pub struct TableEntry {
    name: Name,
    table_path: PathBuf,
    schema: TableSchema,
}

#[derive(Debug)]
pub struct EnumerationEntry {
    name: Name,
    schema: EnumerationSchema
}
#[derive(Debug)]
pub struct ConstantEntry {
    name: Name,
    schema: ConstantSchema,
}

#[derive(Debug)]
pub enum EntityEntry {
    ModuleIndex(usize),
    TableIndex(usize),
    EnumerationIndex(usize),
    ConstantIndex(usize),
}

#[derive(Debug, Deserialize)]
pub struct TableSchema {
    pub name: String,
    pub kind: TableKind,
    pub sheet: String,
    pub fields: Vec<Field>,
}

#[derive(Debug, Deserialize)]
pub struct ConstantSchema {
    pub name: String,
    pub target: Target,
    #[serde(flatten)] pub scalar: ConstantScalar,
}

#[derive(Debug, Deserialize)]
pub enum ConstantScalar {
    SignedInteger {
        scalar_type: ScalarSignedIntegerType,
        value: i64,
    },
    UnsignedInteger {
        scalar_type: ScalarUnsignedIntegerType,
        value: u64,
    },
    Float {
        scalar_type: ScalarFloatType,
        value: f64,
    },
    String {
        scalar_type: ScalarStringType,
        value: String,
    },
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ScalarSignedIntegerType {
    Int8,
    Int16,
    Int32,
    Int64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ScalarUnsignedIntegerType {
    Uint8,
    Uint16,
    Uint32,
    Uint64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ScalarFloatType {
    Float32,
    Float64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ScalarStringType {
    String,
}

#[derive(Debug, Deserialize)]
pub struct EnumerationSchema {
    name: String,
    base: EnumerationBase,
    enums: Vec<String>,
    target: Target,
    attributes: Vec<EnumerationAttribute>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EnumerationBase {
    Uint8,
    Uint16,
    Uint32,
}

#[derive(Debug, Deserialize)]
pub struct EnumerationAttribute {
    target: Target,
    attribute: String,
}

#[derive(Debug, Clone)]
pub struct Name {
    pub name: String,
    pub namespaces: Vec<String>,
}

impl Name {
    pub fn new(name: &str, namespaces: &[String]) -> Self {
        Self {
            name: name.to_owned(),
            namespaces: namespaces.to_vec(),
        }
    }

    pub fn as_entity(&self) -> String {
        self.name.to_snake_case()
    }

    pub fn as_type(&self, with_namespace: bool) -> String {
        if with_namespace && !self.namespaces.is_empty() {
            format!(
                "{}::{}",
                self.namespaces.join("::"),
                self.name.to_upper_camel_case(),
            )
        } else {
            self.name.to_upper_camel_case()
        }
    }

    pub fn as_data_type(&self) -> String {
        format!("{}Data", self.as_type(false))
    }

    pub fn as_data_type_cell(&self) -> String {
        format!("{}_DATA", self.name.to_snake_case().to_uppercase())
    }
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
    #[serde(default)] pub cardinality: Option<Cardinality>,
    #[serde(default)] pub constraints: Option<Vec<Constraint>>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum FieldKind {
    Scalar { #[serde(rename = "type")] scalar_type: ScalarAllType },
    Enum { #[serde(rename = "type")] enum_type: String },
    Link { #[serde(rename = "type")] link_type: String }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ScalarAllType {
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

#[derive(Debug, Deserialize)]
pub enum Constraint {
    #[serde(rename = "exclusive")] Exclusive
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Target {
    Client,
    Server,
    All,
}

impl Target {
    fn is_target(&self) -> bool {
        self == &Target::Server || self == &Target::All
    }
}

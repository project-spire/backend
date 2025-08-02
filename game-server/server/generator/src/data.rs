mod collect;
mod generate;
mod validate;

use std::collections::{HashMap, VecDeque};
use std::fmt::Formatter;
use std::path::PathBuf;

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
        generator.validate()?;
        generator.generate()?;

        Ok(())
    }
}

#[derive(Debug)]
pub enum GenerateError {
    IO(std::io::Error),
    Json(serde_json::Error),
    InvalidFile(String),
    InvalidSchema(String),
    NamespaceCollision { name: String },
    UnknownTableName { table_name: String },
    DependencyCycle,
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
            GenerateError::NamespaceCollision { name } => {
                write!(f, "Namespace collision: {}", name)
            },
            GenerateError::UnknownTableName {table_name} => {
                write!(f, "Unknown table name: {}", table_name)
            },
            GenerateError::DependencyCycle => {
                write!(f, "Cycle detected in table table dependency!")
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
pub struct ModuleDef {
    pub name: String,
    pub namespaces: Vec<String>,
    pub entities: Vec<Entity>,
}

#[derive(Debug)]
pub struct TableDef {
    pub namespaces: Vec<String>,
    pub name: String,
    pub file_path: PathBuf,
    pub schema_path: PathBuf,
}

#[derive(Debug)]
pub struct ConstDef {
    pub namespaces: Vec<String>,
    pub name: String,
    pub file_path: PathBuf,
}

#[derive(Debug)]
pub struct EnumDef {
    pub namespaces: Vec<String>,
    pub name: String,
    pub file_path: PathBuf,
}

#[derive(Debug)]
pub enum Entity {
    Module(String),
    Table(String),
    Const(String),
    Enum(String),
}

#[derive(Debug)]
pub struct Generator {
    config: Config,
    modules: Vec<ModuleDef>,
    tables: HashMap<String, TableDef>,
    constants: HashMap<String, ConstDef>,
    enums: HashMap<String, EnumDef>,
}

impl Generator {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            modules: Vec::new(),
            tables: HashMap::new(),
            constants: HashMap::new(),
            enums: HashMap::new(),
        }
    }

    fn full_data_path(&self, namespaces: &VecDeque<String>, path: &str) -> PathBuf {
        self.config.data_dir
            .join(namespaces.iter().cloned().collect::<Vec<_>>().join("."))
            .join(path)
    }
    
    fn full_gen_dir(&self, namespaces: &[String]) -> PathBuf {
        self.config.gen_dir.join(namespaces.join("/"))
    }
}

pub(crate) mod constant;
pub(crate) mod enumeration;
pub(crate) mod module;
pub(crate) mod table;

use crate::*;
use self::constant::ConstantEntry;
use self::enumeration::EnumerationEntry;
use self::module::ModuleEntry;
use self::table::TableEntry;
use std::collections::{HashMap, HashSet};
use std::fs;

#[derive(Debug)]
pub struct Generator {
    pub config: Config,

    pub modules: Vec<ModuleEntry>,
    pub tables: Vec<TableEntry>,
    pub enumerations: Vec<EnumerationEntry>,
    pub constants: Vec<ConstantEntry>,

    pub names: HashSet<String>,
    pub table_indices: HashMap<String, usize>,
    pub table_hierarchies: HashMap<usize, Vec<usize>>,
}

impl Generator {
    pub fn new(config: Config) -> Self {
        Self {
            config,

            modules: Vec::new(),
            tables: Vec::new(),
            enumerations: Vec::new(),
            constants: Vec::new(),

            names: HashSet::new(),
            table_indices: HashMap::new(),
            table_hierarchies: HashMap::new(),
        }
    }

    pub fn generate(&self) -> Result<(), Error> {
        println!("Generating...");

        fs::create_dir_all(&self.config.gen_dir)?;

        for module in &self.modules {
            self.generate_module(module)?
        }

        for table in &self.tables {
            self.generate_table(&table)?
        }

        for enumeration in &self.enumerations {
            self.generate_enumeration(&enumeration)?
        }

        for constant in &self.constants {
            self.generate_constant(&constant)?
        }

        Ok(())
    }
    
    pub fn log(&self, message: &str) {
        if !self.config.verbose {
            return;
        }
        
        println!("{}", message);
    }

    fn full_gen_dir(&self, namespaces: &[String]) -> PathBuf {
        self.config.gen_dir.join(namespaces.join("/"))
    }
    
    fn get_parent_table(&self, extend: &Option<String>) -> Option<&TableEntry> {
        let parent = match extend {
            Some(parent) => parent,
            None => return None,
        };
        
        Some(&self.tables[self.table_indices[parent]])
    }
}

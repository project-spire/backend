mod constant;
mod enumeration;
mod module;
mod table;

use std::fs;
use crate::data::*;

const TAB: &str = "    ";
const CRATE_PREFIX: &str = "crate::data";
const GENERATED_FILE_WARNING: &str = r#"// This is a generated file. DO NOT MODIFY."#;


impl Generator {
    pub fn generate(&self) -> Result<(), GenerateError> {
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
}

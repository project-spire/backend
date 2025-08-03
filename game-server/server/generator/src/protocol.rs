use glob::glob;
use std::path::PathBuf;

mod category;

#[derive(Debug)]
pub struct Config {
    pub schema_dir: PathBuf,
    
    pub category_path: PathBuf,
    pub category_gen_path: PathBuf,
    
    pub dry_run: bool,
}

impl Config {
    pub fn generate(self) -> Result<(), Box<dyn std::error::Error>> {
        let schemas: Vec<PathBuf> = glob(&format!("{}/**/*.proto", self.schema_dir.display()))?
            .filter_map(Result::ok)
            .collect();

        for schema in &schemas {
            println!("cargo:rerun-if-changed={}", schema.display());
        }

        prost_build::compile_protos(&schemas, &[self.schema_dir.to_str().unwrap()])?;

        Ok(())
    }
}

use std::env;
use std::error::Error;
use std::path::PathBuf;

use glob::glob;

fn main() -> Result<(), Box<dyn Error>> {
    compile()?;

    let config = protocol_generator::Config {
        schema_dir: PathBuf::from("inner/schema"),
        gen_dir: PathBuf::from(env::var("OUT_DIR")?),
        docs_dir: Some(PathBuf::from("inner/docs")),
        generate_impl: true,
        generate_handle: false,
    };
    config.generate()?;

    Ok(())
}

pub fn compile() -> Result<(), Box<dyn Error>> {
    fn find(schema_base_dir: &PathBuf, schema_dir: &PathBuf) -> Vec<PathBuf> {
        let schemas: Vec<PathBuf> = [
            schema_base_dir.join("*.proto").to_str().unwrap(),
            schema_dir.join("**/*.proto").to_str().unwrap(),
        ]
            .iter()
            .flat_map(|pattern| glob(pattern).unwrap())
            .filter_map(Result::ok)
            .collect();

        schemas
    }

    let schema_base_dir = PathBuf::from("inner/schema");

    // Game protocols
    let schema_dir = schema_base_dir.join("game");
    let schemas = find(&schema_base_dir, &schema_dir);

    for schema in &schemas {
        println!("cargo:rerun-if-changed={}", schema.display());
    }

    prost_build::compile_protos(&schemas, &[&schema_base_dir, &schema_dir])?;

    // Lobby protocols
    let schema_dir = schema_base_dir.join("lobby");
    let schemas = find(&schema_base_dir, &schema_dir);

    for schema in &schemas {
        println!("cargo:rerun-if-changed={}", schema.display());
    }
    tonic_prost_build::configure().compile_protos(&schemas, &[schema_base_dir.clone(), schema_dir])?;

    Ok(())
}

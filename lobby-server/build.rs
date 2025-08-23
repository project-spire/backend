use std::path::PathBuf;
use glob::glob;

const SCHEMA_DIR: &str = "lobby-protocol/schema";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let schemas: Vec<PathBuf> = glob(&format!("{SCHEMA_DIR}/**/*.proto"))?
        .filter_map(Result::ok)
        .collect();

    for schema in &schemas {
        println!("cargo:rerun-if-changed={}", schema.display());
    }

    tonic_prost_build::configure().compile_protos(
        &schemas,
        &[SCHEMA_DIR.into()],
    )?;

    Ok(())
}
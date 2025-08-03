use std::{env, fs};
use std::path::PathBuf;
use generator::{data, protocol};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // println!("cargo:rerun-if-changed=build.rs");

    // Copy Settings file
    let out_dir = PathBuf::from(env::var("OUT_DIR")?);
    println!("cargo:rerun-if-changed=settings.ron");
    fs::copy("../settings.ron", out_dir.join("../settings.ron"))?;

    // Generate data codes
    let config = data::Config {
        base_module_path: PathBuf::from("../game-data/data.mod.json"),
        data_dir: PathBuf::from("../game-data/data"),
        // gen_dir: out_dir.join("gen"),
        gen_dir: PathBuf::from("./src/data"),
        dry_run: false,
    };
    config.generate()?;
    
    // Generate protocol codes
    let config = protocol::Config {
        schema_dir: PathBuf::from("../game-protocol/schema"),
        
        category_path: PathBuf::from("../game-protocol/category.json"),
        category_gen_path: PathBuf::from("./src/protocol/category.rs"),
        
        dry_run: false,
    };
    config.generate()?;

    Ok(())
}





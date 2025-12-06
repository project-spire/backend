use std::path::PathBuf;
use std::{env, fs};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Copy Environment file.
    let out_dir = PathBuf::from(env::var("OUT_DIR")?);
    println!("cargo:rerun-if-changed=environment.ron");
    fs::copy("environment.ron", out_dir.join("environment.ron"))?;
    
    // Generate protocol handling.
    let config = protocol_generator::Config {
        schema_dir: PathBuf::from("../protocol/inner/schema"),
        gen_dir: PathBuf::from(env::var("OUT_DIR")?),
        generate_impl: false,
        generate_handle: true,
    };
    config.generate()?;

    Ok(())
}

use std::fs;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = std::env::var("OUT_DIR")?;

    // Copy `config.json`
    println!("cargo:rerun-if-changed=config.json");
    fs::copy("../config.json", Path::new(&out_dir).join("config.json"))?;

    Ok(())
}

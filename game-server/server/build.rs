use std::fs;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = std::env::var("OUT_DIR")?;

    // Copy `config.ron`
    println!("cargo:rerun-if-changed=config.ron");
    fs::copy("../config.ron", Path::new(&out_dir).join("config.ron"))?;

    Ok(())
}

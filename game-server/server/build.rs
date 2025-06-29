use std::fs;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = std::env::var("OUT_DIR")?;

    // Copy Settings file
    println!("cargo:rerun-if-changed=settings.ron");
    fs::copy("../settings.ron", Path::new(&out_dir).join("../settings.ron"))?;

    Ok(())
}

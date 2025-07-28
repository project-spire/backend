use std::{env, fs};
use std::path::PathBuf;
use generator::data;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // println!("cargo:rerun-if-changed=build.rs");

    let out_dir = PathBuf::from(env::var("OUT_DIR")?).join("gen");
    fs::create_dir_all(&out_dir)?;

    let data_dir = PathBuf::from("../game-data");

    // Copy Settings file
    // println!("cargo:rerun-if-changed=settings.ron");
    // fs::copy("../settings.ron", out_dir.join("../settings.ron"))?;

    data::generate_code(&data_dir, &out_dir)?;

    Ok(())
}





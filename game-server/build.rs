use std::env;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
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

use std::path::PathBuf;
use std::process::exit;

fn main() {
    let config = data_generator::Config {
        schema_dir: PathBuf::from("inner/schema"),
        src_dir: PathBuf::from("inner/src"),
        gen_dir: PathBuf::from("src/data"),
        protocol_gen_dir: PathBuf::from("../protocol/inner/schema"),
        sql_gen_dir: PathBuf::from("../db/schema/types"),

        verbose: true,
        dry_run: false,
    };

    if let Err(e) = config.generate() {
        eprintln!("Failed to generate: {}", e);
        exit(1);
    }
}
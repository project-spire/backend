use std::env;
use std::path::PathBuf;
use std::process::exit;
use data_generator::Target;

fn main() {
    let config = data_generator::Config {
        schema_dir: PathBuf::from("inner/schema"),
        src_gen_dir: PathBuf::from(env::var("OUT_DIR").unwrap()),
        protobuf_gen_dir: PathBuf::from("../protocol/inner/schema"),
        sql_gen_dir: PathBuf::from("../db/schema/types"),

        target: Target::Server,
        header_rows: 2,
        dry_run: false,
    };

    if let Err(e) = config.generate() {
        eprintln!("Failed to generate: {}", e);
        exit(1);
    }
}
mod collect;
mod error;
mod generator;
mod name;

use crate::error::Error;
use crate::generator::Generator;
use crate::name::Name;
use serde::Deserialize;
use std::path::PathBuf;

const HEADER_ROWS: usize = 2;
const TAB: &str = "    ";
const CRATE_PREFIX: &str = "crate";
const GENERATED_FILE_WARNING: &str = r#"// This is a generated file. DO NOT MODIFY."#;

#[derive(Debug)]
pub struct Config {
    pub schema_dir: PathBuf,
    pub src_dir: PathBuf,
    pub gen_dir: PathBuf,
    pub protocol_gen_dir: PathBuf,
    pub sql_gen_dir: PathBuf,

    pub verbose: bool,
    pub dry_run: bool,
}

impl Config {
    pub fn generate(self) -> Result<(), Error> {
        let mut generator = Generator::new(self);
        generator.collect()?;
        generator.generate()?;

        Ok(())
    }
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
enum Target {
    Client,
    Server,
    All,
    None,
}

impl Target {
    fn is_target(&self) -> bool {
        self == &Target::Server || self == &Target::All
    }
}

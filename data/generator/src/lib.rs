mod error;
mod generator;
mod name;
mod schema;

use crate::error::Error;
use crate::generator::Generator;
use crate::name::Name;
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Deserialize, Clone, Copy, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Target {
    Client,
    Server,
    All,
    None,
}

#[derive(Debug)]
pub struct Config {
    pub schema_dir: PathBuf,
    // pub src_dir: PathBuf,
    pub src_gen_dir: PathBuf,
    pub protobuf_gen_dir: PathBuf,
    pub sql_gen_dir: PathBuf,

    pub target: Target,
    pub header_rows: usize,
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

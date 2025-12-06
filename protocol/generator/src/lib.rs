mod error;
mod generator;

use std::path::PathBuf;

use crate::error::Error;
use crate::generator::Generator;

#[derive(Debug)]
pub struct Config {
    pub schema_dir: PathBuf,
    pub gen_dir: PathBuf,
    pub generate_impl: bool,
    pub generate_handle: bool,
}

impl Config {
    pub fn generate(self) -> Result<(), Error> {
        let mut generator = Generator::new(self);

        generator.collect()?;
        generator.generate()?;

        Ok(())
    }
}

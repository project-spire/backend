#![allow(static_mut_refs)]

use crate::config::config;
use serde::Deserialize;
use std::mem::MaybeUninit;
use std::path::PathBuf;

static mut ENV: MaybeUninit<Env> = MaybeUninit::uninit();

#[derive(Debug, Deserialize)]
pub struct Env {
    pub cheat_enabled: bool,
    pub data_dir: PathBuf,
}

impl Env {
    pub fn init() -> Result<(), Box<dyn std::error::Error>> {
        let settings = config::Config::builder()
            .add_source(
                config::File::with_name(config().env_file.as_os_str().to_str().unwrap())
                    .required(true),
            )
            .build()?
            .try_deserialize()?;

        unsafe {
            ENV.write(settings);
        }
        Ok(())
    }

    pub fn get() -> &'static Self {
        unsafe { ENV.assume_init_ref() }
    }
}

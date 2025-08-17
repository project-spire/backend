use serde::Deserialize;
use std::path::PathBuf;
use std::sync::OnceLock;

static ENV: OnceLock<Env> = OnceLock::new();

#[derive(Debug, Deserialize)]
pub struct Env {
    pub cheat_enabled: bool,
    pub data_dir: PathBuf,
}

impl Env {
    pub fn init() -> Result<(), Box<dyn std::error::Error>> {
        let settings = config::Config::builder()
            .add_source(config::File::with_name("environment.ron").required(true))
            .build()?
            .try_deserialize()?;

        ENV.set(settings).expect("Environment already initialized");
        Ok(())
    }

    pub fn get() -> &'static Self {
        ENV.get().expect("Environment not initialized yet")
    }
}

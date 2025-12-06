use std::path::PathBuf;
use std::sync::OnceLock;
use std::time::Duration;
use serde::Deserialize;

use crate::config::config;

static ENV: OnceLock<Env> = OnceLock::new();

#[derive(Debug, Deserialize)]
pub struct Env {
    pub data_dir: PathBuf,

    // Network
    login_timeout_sec: u8,
    #[serde(skip_deserializing)]
    pub login_timeout: Duration,
    
    pub ingress_protocols_rate_limit: Option<common::rate_limiter::Params>,
    pub ingress_bytes_rate_limit: Option<common::rate_limiter::Params>,
    
    // Game
    pub cheat_enabled: bool,
}

impl Env {
    pub fn init() -> Result<(), Box<dyn std::error::Error>> {
        let mut env: Env = config::Config::builder()
            .add_source(
                config::File::with_name(config().env_file.as_os_str().to_str().unwrap())
                    .required(true),
            )
            .build()?
            .try_deserialize()?;
        
        env.login_timeout = Duration::from_secs(env.login_timeout_sec as u64);

        ENV.set(env)
            .map_err(|_| "Attempted to initialize Environment more than once")?;
        Ok(())
    }
}

pub fn env() -> &'static Env {
    ENV.get().expect("Environment is not initialized yet")
}

use serde::Deserialize;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::sync::OnceLock;
use config::Source;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub cheat_enabled: bool,
}

impl Config {
    pub fn new() -> Result<Self, config::ConfigError> {
        let config = config::Config::builder()
            .add_source(config::File::with_name("config.ron").required(true))
            .build()?;

        config.try_deserialize()
    }
}

pub static CONFIG: OnceLock<Config> = OnceLock::new();

#[derive(Debug, Deserialize)]
pub struct NetworkConfig {
    pub game_listen_port: u16,
    pub control_listen_port: u16,

    pub db_host: String,
    pub db_port: u16,
    pub db_user: String,
    pub db_name: String,
    
    #[serde(skip_deserializing)]
    pub db_password: String,
    db_password_file: String,

    #[serde(skip_deserializing)]
    pub auth_key: Vec<u8>,
    auth_key_file: String,
}

impl NetworkConfig {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let mut config: NetworkConfig = config::Config::builder()
            .add_source(config::Environment::with_prefix("SPIRE"))
            .build()?
            .try_deserialize()?;

        config.db_password = read_from_file(Path::new(&config.db_password_file))?;
        config.auth_key = read_from_file(Path::new(&config.auth_key_file))?.into_bytes();

        Ok(config)
    }
}

fn read_from_file(path: &Path) -> Result<String, std::io::Error> {
    let mut f = File::open(path)?;
    let mut buf = String::new();
    f.read_to_string(&mut buf)?;

    Ok(buf.trim().to_string())
}

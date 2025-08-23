use serde::Deserialize;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

static CONFIG: OnceLock<Config> = OnceLock::new();

#[derive(Debug, Deserialize)]
pub struct Config {
    #[serde(alias = "lobby_server_port")]
    pub lobby_port: u16,

    pub db_host: String,
    pub db_port: u16,
    pub db_user: String,
    pub db_name: String,
    #[serde(skip_deserializing)]
    pub db_password: String,
    db_password_file: PathBuf,

    #[serde(skip_deserializing)]
    pub token_key: Vec<u8>,
    token_key_file: PathBuf,

    #[serde(alias = "lobby_server_tls_cert_file")]
    pub tls_cert_file: PathBuf,
    #[serde(alias = "lobby_server_tls_key_file")]
    pub tls_key_file: PathBuf,
}

impl Config {
    pub fn init() -> Result<(), Box<dyn std::error::Error>> {
        let config = config::Config::builder()
            .add_source(config::Environment::with_prefix("SPIRE"))
            .build()?;

        let mut config: Config = config.try_deserialize()?;

        config.db_password = read_from_file(&config.db_password_file)?;
        config.token_key = read_from_file(&config.token_key_file)?.into_bytes();

        CONFIG.set(config).expect("Config already initialized");
        Ok(())
    }

    pub fn get() -> &'static Self {
        CONFIG.get().expect("Config not initialized yet!")
    }
}

fn read_from_file(path: &Path) -> Result<String, std::io::Error> {
    let mut f = File::open(path)?;
    let mut buf = String::new();
    f.read_to_string(&mut buf)?;

    Ok(buf.trim().to_string())
}

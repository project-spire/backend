use serde::Deserialize;
use std::path::PathBuf;
use std::sync::OnceLock;

static CONFIG: OnceLock<Config> = OnceLock::new();

#[derive(Debug, Deserialize)]
pub struct Config {
    #[serde(alias = "lobby_server_port")]
    pub port: u16,
    #[serde(alias = "lobby_server_tls_cert_file")]
    pub tls_cert_file: PathBuf,
    #[serde(alias = "lobby_server_tls_key_file")]
    pub tls_key_file: PathBuf,

    #[serde(skip_deserializing)]
    pub token_key: Vec<u8>,
    token_key_file: PathBuf,
    pub token_expiration_seconds: u64,

    pub db_host: String,
    pub db_port: u16,
    pub db_user: String,
    pub db_name: String,
    #[serde(skip_deserializing)]
    pub db_password: String,
    db_password_file: PathBuf,

    #[serde(alias = "lobby_server_dev_mode", default)]
    pub dev_mode: bool,

    #[serde(alias = "lobby_server_node_id")]
    pub node_id: u16,
}

impl Config {
    pub fn init() -> Result<(), Box<dyn std::error::Error>> {
        let config = config::Config::builder()
            .add_source(config::Environment::with_prefix("SPIRE"))
            .build()?;

        let mut config: Config = config.try_deserialize()?;

        config.db_password = common::io::read_file(&config.db_password_file)?;
        config.token_key = common::io::read_file(&config.token_key_file)?.into_bytes();

        CONFIG.set(config).expect("Config already initialized");
        Ok(())
    }
}

pub fn config() -> &'static Config {
    CONFIG.get().expect("Config not initialized yet!")
}

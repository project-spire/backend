use serde::Deserialize;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::sync::OnceLock;

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub cheat_enabled: bool,
}

impl Settings {
    pub fn new() -> Result<Self, config::ConfigError> {
        let settings = config::Config::builder()
            .add_source(config::File::with_name("settings.ron").required(true))
            .build()?;

        settings.try_deserialize()
    }
}

pub static SETTINGS: OnceLock<Settings> = OnceLock::new();

#[derive(Debug, Deserialize)]
pub struct NetworkSettings {
    #[serde(skip_deserializing)] pub port: u16,
    game_server_port: u16,

    #[serde(skip_deserializing)] pub control_port: u16,
    game_server_control_port: u16,

    pub db_host: String,
    pub db_port: u16,
    pub db_user: String,
    pub db_name: String,

    #[serde(skip_deserializing)] pub db_password: String,
    db_password_file: String,

    #[serde(skip_deserializing)] pub token_key: Vec<u8>,
    token_key_file: String,
}

impl NetworkSettings {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let mut settings: NetworkSettings = config::Config::builder()
            .add_source(config::Environment::with_prefix("SPIRE"))
            .build()?
            .try_deserialize()?;

        settings.port = settings.game_server_port;
        settings.control_port = settings.game_server_control_port;
        settings.db_password = read_from_file(Path::new(&settings.db_password_file))?;
        settings.token_key = read_from_file(Path::new(&settings.token_key_file))?.into_bytes();

        Ok(settings)
    }
}

fn read_from_file(path: &Path) -> Result<String, std::io::Error> {
    let mut f = File::open(path)?;
    let mut buf = String::new();
    f.read_to_string(&mut buf)?;

    Ok(buf.trim().to_string())
}

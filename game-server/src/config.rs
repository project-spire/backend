use quinn::rustls::pki_types::{CertificateDer, PrivateKeyDer};
use serde::Deserialize;
use std::path::PathBuf;
use std::sync::OnceLock;
use std::time::Duration;

pub static CONFIG: OnceLock<Config> = OnceLock::new();

#[derive(Debug)]
pub struct Config {
    pub net: NetworkConfig,
    pub db: DatabaseConfig,
    pub app: ApplicationConfig,
}

#[derive(Debug, Deserialize)]
pub struct NetworkConfig {
    #[serde(alias = "game_server_node_id")]
    pub node_id: u16,
    #[serde(alias = "game_server_port")]
    pub port: u16,
    #[serde(alias = "game_server_control_port")]
    pub control_port: u16,
    pub application_protocol: String,

    #[serde(alias = "game_server_tls_cert_file")]
    tls_cert_file: PathBuf,
    #[serde(alias = "game_server_tls_key_file")]
    tls_key_file: PathBuf,

    #[serde(skip_deserializing)]
    pub token_key: Vec<u8>,
    token_key_file: PathBuf,
}

#[derive(Debug, Deserialize)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub name: String,
    #[serde(skip_deserializing)]
    pub password: String,
    password_file: PathBuf,
}
#[derive(Debug, Deserialize)]
pub struct ApplicationConfig {
    pub data: app::Data,
    pub cheat: app::Cheat,
    pub login: app::Login,
    pub ingress: app::Ingress,
}

pub mod app {
    use serde::Deserialize;
    use std::path::PathBuf;
    use std::time::Duration;

    #[derive(Debug, Deserialize)]
    pub struct Data {
        pub dir: PathBuf,
    }

    #[derive(Debug, Deserialize)]
    pub struct Cheat {
        #[serde(default)]
        pub enabled: bool,
    }

    #[derive(Debug, Deserialize)]
    pub struct Login {
        timeout_seconds: u8,
        #[serde(skip_deserializing)]
        pub timeout: Duration,
    }

    #[derive(Debug, Deserialize)]
    pub struct Ingress {
        pub protocols_rate_limit: Option<common::rate_limiter::Params>,
        pub bytes_rate_limit: Option<common::rate_limiter::Params>,
    }

    impl Login {
        pub fn init(&mut self) {
            self.timeout = Duration::from_secs(self.timeout_seconds as u64);
        }
    }
}

pub fn init(local_env: &Option<PathBuf>) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(local_env) = local_env {
        println!("Using local environment file \"{}\"", local_env.display());
        dotenvy::from_filename(local_env)?;
    }

    let net = load_network_config()?;
    let db = load_database_config()?;
    let app = load_application_config()?;

    let config = Config {
        net,
        db,
        app,
    };

    CONFIG.set(config).expect("Config already initialized");
    Ok(())
}

fn load_network_config() -> Result<NetworkConfig, Box<dyn std::error::Error>> {
    let mut config: NetworkConfig = config::Config::builder()
        .add_source(config::Environment::with_prefix("SPIRE"))
        .build()?
        .try_deserialize()?;
    config.token_key = common::io::read_file(&config.token_key_file)?.into_bytes();

    Ok(config)
}

fn load_database_config() -> Result<DatabaseConfig, Box<dyn std::error::Error>> {
    let mut config: DatabaseConfig = config::Config::builder()
        .add_source(config::Environment::with_prefix("SPIRE_DB"))
        .build()?
        .try_deserialize()?;
    config.password = common::io::read_file(&config.password_file)?;

    Ok(config)
}

fn load_application_config() -> Result<ApplicationConfig, Box<dyn std::error::Error>> {
    let mut config: ApplicationConfig = config::Config::builder()
        .add_source(config::File::with_name("config"))
        .build()?
        .try_deserialize()?;

    config.login.init();

    Ok(config)
}

pub fn get_tls_cert_chain() -> Result<Vec<CertificateDer<'static>>, Box<dyn std::error::Error>> {
    let cert_bytes = std::fs::read(&CONFIG.get().unwrap().net.tls_cert_file)?;
    let cert_chain = rustls_pemfile::certs(&mut cert_bytes.as_slice()).collect::<Result<Vec<_>, _>>()?;
    Ok(cert_chain)
}

pub fn get_tls_key() -> Result<PrivateKeyDer<'static>, Box<dyn std::error::Error>> {
    let key_bytes = std::fs::read(&CONFIG.get().unwrap().net.tls_key_file)?;
    let key = rustls_pemfile::private_key(&mut key_bytes.as_slice())?.ok_or(
        std::io::Error::new(std::io::ErrorKind::NotFound, "TLS private key file"),
    )?;
    Ok(key)
}

#[macro_export]
macro_rules! config {
    ($field:ident) => {{
        &$crate::config::CONFIG.get().unwrap().$field
    }};
}

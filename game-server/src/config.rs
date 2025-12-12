#![allow(static_mut_refs)]

use std::mem::MaybeUninit;
use quinn::rustls::pki_types::{CertificateDer, PrivateKeyDer};
use serde::Deserialize;
use std::path::PathBuf;

static mut CONFIG: MaybeUninit<Config> = MaybeUninit::uninit();

#[derive(Debug)]
pub struct Config {
    pub app: ApplicationConfig,
    pub auth: AuthConfig,
    pub db: DatabaseConfig,
    pub net: NetworkConfig,
}

#[derive(Debug, Deserialize)]
pub struct ApplicationConfig {
    pub data: app::Data,
    pub cheat: app::Cheat,
    pub zone: app::Zone,
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

    fn tick_interval_milliseconds_default() -> u8 { 50 }
    #[derive(Debug, Deserialize)]
    pub struct Zone {
        #[serde(default = "tick_interval_milliseconds_default")]
        tick_interval_milliseconds: u8,
        #[serde(skip_deserializing)]
        pub tick_interval: Duration,
    }

    impl Zone {
        pub fn init(&mut self) {
            self.tick_interval = Duration::from_millis(self.tick_interval_milliseconds as u64);
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct AuthConfig {
    pub login: auth::Login,

    #[serde(alias = "game_server_tls_cert_file")]
    tls_cert_file: PathBuf,
    #[serde(alias = "game_server_tls_key_file")]
    tls_key_file: PathBuf,

    #[serde(skip_deserializing)]
    pub token_key: Vec<u8>,
    token_key_file: PathBuf,
}

pub mod auth {
    use serde::Deserialize;
    use std::time::Duration;

    #[derive(Debug, Deserialize)]
    pub struct Login {
        timeout_seconds: u8,
        #[serde(skip_deserializing)]
        pub timeout: Duration,
    }

    impl Login {
        pub fn init(&mut self) {
            self.timeout = Duration::from_secs(self.timeout_seconds as u64);
        }
    }
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
pub struct NetworkConfig {
    #[serde(alias = "game_server_node_id")]
    pub node_id: u16,
    #[serde(alias = "game_server_port")]
    pub port: u16,
    #[serde(alias = "game_server_control_port")]
    pub control_port: u16,
    pub application_protocol: String,

    pub ingress: net::Ingress,

    // #[serde(alias = "game_server_tls_cert_file")]
    // tls_cert_file: PathBuf,
    // #[serde(alias = "game_server_tls_key_file")]
    // tls_key_file: PathBuf,
    //
    // #[serde(skip_deserializing)]
    // pub token_key: Vec<u8>,
    // token_key_file: PathBuf,
}

pub mod net {
    use serde::Deserialize;

    #[derive(Debug, Deserialize)]
    pub struct Ingress {
        pub protocols_rate_limit: Option<util::rate_limiter::Params>,
        pub bytes_rate_limit: Option<util::rate_limiter::Params>,
    }
}

pub fn init(local_env: &Option<PathBuf>) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(local_env) = local_env {
        println!("Using local environment file \"{}\"", local_env.display());
        dotenvy::from_filename(local_env)?;
    }

    let app = load_application_config()?;
    let auth = load_auth_config()?;
    let db = load_database_config()?;
    let net = load_network_config()?;

    let config = Config {
        app,
        auth,
        db,
        net,
    };

    unsafe {
        CONFIG.write(config);
    }
    Ok(())
}

fn load_application_config() -> Result<ApplicationConfig, Box<dyn std::error::Error>> {
    let mut config: ApplicationConfig = config::Config::builder()
        .add_source(config::File::with_name("config"))
        .build()?
        .try_deserialize()?;

    config.zone.init();

    Ok(config)
}

fn load_auth_config() -> Result<AuthConfig, Box<dyn std::error::Error>> {
    let mut config: AuthConfig = config::Config::builder()
        .add_source(config::File::with_name("config"))
        .add_source(config::Environment::with_prefix("SPIRE"))
        .build()?
        .try_deserialize()?;

    config.login.init();
    config.token_key = util::io::read_file(&config.token_key_file)?.into_bytes();

    Ok(config)
}

fn load_database_config() -> Result<DatabaseConfig, Box<dyn std::error::Error>> {
    let mut config: DatabaseConfig = config::Config::builder()
        .add_source(config::Environment::with_prefix("SPIRE_DB"))
        .build()?
        .try_deserialize()?;
    config.password = util::io::read_file(&config.password_file)?;

    Ok(config)
}

fn load_network_config() -> Result<NetworkConfig, Box<dyn std::error::Error>> {
    let config: NetworkConfig = config::Config::builder()
        .add_source(config::File::with_name("config"))
        .add_source(config::Environment::with_prefix("SPIRE"))
        .build()?
        .try_deserialize()?;

    Ok(config)
}

pub fn get_tls_cert_chain() -> Result<Vec<CertificateDer<'static>>, Box<dyn std::error::Error>> {
    let cert_bytes = std::fs::read(&get_config().auth.tls_cert_file)?;
    let cert_chain = rustls_pemfile::certs(&mut cert_bytes.as_slice()).collect::<Result<Vec<_>, _>>()?;
    Ok(cert_chain)
}

pub fn get_tls_key() -> Result<PrivateKeyDer<'static>, Box<dyn std::error::Error>> {
    let key_bytes = std::fs::read(&get_config().auth.tls_key_file)?;
    let key = rustls_pemfile::private_key(&mut key_bytes.as_slice())?.ok_or(
        std::io::Error::new(std::io::ErrorKind::NotFound, "TLS private key file"),
    )?;
    Ok(key)
}

pub fn get_config() -> &'static Config {
    unsafe { CONFIG.assume_init_ref() }
}

#[macro_export]
macro_rules! config {
    ($field:ident) => {{
        &$crate::config::get_config().$field
    }};
}

use serde::Deserialize;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use quinn::rustls::pki_types::{CertificateDer, PrivateKeyDer};

static CONFIG: OnceLock<Config> = OnceLock::new();

#[derive(Debug, Deserialize)]
pub struct Config {
    #[serde(alias = "game_server_port")]
    pub port: u16,

    #[serde(alias = "game_server_control_port")]
    pub control_port: u16,

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

    #[serde(alias = "game_server_tls_cert_file")]
    pub tls_cert_file: PathBuf,
    #[serde(alias = "game_server_tls_key_file")]
    pub tls_key_file: PathBuf,
    
    pub application_protocol: String,
}

impl Config {
    pub fn init() -> Result<(), Box<dyn std::error::Error>> {
        let mut config = config::Config::builder()
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

    pub fn get_tls_cert_chain() -> Result<Vec<CertificateDer<'static>>, std::io::Error> {
        let cert_bytes = std::fs::read(&Config::get().tls_cert_file)?;
        let cert_chain = rustls_pemfile::certs(&mut cert_bytes.as_slice())
            .collect::<Result<Vec<_>, _>>()?;

        Ok(cert_chain)
    }

    pub fn get_tls_key() -> Result<PrivateKeyDer<'static>, std::io::Error> {
        let key_bytes = std::fs::read(&&Config::get().tls_key_file)?;
        let key = rustls_pemfile::private_key(&mut key_bytes.as_slice())?
            .ok_or(std::io::Error::new(std::io::ErrorKind::NotFound, "TLS private key file"))?;

        Ok(key)
    }
}

fn read_from_file(path: &Path) -> Result<String, std::io::Error> {
    let mut f = File::open(path)?;
    let mut buf = String::new();
    f.read_to_string(&mut buf)?;

    Ok(buf.trim().to_string())
}

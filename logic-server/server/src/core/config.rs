use jsonwebtoken::DecodingKey;
use serde::Deserialize;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::str::FromStr;
use std::sync::OnceLock;

pub struct ServerConfig {
    pub game_listen_port: u16,
    pub admin_listen_port: u16,
}

impl ServerConfig {
    pub fn load() -> Self {
        let game_listen_port = u16::from_str(env!("SPIRE_GAME_LISTEN_PORT")).unwrap();
        let admin_listen_port = u16::from_str(env!("SPIRE_ADMIN_LISTEN_PORT")).unwrap();

        ServerConfig {
            game_listen_port,
            admin_listen_port,
        }
    }
}

pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub password: String,
    pub database: String,
}

impl DatabaseConfig {
    pub fn load() -> Self {
        let host = env!("SPIRE_DB_HOST").to_string();
        let port = u16::from_str(env!("SPIRE_DB_PORT")).unwrap();
        let user = env!("SPIRE_DB_USER").to_string();
        let password = read_from_file(Path::new(env!("SPIRE_DB_PASSWORD_FILE")));
        let database = env!("SPIRE_DB_NAME").to_string();

        DatabaseConfig {
            host,
            port,
            user,
            password,
            database,
        }
    }
}

pub struct AuthConfig {
    pub key: DecodingKey
}

impl AuthConfig {
    pub fn load() -> Self {
        let key = read_from_file(Path::new(env!("SPIRE_AUTH_KEY_FILE"))).into_bytes();
        let key = DecodingKey::from_secret(&key);

        AuthConfig {
            key
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub cheat_enabled: bool,
}

impl Config {
    pub fn init() {
        let path = Path::new(env!("OUT_DIR")).join("config.json");
        println!("Initializing config from {}...", path.display());
        let config: Config = serde_json::from_str(&read_from_file(&path)).unwrap();

        CONFIG.set(config).unwrap();
        println!("Initializing config done!");
    }
}

static CONFIG: OnceLock<Config> = OnceLock::new();
pub fn config() -> &'static Config {
    &CONFIG.get().unwrap()
}

fn read_from_file(path: &Path) -> String {
    let mut f = File::open(path).unwrap();
    let mut buf = String::new();
    f.read_to_string(&mut buf).unwrap();

    buf.trim().to_string()
}

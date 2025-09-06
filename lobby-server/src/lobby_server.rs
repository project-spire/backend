use jsonwebtoken::EncodingKey;
use crate::config::config;
use crate::db;

#[derive(Clone)]
pub struct LobbyServer {
    pub db_pool: db::Pool,
    pub encoding_key: EncodingKey,
}

impl LobbyServer {
    pub fn new(db_pool: db::Pool) -> Self {
        let encoding_key = EncodingKey::from_secret(&config().token_key);

        Self { db_pool, encoding_key }
    }
}

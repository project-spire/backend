use jsonwebtoken::EncodingKey;
use crate::config::Config;
use crate::db;

#[derive(Clone)]
pub struct LobbyServer {
    db_client: db::Client,
    encoding_key: EncodingKey,
}

impl LobbyServer {
    pub fn new(db_client: db::Client) -> Self {
        let encoding_key = EncodingKey::from_secret(&Config::get().token_key);

        Self { db_client, encoding_key }
    }
}

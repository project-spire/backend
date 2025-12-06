mod new_connection;

pub use new_connection::NewConnection;

use actix::prelude::*;
use common::token;
use crate::config::config;
use crate::net::session::Entry;
use jsonwebtoken::DecodingKey;
use protocol::game::auth::*;

pub struct Authenticator {
    decoding_key: DecodingKey,
}

impl Default for Authenticator {
    fn default() -> Self {
        let decoding_key = DecodingKey::from_secret(&config().token_key);

        Self { decoding_key }
    }
}

impl Actor for Authenticator {
    type Context = Context<Self>;
}

impl Supervised for Authenticator {}

impl SystemService for Authenticator {}

impl Authenticator {
    fn validate_login(
        &self,
        login: &Login,
    ) -> Result<(Entry, login::Kind), Box<dyn std::error::Error>> {
        let claims = token::verify(&login.token, &self.decoding_key)?;
        let entry = Entry {
            account_id: claims.account_id,
            character_id: login.character_id,
        };
        let login_kind = login::Kind::try_from(login.kind)?;

        Ok((entry, login_kind))
    }
}

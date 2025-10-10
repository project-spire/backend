use jsonwebtoken::DecodingKey;
use tonic::{Request, Status};
use tonic::service::Interceptor;

use crate::config::config;
use util::token;

#[derive(Clone)]
pub struct Authenticator {
    decoding_key: DecodingKey,
}

impl Authenticator {
    pub fn new() -> Self {
        let decoding_key = DecodingKey::from_secret(&config().token_key);

        Self { decoding_key }
    }
}

impl Interceptor for Authenticator {
    fn call(&mut self, mut request: Request<()>) -> Result<Request<()>, Status> {
        let token = request
            .metadata()
            .get("authentication")
            .and_then(|v| v.to_str().ok());

        let token = if let Some(token) = token {
            token
        } else {
            return Err(Status::unauthenticated("Missing authentication token"));
        };

        let claims = match token::verify(
            token,
            &self.decoding_key
        ) {
            Ok(claims) => claims,
            Err(e) => return Err(Status::unauthenticated(
                format!("Invalid authentication token: {}", e)
            )),
        };

        request.extensions_mut().insert(claims);

        Ok(request)
    }
}

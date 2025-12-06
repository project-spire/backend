use chrono::Utc;
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Validation};
use serde::{Deserialize, Serialize};

const ALGORITHM: Algorithm = Algorithm::HS256;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Claims {
    #[serde(rename = "aid")]
    pub account_id: i64,

    #[serde(rename = "exp")]
    pub expire: i64,
}

pub fn generate(
    account_id: i64,
    encoding_key: &EncodingKey,
    expiration: std::time::Duration,
) -> Result<String, jsonwebtoken::errors::Error> {
    let expire = (Utc::now() + expiration).timestamp();

    let claims = Claims {
        account_id,
        expire,
    };

    jsonwebtoken::encode(
        &jsonwebtoken::Header::new(ALGORITHM),
        &claims,
        &encoding_key,
    )
}

pub fn verify(
    token: &str,
    decoding_key: &DecodingKey
) -> Result<Claims, Box<dyn std::error::Error>> {
    let validation = Validation::new(ALGORITHM);
    let claims = jsonwebtoken::decode::<Claims>(
        token,
        decoding_key,
        &validation,
    ).map(|data| data.claims)?;

    Ok(claims)
}

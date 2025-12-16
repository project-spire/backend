use chrono::Utc;
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Validation};
use serde::{Deserialize, Serialize};

const ALGORITHM: Algorithm = Algorithm::HS256;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Claims {
    #[serde(rename = "iat")]
    pub issue: i64,

    #[serde(rename = "exp")]
    pub expire: i64,

    #[serde(rename = "aid")]
    pub account_id: i64,
}

pub fn generate(
    account_id: i64,
    encoding_key: &EncodingKey,
    expiration: std::time::Duration,
) -> Result<String, jsonwebtoken::errors::Error> {
    let now = Utc::now();
    let issue = now.timestamp();
    let expire = (now + expiration).timestamp();

    let claims = Claims {
        issue,
        expire,
        account_id,
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
) -> Result<Claims, jsonwebtoken::errors::Error> {
    let validation = Validation::new(ALGORITHM);
    let claims = jsonwebtoken::decode::<Claims>(
        token,
        decoding_key,
        &validation,
    ).map(|data| data.claims)?;

    Ok(claims)
}

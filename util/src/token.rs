use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Validation};
use serde::{Deserialize, Serialize};

const ALGORITHM: Algorithm = Algorithm::HS256;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Claims {
    #[serde(alias = "aid")]
    pub account_id: i64,
}

pub fn generate(
    account_id: i64,
    encoding_key: &EncodingKey
) -> Result<String, jsonwebtoken::errors::Error> {
    let claims = Claims {
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
) -> Result<Claims, Box<dyn std::error::Error>> {
    let validation = Validation::new(ALGORITHM);
    let claims = jsonwebtoken::decode::<Claims>(
        token,
        decoding_key,
        &validation,
    ).map(|data| data.claims)?;

    Ok(claims)
}

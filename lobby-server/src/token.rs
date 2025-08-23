use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct RawClaims {
    pub aid: String, // account_id
}

#[derive(Clone)]
pub struct Claims {
    pub account_id: Uuid,
}

impl TryFrom<RawClaims> for Claims {
    type Error = ();

    fn try_from(value: RawClaims) -> Result<Self, Self::Error> {
        let account_id = match Uuid::try_parse(&value.aid) {
            Ok(id) => id,
            Err(_) => return Err(())
        };

        Ok(Self { account_id })
    }
}

pub fn generate_token(
    account_id: Uuid,
    algorithm: Algorithm,
    encoding_key: &EncodingKey
) -> Result<String, jsonwebtoken::errors::Error> {
    let claims = RawClaims {
        aid: account_id.to_string(),
    };

    jsonwebtoken::encode(
        &jsonwebtoken::Header::new(algorithm),
        &claims,
        &encoding_key,
    )
}

pub fn verify_token(
    token: &str,
    validation: &Validation,
    decoding_key: &DecodingKey
) -> Result<Claims, Box<dyn std::error::Error>> {
    let claims = jsonwebtoken::decode::<RawClaims>(
        token,
        decoding_key,
        validation,
    ).map(|data| data.claims)?;

    Claims::try_from(claims)
        .map_err(|_| std::io::Error::new(std::io::ErrorKind::Other, "Invalid account id").into())
}

use chrono::Duration;
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    user_id: String,
    username: String,
    exp: u64,
}

pub struct JwtService {
    encoding: EncodingKey,
    decoding: DecodingKey,
}

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("JWT Error: {0}")]
    Jwt(#[from] jsonwebtoken::errors::Error),
}

impl JwtService {
    const TOKEN_LIFE: Duration = Duration::hours(24);

    pub fn new(secret: &str) -> Result<Self, AuthError> {
        Ok(Self {
            decoding: DecodingKey::from_base64_secret(secret)?,
            encoding: EncodingKey::from_base64_secret(secret)?,
        })
    }

    pub fn generate_token(&self, user_id: &str, username: &str) -> Result<String, AuthError> {
        let exp = jsonwebtoken::get_current_timestamp() + Self::TOKEN_LIFE.as_seconds_f32() as u64;

        let claims = Claims {
            user_id: user_id.into(),
            username: username.into(),
            exp,
        };

        jsonwebtoken::encode(
            &Header::new(jsonwebtoken::Algorithm::HS512),
            &claims,
            &self.encoding,
        )
        .map_err(Into::into)
    }

    pub fn verify_token(&self, token: String) -> Result<Claims, AuthError> {
        Ok(jsonwebtoken::decode::<Claims>(
            token,
            &self.decoding,
            &Validation::new(jsonwebtoken::Algorithm::HS512),
        )?
        .claims)
    }
}

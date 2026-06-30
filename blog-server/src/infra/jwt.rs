use chrono::Duration;
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    user_id: String,
    username: String,
    exp: u64,
}

#[derive(Debug, thiserror::Error)]
pub enum JwtError {
    /// Decoding/Encoding errors, mapped from jsonwebtoken's
    #[error("Encoding/Decoding Error: {0}")]
    WebToken(#[from] jsonwebtoken::errors::Error),

    /// Env variable error
    #[error("JWT Secret Missing. Set the JWT_TOKEN env var")]
    Env,
}

pub struct JwtService {
    encoding: EncodingKey,
    decoding: DecodingKey,
}

impl JwtService {
    const TOKEN_LIFE: Duration = Duration::hours(24);

    pub fn new() -> Result<Self, JwtError> {
        // не уверен, что тут это надо делать
        let secret = env::var("JWT_SECRET").map_err(|_| JwtError::Env)?;

        Ok(Self {
            decoding: DecodingKey::from_base64_secret(&secret)?,
            encoding: EncodingKey::from_base64_secret(&secret)?,
        })
    }

    pub fn generate_token(
        &self,
        user_id: &str,
        username: &str,
    ) -> Result<String, JwtError> {
        let exp = jsonwebtoken::get_current_timestamp()
            + Self::TOKEN_LIFE.as_seconds_f32() as u64;

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

    pub fn verify_token(&self, token: String) -> Result<Claims, JwtError> {
        Ok(jsonwebtoken::decode::<Claims>(
            token,
            &self.decoding,
            &Validation::new(jsonwebtoken::Algorithm::HS512),
        )?
        .claims)
    }
}

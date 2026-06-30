use crate::domain::Username;
use chrono::Duration;
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::{env, sync::Arc};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    user_id: i64,
    username: Username,
    exp: u64,
}

impl Claims {
    pub fn get_user_id(&self) -> i64 {
        self.user_id
    }
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

    pub fn new() -> Result<Arc<Self>, JwtError> {
        // не уверен, что тут это надо делать
        let secret = env::var("JWT_SECRET").map_err(|_| JwtError::Env)?;

        Ok(Arc::new(Self {
            decoding: DecodingKey::from_base64_secret(&secret)?,
            encoding: EncodingKey::from_base64_secret(&secret)?,
        }))
    }

    pub fn generate_token(
        &self,
        user_id: i64,
        username: &str,
    ) -> Result<Token, JwtError> {
        let exp = jsonwebtoken::get_current_timestamp()
            + Self::TOKEN_LIFE.as_seconds_f32() as u64;

        let claims = Claims {
            user_id,
            username: username.into(),
            exp,
        };

        Ok(Token::new(jsonwebtoken::encode(
            &Header::new(jsonwebtoken::Algorithm::HS512),
            &claims,
            &self.encoding,
        )?))
    }

    pub fn verify_token(&self, token: Token) -> Result<Claims, JwtError> {
        Ok(jsonwebtoken::decode::<Claims>(
            token,
            &self.decoding,
            &Validation::new(jsonwebtoken::Algorithm::HS512),
        )?
        .claims)
    }
}

#[derive(Debug, Serialize)]
#[serde(transparent)]
pub struct Token(String);

impl Token {
    pub fn new(s: String) -> Self {
        Self(s)
    }
}

impl AsRef<[u8]> for Token {
    fn as_ref(&self) -> &[u8] {
        self.0.as_bytes()
    }
}

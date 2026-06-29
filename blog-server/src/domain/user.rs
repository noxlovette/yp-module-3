use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

type Username = String;
type Email = String;

#[derive(Debug, Deserialize, Clone)]
#[serde(transparent)]
struct Password(String);

#[derive(Serialize, Debug, Clone)]
pub struct User {
    id: i64,
    username: Username,
    email: Email,
    password_hash: String,
    created_at: DateTime<Utc>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct SignupPayload {
    username: Username,
    password: Password,
    email: Email,
}

#[derive(Deserialize, Debug, Clone)]
pub struct LoginPayload {
    username: Username,
    password: Email,
}

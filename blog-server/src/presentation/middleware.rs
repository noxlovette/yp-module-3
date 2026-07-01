use crate::{domain::Username, infra::Token, presentation::http::AppState};
use actix_web::{Error, HttpMessage, dev::ServiceRequest, error::ErrorUnauthorized, web};
use actix_web_httpauth::extractors::bearer::BearerAuth;

/// What handlers/other middleware see once a request has passed
/// `jwt_validator`. Kept separate from `infra::Claims` so presentation
/// code doesn't need to know anything about JWTs.
#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    pub user_id: i64,
    pub username: Username,
}

pub async fn jwt_validator(
    req: ServiceRequest,
    credentials: BearerAuth,
) -> Result<ServiceRequest, (Error, ServiceRequest)> {
    let jwt = match req.app_data::<web::Data<AppState>>() {
        Some(state) => state.jwt().clone(),
        None => return Err((ErrorUnauthorized("missing app state"), req)),
    };

    let token = Token::new(credentials.token().to_string());

    match jwt.verify_token(token) {
        Ok(claims) => {
            req.extensions_mut().insert(AuthenticatedUser {
                user_id: claims.get_user_id(),
                username: claims.get_username().clone(),
            });
            Ok(req)
        }
        Err(_) => Err((ErrorUnauthorized("invalid or expired token"), req)),
    }
}

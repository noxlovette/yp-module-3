use reqwest::StatusCode;
use thiserror::Error;

pub type BlogClientResult<T> = Result<T, BlogClientError>;

#[derive(Debug, Error)]
pub enum BlogClientError {
    #[error("HTTP transport error: {0}")]
    Network(#[source] reqwest::Error),

    #[error("gRPC call failed: {0}")]
    Grpc(#[from] tonic::Status),

    #[error("gRPC transport error: {0}")]
    GrpcTransport(#[from] tonic::transport::Error),

    #[error("resource not found")]
    NotFound,

    #[error("upstream error: {0}")]
    Upstream(StatusCode),

    #[error("authentication required or token invalid/expired")]
    Unauthorized,

    #[error("forbidden")]
    Forbidden,

    #[error("invalid request: {0}")]
    InvalidRequest(String),

    #[error("invalid params. offset and limit must be positive")]
    InvalidParams,
}

impl From<reqwest::Error> for BlogClientError {
    fn from(value: reqwest::Error) -> Self {
        match value.status() {
            Some(StatusCode::NOT_FOUND) => Self::NotFound,
            Some(StatusCode::UNAUTHORIZED) => Self::Unauthorized,
            Some(StatusCode::FORBIDDEN) => Self::Forbidden,
            Some(s) => Self::Upstream(s),
            None => Self::Network(value),
        }
    }
}

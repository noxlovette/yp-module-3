use thiserror::Error;

pub type BlogClientResult<T> = Result<T, BlogClientError>;

#[derive(Debug, Error)]
pub enum BlogClientError {
    #[error("HTTP transport error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("gRPC call failed: {0}")]
    Grpc(#[from] tonic::Status),

    #[error("gRPC transport error: {0}")]
    GrpcTransport(#[from] tonic::transport::Error),

    #[error("resource not found")]
    NotFound,

    #[error("authentication required or token invalid/expired")]
    Unauthorized,

    #[error("invalid request: {0}")]
    InvalidRequest(String),
}

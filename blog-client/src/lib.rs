pub mod error;
pub mod grpc_client;
pub mod http_client;

use blog_proto::{
    CreatePostRequest, CreatePostResponse, DeletePostRequest, GetPostRequest,
    ListPostsRequest, ListPostsResponse, LoginRequest, LoginResponse, Post,
    RegisterRequest, RegisterResponse, UpdatePostRequest,
};
use tonic::{Request, transport::Endpoint};

use crate::{
    error::{BlogClientError, BlogClientResult},
    grpc_client::GrpcClient,
    http_client::HttpClient,
};

pub enum Transport {
    Http(String),
    Grpc(String),
}

pub struct BlogClient {
    token: Option<String>,
    kind: ClientKind,
}

pub enum ClientKind {
    Http(HttpClient),
    Grpc(GrpcClient),
}

pub struct Limit(i64);
pub struct Offset(i64);

impl Default for Limit {
    fn default() -> Self {
        Self(10)
    }
}

impl Default for Offset {
    fn default() -> Self {
        Self(0)
    }
}

impl Offset {
    pub fn get(&self) -> i64 {
        self.0
    }
}

impl Limit {
    pub fn get(&self) -> i64 {
        self.0
    }
}

/// Attaches `Bearer <token>` to an outgoing gRPC request's metadata.
fn with_auth<T>(
    mut req: Request<T>,
    token: &str,
) -> BlogClientResult<Request<T>> {
    req.metadata_mut().insert(
        "authorization",
        format!("Bearer {token}").parse().map_err(|_| {
            BlogClientError::InvalidRequest("invalid token".into())
        })?,
    );
    Ok(req)
}

impl BlogClient {
    pub async fn new(transport: Transport) -> BlogClientResult<Self> {
        match transport {
            Transport::Http(base_url) => Ok(Self {
                kind: ClientKind::Http(HttpClient::new(base_url)),
                token: None,
            }),
            Transport::Grpc(addr) => Ok(Self {
                kind: ClientKind::Grpc(GrpcClient::new(
                    Endpoint::from_shared(addr)?.connect().await?,
                )),
                token: None,
            }),
        }
    }

    pub fn set_token(&mut self, token: impl Into<String>) {
        self.token = Some(token.into());
    }
    pub fn get_token(&self) -> Option<&str> {
        self.token.as_deref()
    }

    fn require_token(&self) -> BlogClientResult<String> {
        self.token.clone().ok_or(BlogClientError::Unauthorized)
    }

    pub async fn register(
        &mut self,
        username: String,
        email: String,
        password: String,
    ) -> BlogClientResult<RegisterResponse> {
        let r = RegisterRequest {
            username,
            email,
            password,
        };
        let auth = match &mut self.kind {
            ClientKind::Http(c) => c.register(r).await?,
            ClientKind::Grpc(c) => c.register(r).await?.into_inner(),
        };

        self.token = Some(auth.token.clone());
        Ok(auth)
    }

    pub async fn login(
        &mut self,
        username: String,
        password: String,
    ) -> BlogClientResult<LoginResponse> {
        let r = LoginRequest { username, password };
        let auth = match &mut self.kind {
            ClientKind::Http(c) => c.login(r).await?,
            ClientKind::Grpc(c) => c.login(r).await?.into_inner(),
        };

        self.token = Some(auth.token.clone());
        Ok(auth)
    }

    pub async fn create_post(
        &mut self,
        title: String,
        content: String,
    ) -> BlogClientResult<CreatePostResponse> {
        let token = self.require_token()?;
        let r = CreatePostRequest { title, content };
        match &mut self.kind {
            ClientKind::Http(c) => c.create_post(&token, r).await,
            ClientKind::Grpc(c) => {
                let req = with_auth(Request::new(r), &token)?;
                Ok(c.create_post(req).await?.into_inner())
            }
        }
    }

    pub async fn get_post(&mut self, id: i64) -> BlogClientResult<Post> {
        let token = self.require_token()?;
        match &mut self.kind {
            ClientKind::Http(c) => c.get_post(&token, id).await,
            ClientKind::Grpc(c) => {
                let req =
                    with_auth(Request::new(GetPostRequest { id }), &token)?;
                Ok(c.get_post(req)
                    .await?
                    .into_inner()
                    .post
                    .ok_or(BlogClientError::NotFound)?)
            }
        }
    }

    pub async fn update_post(
        &mut self,
        id: i64,
        title: &str,
        content: &str,
    ) -> BlogClientResult<Post> {
        let token = self.require_token()?;
        match &mut self.kind {
            ClientKind::Http(c) => {
                c.update_post(&token, id, title, content).await
            }
            ClientKind::Grpc(c) => {
                let req = with_auth(
                    Request::new(UpdatePostRequest {
                        id,
                        title: title.to_string(),
                        content: content.to_string(),
                    }),
                    &token,
                )?;
                Ok(c.update_post(req)
                    .await?
                    .into_inner()
                    .post
                    .ok_or(BlogClientError::NotFound)?)
            }
        }
    }

    pub async fn delete_post(&mut self, id: i64) -> BlogClientResult<()> {
        let token = self.require_token()?;
        match &mut self.kind {
            ClientKind::Http(c) => c.delete_post(&token, id).await,
            ClientKind::Grpc(c) => {
                let req =
                    with_auth(Request::new(DeletePostRequest { id }), &token)?;
                c.delete_post(req).await?;
                Ok(())
            }
        }
    }

    pub async fn list_posts(
        &mut self,
        limit: Option<Limit>,
        offset: Option<Offset>,
    ) -> BlogClientResult<ListPostsResponse> {
        let token = self.require_token()?;
        let posts = match &mut self.kind {
            ClientKind::Http(c) => c.list_posts(&token, limit, offset).await?,
            ClientKind::Grpc(c) => {
                let req = with_auth(
                    Request::new(ListPostsRequest {
                        limit: limit.map(|l| l.get()),
                        offset: offset.map(|o| o.get()),
                    }),
                    &token,
                )?;
                c.list_posts(req).await?.into_inner().posts
            }
        };

        Ok(ListPostsResponse { posts })
    }
}

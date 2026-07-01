use std::sync::Arc;

use blog_proto::{
    CreatePostRequest, CreatePostResponse, DeletePostRequest,
    DeletePostResponse, GetPostRequest, GetPostResponse, ListPostsRequest,
    ListPostsResponse, LoginRequest, LoginResponse, Post as ProtoPost,
    RegisterRequest, RegisterResponse, UpdatePostRequest, UpdatePostResponse,
    blog_service_server::BlogService as BlogServiceTrait,
};
use tonic::{Request, Response, Status};

use crate::{
    application::{AuthService, BlogService},
    domain::{
        DomainError, Email, LoginPayload, Password, PostUpsert, SignupPayload,
        Username,
    },
    infra::Token,
};

impl From<crate::domain::Post> for ProtoPost {
    fn from(p: crate::domain::Post) -> Self {
        ProtoPost {
            id: p.id(),
            title: p.title().to_string(),
            content: p.content().to_string(),
            author_id: p.author_id(),
            created_at: p.created_at().timestamp(),
            updated_at: p.updated_at().timestamp(),
        }
    }
}

impl From<DomainError> for Status {
    fn from(err: DomainError) -> Self {
        match err {
            DomainError::UserNotFound | DomainError::PostNotFound => {
                Status::not_found(err.to_string())
            }
            DomainError::UsernameAlreadyExists
            | DomainError::EmailAlreadyExists => {
                Status::already_exists(err.to_string())
            }
            DomainError::InvalidCredentials | DomainError::Unauthorized => {
                Status::unauthenticated(err.to_string())
            }
            DomainError::Forbidden => {
                Status::permission_denied(err.to_string())
            }
            DomainError::TypeMismatch
            | DomainError::Parsing(_)
            | DomainError::Password(_) => {
                Status::invalid_argument(err.to_string())
            }
            DomainError::Database(_) => Status::internal(err.to_string()),
        }
    }
}

pub struct BlogGrpcService {
    auth: Arc<AuthService>,
    blog: Arc<BlogService>,
}

impl BlogGrpcService {
    pub fn new(auth: Arc<AuthService>, blog: Arc<BlogService>) -> Self {
        Self { auth, blog }
    }

    fn authenticate<T>(&self, request: &Request<T>) -> Result<i64, Status> {
        let header = request
            .metadata()
            .get("authorization")
            .ok_or_else(|| {
                Status::unauthenticated("missing authorization metadata")
            })?
            .to_str()
            .map_err(|_| {
                Status::unauthenticated("invalid authorization metadata")
            })?;

        let token = header.strip_prefix("Bearer ").ok_or_else(|| {
            Status::unauthenticated("expected a Bearer token")
        })?;

        let claims = self
            .auth
            .jwt()
            .verify_token(Token::new(token.to_string()))
            .map_err(|_| {
                tracing::warn!(
                    "grpc request rejected: invalid or expired token"
                );
                Status::unauthenticated("invalid or expired token")
            })?;

        Ok(claims.get_user_id())
    }
}

#[tonic::async_trait]
impl BlogServiceTrait for BlogGrpcService {
    async fn register(
        &self,
        request: Request<RegisterRequest>,
    ) -> Result<Response<RegisterResponse>, Status> {
        let req = request.into_inner();

        let payload = SignupPayload {
            username: Username::parse(req.username)
                .map_err(DomainError::from)?,
            password: Password::parse(req.password)
                .map_err(DomainError::from)?,
            email: Email::parse(req.email).map_err(DomainError::from)?,
        };

        let user_token = self.auth.signup(payload).await?;

        Ok(Response::new(RegisterResponse {
            token: user_token.token().as_str().to_string(),
        }))
    }

    async fn login(
        &self,
        request: Request<LoginRequest>,
    ) -> Result<Response<LoginResponse>, Status> {
        let req = request.into_inner();

        let payload = LoginPayload::Username {
            username: Username::parse(req.username)
                .map_err(DomainError::from)?,
            password: Password::parse(req.password)
                .map_err(DomainError::from)?,
        };

        let user_token = self.auth.login(payload).await?;

        Ok(Response::new(LoginResponse {
            token: user_token.token().as_str().to_string(),
        }))
    }

    async fn create_post(
        &self,
        request: Request<CreatePostRequest>,
    ) -> Result<Response<CreatePostResponse>, Status> {
        let user_id = self.authenticate(&request)?;
        tracing::Span::current().record("user_id", user_id);
        let req = request.into_inner();

        let post = self
            .blog
            .create(
                user_id,
                PostUpsert {
                    title: req.title,
                    content: req.content,
                },
            )
            .await?;

        Ok(Response::new(CreatePostResponse {
            post: Some(post.into()),
        }))
    }

    async fn update_post(
        &self,
        request: Request<UpdatePostRequest>,
    ) -> Result<Response<UpdatePostResponse>, Status> {
        let user_id = self.authenticate(&request)?;
        tracing::Span::current().record("user_id", user_id);
        let req = request.into_inner();

        let post = self
            .blog
            .update(
                user_id,
                req.id,
                PostUpsert {
                    title: req.title,
                    content: req.content,
                },
            )
            .await?;

        Ok(Response::new(UpdatePostResponse {
            post: Some(post.into()),
        }))
    }

    async fn delete_post(
        &self,
        request: Request<DeletePostRequest>,
    ) -> Result<Response<DeletePostResponse>, Status> {
        let user_id = self.authenticate(&request)?;
        tracing::Span::current().record("user_id", user_id);
        let id = request.into_inner().id;

        self.blog.delete(user_id, id).await?;

        Ok(Response::new(DeletePostResponse {}))
    }

    async fn get_post(
        &self,
        request: Request<GetPostRequest>,
    ) -> Result<Response<GetPostResponse>, Status> {
        let user_id = self.authenticate(&request)?;
        tracing::Span::current().record("user_id", user_id);
        let id = request.into_inner().id;
        let post = self.blog.get(user_id, id).await?;

        Ok(Response::new(GetPostResponse {
            post: Some(post.into()),
        }))
    }

    async fn list_posts(
        &self,
        request: Request<ListPostsRequest>,
    ) -> Result<Response<ListPostsResponse>, Status> {
        let user_id = self.authenticate(&request)?;
        tracing::Span::current().record("user_id", user_id);
        let posts = self.blog.list(user_id).await?;

        Ok(Response::new(ListPostsResponse {
            posts: posts.into_iter().map(Into::into).collect(),
        }))
    }
}

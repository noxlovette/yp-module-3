use blog_proto::{
    CreatePostRequest, CreatePostResponse, LoginRequest, LoginResponse, Post,
    RegisterRequest, RegisterResponse, UpdatePostRequest, User,
};
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::{
    Limit, Offset,
    error::{BlogClientError, BlogClientResult},
};

pub struct HttpClient {
    http: Client,
    base_url: String,
}

impl HttpClient {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            http: Client::new(),
            base_url: base_url.into(),
        }
    }

    fn url(&self, path: &str) -> String {
        format!("{}{path}", self.base_url)
    }
}

// blog-server's `LoginPayload` is an externally-tagged enum, so a
// username/password login has to be sent as `{"Username": {...}}`.
#[derive(Serialize)]
enum LoginPayloadDto {
    Username { username: String, password: String },
}

#[derive(Deserialize)]
struct AuthDto {
    token: String,
    user: User,
}

async fn map_error(resp: reqwest::Response) -> BlogClientError {
    match resp.status() {
        reqwest::StatusCode::NOT_FOUND => BlogClientError::NotFound,
        reqwest::StatusCode::UNAUTHORIZED | reqwest::StatusCode::FORBIDDEN => {
            BlogClientError::Unauthorized
        }
        status => {
            let body = resp.text().await.unwrap_or_default();
            BlogClientError::InvalidRequest(format!("{status}: {body}"))
        }
    }
}

impl HttpClient {
    pub async fn register(
        &self,
        r: RegisterRequest,
    ) -> BlogClientResult<RegisterResponse> {
        let resp = self
            .http
            .post(self.url("/api/auth/register"))
            .json(&r)
            .send()
            .await?;

        if !resp.status().is_success() {
            return Err(map_error(resp).await);
        }

        let auth: AuthDto = resp.json().await?;
        Ok(RegisterResponse {
            token: auth.token,
            user: Some(auth.user),
        })
    }

    pub async fn login(
        &self,
        r: LoginRequest,
    ) -> BlogClientResult<LoginResponse> {
        let payload = LoginPayloadDto::Username {
            username: r.username,
            password: r.password,
        };

        let resp = self
            .http
            .post(self.url("/api/auth/login"))
            .json(&payload)
            .send()
            .await?;

        if !resp.status().is_success() {
            return Err(map_error(resp).await);
        }

        let auth: AuthDto = resp.json().await?;
        Ok(LoginResponse {
            token: auth.token,
            user: Some(auth.user),
        })
    }

    pub async fn create_post(
        &self,
        token: &str,
        r: CreatePostRequest,
    ) -> BlogClientResult<CreatePostResponse> {
        let resp = self
            .http
            .post(self.url("/api/posts/"))
            .bearer_auth(token)
            .json(&r)
            .send()
            .await?;

        if !resp.status().is_success() {
            return Err(map_error(resp).await);
        }

        Ok(CreatePostResponse {
            post: Some(resp.json().await?),
        })
    }

    pub async fn get_post(
        &self,
        token: &str,
        id: i64,
    ) -> BlogClientResult<Post> {
        let resp = self
            .http
            .get(self.url(&format!("/api/posts/{id}")))
            .bearer_auth(token)
            .send()
            .await?;

        if !resp.status().is_success() {
            return Err(map_error(resp).await);
        }

        Ok(resp.json().await?)
    }

    pub async fn update_post(
        &self,
        token: &str,
        id: i64,
        title: &str,
        content: &str,
    ) -> BlogClientResult<Post> {
        // The proto request carries `id` for the gRPC path, but the HTTP
        // route takes it from the URL; the server ignores unknown JSON
        // fields, so sending it in the body too is harmless.
        let r = UpdatePostRequest {
            id,
            title: title.to_string(),
            content: content.to_string(),
        };

        let resp = self
            .http
            .put(self.url(&format!("/api/posts/{id}")))
            .bearer_auth(token)
            .json(&r)
            .send()
            .await?;

        if !resp.status().is_success() {
            return Err(map_error(resp).await);
        }

        Ok(resp.json().await?)
    }

    pub async fn delete_post(
        &self,
        token: &str,
        id: i64,
    ) -> BlogClientResult<()> {
        let resp = self
            .http
            .delete(self.url(&format!("/api/posts/{id}")))
            .bearer_auth(token)
            .send()
            .await?;

        if !resp.status().is_success() {
            return Err(map_error(resp).await);
        }

        Ok(())
    }

    pub async fn list_posts(
        &self,
        token: &str,
        limit: Option<Limit>,
        offset: Option<Offset>,
    ) -> BlogClientResult<Vec<Post>> {
        let query = [
            limit.map(|l| ("limit", l.get())),
            offset.map(|o| ("offset", o.get())),
        ];

        let resp = self
            .http
            .get(self.url("/api/posts/"))
            .bearer_auth(token)
            .query(&query.into_iter().flatten().collect::<Vec<_>>())
            .send()
            .await?;

        if !resp.status().is_success() {
            return Err(map_error(resp).await);
        }

        Ok(resp.json().await?)
    }
}

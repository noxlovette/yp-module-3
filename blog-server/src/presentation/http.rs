use actix_web::web;

pub struct AppState {
    auth_service: AuthService,
    blog_service: BlogService,
}

type IdPath = web::Path<i64>;

pub mod auth {
    use crate::domain::{LoginPayload, SignupPayload};
    use actix_web::{
        App, HttpResponse, HttpServer, Responder, get, post, web::Json,
    };

    #[post("/register")]
    pub async fn register(payload: Json<SignupPayload>) -> impl Responder {
        todo!("authservice signs the user up")
    }

    #[post("/login")]
    pub async fn login(payload: Json<LoginPayload>) -> impl Responder {
        todo!("authservice logs the user in")
    }
}
pub mod posts {

    use super::IdPath;
    use crate::domain::PostUpsert;
    use actix_web::{Responder, delete, get, post, put, web::Json};

    #[get("/{id}")]
    pub async fn get_post(path: IdPath) -> impl Responder {
        let id = path.into_inner();

        todo!("postser")
    }

    #[put("/{id}")]
    pub async fn update_post(
        path: IdPath,
        payload: Json<PostUpsert>,
    ) -> impl Responder {
        let id = path.into_inner();
    }
    #[delete("/{id}")]
    pub async fn delete_post(path: IdPath) -> impl Responder {
        let id = path.into_inner();
    }

    #[get("/")]
    pub async fn list_posts() -> impl Responder {
        todo!()
    }

    #[post("/")]
    pub async fn create_post(payload: Json<PostUpsert>) -> impl Responder {
        todo!()
    }
}

mod routes;

use crate::routes::v1::register::Claims;
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::http::StatusCode;
use axum::{routing::{get, post}, Extension, Router};
use diesel::{r2d2::{self, ConnectionManager}, PgConnection};
use dotenvy::dotenv;
use jsonwebtoken::{decode, DecodingKey, Validation};
use tower_http::trace::TraceLayer;
use tracing_subscriber;
use uuid::Uuid;

type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;

#[derive(Clone)]
pub struct AppState {
    pub db: DbPool,
    pub jwt_secret: String,
}

fn establish_connection() -> DbPool {
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    let pool = r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool.");

    pool
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter("info,tower_http=debug")
        .init();
    let pool = establish_connection();
    let jwt_secret = std::env::var("JWT_SECRET")
        .expect("JWT_SECRET must be set");
    let state = AppState {
        db: pool,
        jwt_secret,
    };
    let app = Router::new()
        .route("/v1/register", post(routes::v1::register::register_phone))
        .route("/v1/register/confirm", post(routes::v1::register::register_confirm))
        .route("/v1/keys/upload", post(routes::v1::keys::upload_keys))
        .route("/v1/devices", get(routes::v1::devices::get_devices))
        .route("/v1/messages", get(routes::v1::messages::get_messages))
        .layer(TraceLayer::new_for_http())
        .layer(Extension(state))
    ;

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("[Server] Started on http://0.0.0.0:3000");
    axum::serve(listener, app).await.unwrap();
}

pub struct AuthUser {
    pub user_id: Uuid,
    pub device_id: Uuid,
}

impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, String);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get("Authorization")
            .and_then(|v| v.to_str().ok())
            .ok_or((StatusCode::UNAUTHORIZED, "Unauthorized".into()))?;

        let token = auth_header
            .strip_prefix("Bearer ")
            .ok_or((StatusCode::UNAUTHORIZED, "Unauthorized".into()))?;

        let state = parts
            .extensions
            .get::<AppState>()
            .ok_or((StatusCode::INTERNAL_SERVER_ERROR, "Something went wrong".into()))?;

        let decoded = decode::<Claims>(
            token,
            &DecodingKey::from_secret(state.jwt_secret.as_bytes()),
            &Validation::default(),
        )
            .map_err(|_| (StatusCode::UNAUTHORIZED, "Unauthorized".into()))?;

        Ok(AuthUser {
            user_id: decoded.claims.sub,
            device_id: decoded.claims.device,
        })
    }
}

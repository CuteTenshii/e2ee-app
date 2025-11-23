mod routes;

use axum::{Router, routing::get, Extension};
use diesel::{PgConnection, r2d2::{self, ConnectionManager}};
use dotenvy::dotenv;
use tower_http::trace::TraceLayer;
use tracing_subscriber;

type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;

#[derive(Clone)]
pub struct AppState {
    pub db: DbPool,
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
    let state = AppState { db: pool };
    let app = Router::new()
        .route("/messages", get(routes::messages::get_messages))
        .layer(TraceLayer::new_for_http())
        .layer(Extension(state))
    ;

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("[Server] Started on http://0.0.0.0:3000");
    axum::serve(listener, app).await.unwrap();
}

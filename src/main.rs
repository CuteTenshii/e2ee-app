mod routes;

use axum::{Router, routing::get, Extension};
use diesel::{PgConnection, r2d2::{self, ConnectionManager}};
use dotenvy::dotenv;

type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;

#[derive(Clone)]
pub struct AppState {
    pub db: DbPool,
}

fn establish_connection() -> DbPool {
    dotenv().ok();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    let pool = r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool.");

    pool
}

#[tokio::main]
async fn main() {
    let pool = establish_connection();
    let state = AppState { db: pool };
    let app = Router::new()
        .route("/messages", get(routes::messages::get_messages))
        .layer(Extension(state))
    ;

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

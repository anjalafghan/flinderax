use axum::{
    routing::{get},
    Router,
};
use sqlx::{SqlitePool};
use std::env;

mod models;

mod handlers;
mod routes;

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    dotenvy::dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL missing");
    let pool = SqlitePool::connect(&database_url).await?;
    sqlx::migrate!().run(&pool).await?;
    
    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .nest("/user", routes::user::routes(pool.clone()));
    
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .unwrap();
    
    axum::serve(listener, app).await.unwrap();
    Ok(())
}


use std::time::Duration;

use axum::{Router, routing::get};
use tokio::time::Interval;
use tokio::time::interval;
use tower_http::cors::{Any, CorsLayer};

mod handlers;
use handlers::sysinfo;
use handlers::syslog;
use handlers::write_logs;

#[tokio::main]
async fn main() {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/api/", get(sysinfo))
        .route("/api/status", get(sysinfo))
        .route("/api/log", get(syslog))
        .layer(cors);

    let mut interval = interval(Duration::from_secs(60));

    tokio::spawn(async move {
        loop {
            interval.tick().await;
            write_logs().await;
        }
    });

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Server started successfully at 0.0.0.0:3000");
    axum::serve(listener, app).await.unwrap();
}



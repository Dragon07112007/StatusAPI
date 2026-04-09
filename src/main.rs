use std::time::Duration;

use axum::{Router, routing::get};
use tokio::time::{Instant, interval_at};
use tower_http::cors::{Any, CorsLayer};

mod system_info;
mod handlers;
use handlers::sysinfo;
use handlers::syslog;
use handlers::write_logs;
use system_info::{MinuteRingBuffer, collect_system_sample};

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

    let start = Instant::now() + Duration::from_secs(1);
    let mut interval = interval_at(start, Duration::from_secs(1));

    tokio::spawn(async move {
        let mut buffer = MinuteRingBuffer::new(60);
        let mut samples_seen: u64 = 0;

        loop {
            interval.tick().await;

            let sample = tokio::task::spawn_blocking(collect_system_sample)
                .await
                .expect("system sampling task panicked");

            buffer.push(sample);
            samples_seen += 1;

            if samples_seen % 60 == 0 {
                let average = buffer
                    .average()
                    .expect("ring buffer should contain a full minute of samples");
                write_logs(average).await;
            }
        }
    });

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Server started successfully at 0.0.0.0:3000");
    axum::serve(listener, app).await.unwrap();
}

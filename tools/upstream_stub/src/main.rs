use axum::{routing::post, Router};
use axum::response::IntoResponse;
use clap::Parser;

#[derive(Parser, Debug)]
struct Args {
    #[arg(long, default_value = "127.0.0.1:8000")]
    bind: String,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let app = Router::new().route("/v1/chat/completions", post(handler))
        .route("/health", post(|| async { "ok" }));

    let listener = tokio::net::TcpListener::bind(args.bind).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn handler(body: String) -> impl IntoResponse {
    // echo-like behavior (good for proxy verification)
    let response = serde_json::json!({
        "id": "stub",
        "object": "chat.completion",
        "choices": [{ "index": 0, "message": { "role": "assistant", "content": format!("echo: {body}") } }]
    });
    axum::Json(response)
}

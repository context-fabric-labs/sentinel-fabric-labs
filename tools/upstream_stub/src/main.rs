use axum::{
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::net::SocketAddr;

struct CancelLog {
    name: &'static str,
    completed: bool,
}
impl CancelLog {
    fn new(name: &'static str) -> Self {
        Self {
            name,
            completed: false,
        }
    }
}
impl Drop for CancelLog {
    fn drop(&mut self) {
        if !self.completed {
            eprintln!("upstream_stub: request canceled ({})", self.name);
        }
    }
}
#[derive(Debug, Deserialize)]
struct ChatReq {
    model: Option<String>,
    messages: Option<Vec<Message>>,
    max_tokens: Option<u32>,
    stream: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Debug, Serialize)]
struct ChatResp {
    id: String,
    object: String,
    choices: Vec<Choice>,
}

#[derive(Debug, Serialize)]
struct Choice {
    index: u32,
    message: AssistantMessage,
    finish_reason: String,
}

#[derive(Debug, Serialize)]
struct AssistantMessage {
    role: String,
    content: String,
}

async fn health() -> Json<serde_json::Value> {
    Json(json!({ "ok": true, "service": "upstream_stub", "version": "0.0.0" }))
}

async fn chat(Json(req): Json<ChatReq>) -> Json<ChatResp> {
    let mut cancel = CancelLog::new("chat");
    // Validate / acknowledge stream flag (non-stream MVP)
    if req.stream.unwrap_or(false) {
        // You can either reject or just ignore but "read" it.
        // Here we reject to keep semantics honest.
        cancel.completed = true;
        return Json(ChatResp {
            id: "stub-err".into(),
            object: "chat.completion".into(),
            choices: vec![Choice {
                index: 0,
                message: AssistantMessage {
                    role: "assistant".into(),
                    content: "upstream_stub: stream=true not supported in stub (use stream=false)"
                        .into(),
                },
                finish_reason: "stop".into(),
            }],
        });
    }

    let last = req.messages.as_ref().and_then(|m| m.last());

    // Read role to avoid dead-code + keep request shape realistic
    let last_role = last.map(|m| m.role.as_str()).unwrap_or("none");
    let prompt_preview = last
        .map(|m| m.content.chars().take(80).collect::<String>())
        .unwrap_or_else(|| "<no-messages>".to_string());

    let content = format!(
        "stub response (model={:?}, max_tokens={:?}, last_role={}) :: {}",
        req.model, req.max_tokens, last_role, prompt_preview
    );

    cancel.completed = true;
    Json(ChatResp {
        id: "stub-1".into(),
        object: "chat.completion".into(),
        choices: vec![Choice {
            index: 0,
            message: AssistantMessage {
                role: "assistant".into(),
                content,
            },
            finish_reason: "stop".into(),
        }],
    })
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/health", get(health))
        .route("/v1/chat/completions", post(chat));

    let addr: SocketAddr = "0.0.0.0:4000".parse().unwrap();
    println!("upstream_stub listening on http://{addr}");

    axum::serve(tokio::net::TcpListener::bind(addr).await.unwrap(), app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
}

async fn shutdown_signal() {
    let _ = tokio::signal::ctrl_c().await;
}

//! S1.3 – request-id propagation, per-route metrics middleware, TraceLayer helpers.

use axum::{
    body::Body,
    http::{Request, StatusCode},
    middleware::Next,
    response::IntoResponse,
};
use std::time::Instant;
use uuid::Uuid;

/// Extension type carrying the request-id through the middleware stack.
#[derive(Clone, Debug)]
pub struct RequestId(pub String);

static X_REQUEST_ID: axum::http::header::HeaderName =
    axum::http::header::HeaderName::from_static("x-request-id");

/// Accept `x-request-id` if present, otherwise generate UUIDv4.
/// Stores it in request extensions, echoes it on the response.
pub async fn request_id_layer(mut req: Request<Body>, next: Next) -> impl IntoResponse {
    let req_id = req
        .headers()
        .get(&X_REQUEST_ID)
        .and_then(|hv| hv.to_str().ok())
        .map(|s| s.to_owned())
        .unwrap_or_else(|| Uuid::new_v4().to_string());

    req.extensions_mut().insert(RequestId(req_id.clone()));

    if let Ok(hv) = axum::http::HeaderValue::from_str(&req_id) {
        req.headers_mut().insert(X_REQUEST_ID.clone(), hv);
    }

    let mut resp = next.run(req).await.into_response();

    if let Ok(hv) = axum::http::HeaderValue::from_str(&req_id) {
        resp.headers_mut().insert(X_REQUEST_ID.clone(), hv);
    }
    resp
}

/// Per-route counters + duration histograms.
/// sentinel_http_requests_total{route,method,status}
/// sentinel_http_request_duration_seconds{route,method,status}
/// sentinel_chat_request_duration_seconds{status}  (only /v1/chat/completions)
pub async fn http_metrics_layer(req: Request<Body>, next: Next) -> impl IntoResponse {
    let route = req.uri().path().to_owned();
    let method = req.method().to_string();
    let start = Instant::now();

    let resp = next.run(req).await.into_response();
    let status = resp.status().as_u16().to_string();
    let secs = start.elapsed().as_secs_f64();

    metrics::counter!(
        "sentinel_http_requests_total",
        "route" => route.clone(),
        "method" => method.clone(),
        "status" => status.clone()
    )
    .increment(1);

    metrics::histogram!(
        "sentinel_http_request_duration_seconds",
        "route" => route.clone(),
        "method" => method.clone(),
        "status" => status.clone()
    )
    .record(secs);

    if route == "/v1/chat/completions" {
        metrics::histogram!(
            "sentinel_chat_request_duration_seconds",
            "status" => status
        )
        .record(secs);
    }

    resp
}

/// on_response callback for TraceLayer – logs status + latency and bumps response counters.
#[allow(dead_code)]
pub fn on_response_log(status: StatusCode, latency: std::time::Duration, span: &tracing::Span) {
    let latency_ms = latency.as_secs_f64() * 1000.0;
    metrics::counter!(
        "sentinel_responses_total",
        "status" => status.as_u16().to_string()
    )
    .increment(1);

    if status == StatusCode::PAYLOAD_TOO_LARGE {
        metrics::counter!("sentinel_payload_too_large_total").increment(1);
    }
    if status == StatusCode::GATEWAY_TIMEOUT {
        metrics::counter!("sentinel_timeouts_total").increment(1);
    }

    tracing::info!(
        parent: span,
        status = status.as_u16(),
        latency_ms,
        "request_complete"
    );
}

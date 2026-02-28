use axum::http::header;
use std::{net::SocketAddr, sync::Arc, time::Duration};
use tower_http::limit::ResponseBody;

use axum::{
    body::Body,
    extract::State,
    http::{Request, StatusCode, Uri},
    response::IntoResponse,
    routing::{any, get},
    Router,
};
use clap::Parser;
use http_body_util::BodyExt;
use hyper::header::HOST;
use hyper_util::{client::legacy::Client, rt::TokioExecutor};
use metrics_exporter_prometheus::{PrometheusBuilder, PrometheusHandle};
use std::error::Error as StdError;
use tokio::sync::{OwnedSemaphorePermit, Semaphore};
use tower::ServiceBuilder;
use tower_http::{limit::RequestBodyLimitLayer, timeout::TimeoutLayer, trace::TraceLayer};
use tracing::{info, Level};
use tracing_subscriber::EnvFilter;

mod admission;
mod breaker;
mod observability;
use crate::admission::{AdmissionConfig, AdmissionController};
use crate::breaker::{run_health_probe, Breaker, BreakerConfig, HalfOpenGuard};
use crate::observability::{http_metrics_layer, request_id_layer, RequestId};

#[derive(Parser, Debug)]
struct Args {
    /// Bind address, e.g. 127.0.0.1:8080
    #[arg(long, default_value = "127.0.0.1:8080")]
    bind: String,

    /// Upstream base URL, e.g. http://127.0.0.1:4000
    #[arg(long, default_value = "http://127.0.0.1:4000")]
    upstream: String,

    /// Max inflight requests (overload => 429)
    #[arg(long, default_value_t = 64)]
    max_inflight: usize,

    /// Max request body bytes
    #[arg(long, default_value_t = 1_000_000)]
    max_body_bytes: usize,

    /// Request timeout in milliseconds
    #[arg(long, default_value_t = 30_000)]
    timeout_ms: u64,
}

#[derive(Clone)]
struct AppState {
    upstream_base: String,
    client: Client<hyper_util::client::legacy::connect::HttpConnector, Body>,
    inflight: Arc<Semaphore>,
    prom: PrometheusHandle,
    admission: Arc<AdmissionController>,
    breaker: Arc<Breaker>,
}

struct InflightGuard {
    _permit: OwnedSemaphorePermit,
}
impl Drop for InflightGuard {
    fn drop(&mut self) {
        metrics::gauge!("sentinel_inflight").decrement(1.0);
    }
}

// RequestId type and middlewares moved to observability.rs

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive(Level::INFO.into()))
        .init();

    let args = Args::parse();

    // Allow env override without clap env feature:
    let upstream_base = std::env::var("UPSTREAM_URL").unwrap_or_else(|_| args.upstream.clone());
    let upstream_base = upstream_base.trim_end_matches('/').to_string();

    let addr: SocketAddr = args.bind.parse().expect("invalid --bind");

    let prom = PrometheusBuilder::new()
        .install_recorder()
        .expect("install Prometheus recorder");

    let client = Client::builder(TokioExecutor::new()).build_http();

    let admission_cfg = AdmissionConfig::default();
    let admission = Arc::new(AdmissionController::new(admission_cfg));
    let breaker_cfg = BreakerConfig::default();
    let breaker = Arc::new(Breaker::new(&breaker_cfg));

    let state = AppState {
        upstream_base: upstream_base.clone(),
        client,
        inflight: Arc::new(Semaphore::new(args.max_inflight)),
        prom,
        admission: admission.clone(),
        breaker: breaker.clone(),
    };

    info!(
        "sentinel listening on {} upstream={} max_inflight={} max_body_bytes={} timeout_ms={}",
        addr, upstream_base, args.max_inflight, args.max_body_bytes, args.timeout_ms
    );

    let trace = TraceLayer::new_for_http()
        .make_span_with(|req: &Request<_>| {
            let rid = req
                .extensions()
                .get::<RequestId>()
                .map(|r| r.0.clone())
                .unwrap_or_else(|| "".to_string());
            let method = req.method().to_string();
            let route = req.uri().path().to_string();
            let uri = req.uri().to_string();
            tracing::info_span!(
                "http_request",
                request_id = %rid,
                method = %method,
                route = %route,
                uri = %uri
            )
        })
        .on_response(
            |response: &axum::http::Response<ResponseBody<Body>>,
             latency: Duration,
             span: &tracing::Span| {
                let code = response.status();

                metrics::counter!(
                    "sentinel_responses_total",
                    "status" => code.as_u16().to_string()
                )
                .increment(1);

                if code == StatusCode::PAYLOAD_TOO_LARGE {
                    metrics::counter!("sentinel_payload_too_large_total").increment(1);
                }
                if code == StatusCode::GATEWAY_TIMEOUT {
                    metrics::counter!("sentinel_timeouts_total").increment(1);
                }

                let latency_ms = latency.as_secs_f64() * 1000.0;
                tracing::info!(parent: span, status = %code.as_u16(), latency_ms = latency_ms, "request_complete");
            },
        );

    let middleware = ServiceBuilder::new()
        .layer(trace)
        .layer(RequestBodyLimitLayer::new(args.max_body_bytes))
        // tower-http 0.6.7+ prefers with_status_code
        .layer(TimeoutLayer::with_status_code(
            StatusCode::GATEWAY_TIMEOUT,
            Duration::from_millis(args.timeout_ms),
        ));

    let app = Router::new()
        .route("/health", get(|| async { "ok" }))
        .route("/metrics", get(metrics_handler))
        .route("/v1/chat/completions", any(proxy_handler))
        .with_state(state)
        .layer(middleware)
        // Apply per-route metrics and request-id as outer middleware
        .layer(axum::middleware::from_fn(http_metrics_layer))
        .layer(axum::middleware::from_fn(request_id_layer));

    // Spawn background health probe for breaker
    {
        let b = breaker.clone();
        let base = upstream_base.clone();
        tokio::spawn(async move {
            run_health_probe(b, BreakerConfig::default(), base).await;
        });
    }

    axum::serve(tokio::net::TcpListener::bind(addr).await.unwrap(), app)
        .await
        .unwrap();
}

async fn metrics_handler(State(st): State<AppState>) -> impl IntoResponse {
    st.prom.render()
}

async fn proxy_handler(State(st): State<AppState>, req: Request<Body>) -> impl IntoResponse {
    // Inflight boundedness (fast reject)
    let permit = match st.inflight.clone().try_acquire_owned() {
        Ok(p) => p,
        Err(_) => {
            metrics::counter!("sentinel_rejected_overload_total").increment(1);
            let mut resp = (StatusCode::TOO_MANY_REQUESTS, "overloaded").into_response();
            resp.headers_mut()
                .insert(header::RETRY_AFTER, "1".parse().unwrap());
            return resp;
        }
    };
    metrics::gauge!("sentinel_inflight").increment(1.0);
    let _guard = InflightGuard { _permit: permit };

    metrics::counter!("sentinel_requests_total").increment(1);
    let start = std::time::Instant::now();

    // Build upstream URI = upstream_base + path_and_query
    let pq = req
        .uri()
        .path_and_query()
        .map(|v| v.as_str())
        .unwrap_or("/");
    let upstream_uri: Uri = match format!("{}{}", st.upstream_base, pq).parse() {
        Ok(u) => u,
        Err(e) => {
            metrics::counter!("sentinel_bad_upstream_uri_total").increment(1);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("bad upstream uri: {e}"),
            )
                .into_response();
        }
    };

    // Grab request id for propagation
    let req_id = req.extensions().get::<RequestId>().map(|r| r.0.clone());

    let (mut parts, body) = req.into_parts();

    // Non-stream MVP: read body fully
    let bytes = match body.collect().await {
        Ok(collected) => collected.to_bytes(),
        Err(e) => {
            metrics::counter!("sentinel_bad_request_body_total").increment(1);
            return (StatusCode::BAD_REQUEST, format!("bad request body: {e}")).into_response();
        }
    };

    // S1.6: Circuit breaker fast-fail for chat endpoint
    let mut _half_open_guard: Option<HalfOpenGuard> = None;
    if parts.uri.path() == "/v1/chat/completions" {
        match st.breaker.allow(&BreakerConfig::default()).await {
            Ok(guard_opt) => {
                _half_open_guard = guard_opt;
            }
            Err(()) => {
                metrics::counter!("sentinel_breaker_fast_fail_total").increment(1);
                return (StatusCode::SERVICE_UNAVAILABLE, "upstream unavailable").into_response();
            }
        }
    }

    // Set HOST to upstream authority
    parts.headers.remove(HOST);
    if let Some(auth) = upstream_uri.authority() {
        if let Ok(hv) = auth.as_str().parse() {
            parts.headers.insert(HOST, hv);
        }
    }

    // Propagate request-id to upstream
    if let Some(rid) = req_id.clone() {
        if let Ok(hv) = axum::http::HeaderValue::from_str(&rid) {
            parts.headers.insert(
                axum::http::header::HeaderName::from_static("x-request-id"),
                hv,
            );
        }
    }

    // S1.5 admission + degrade for chat
    let mut body_bytes = bytes.to_vec();
    if upstream_uri.path() == "/v1/chat/completions" {
        let tokens = admission::estimate_tokens(&body_bytes);
        metrics::counter!("sentinel_token_estimate_total").increment(1);
        metrics::histogram!("sentinel_token_estimate_hist").record(tokens as f64);

        match st.admission.admit().await {
            Ok(guard) => {
                let degrade_tokens = st.admission.config.degrade_max_tokens;
                let mut degraded = false;
                if admission::AdmissionController::should_degrade(&guard) {
                    body_bytes = admission::clamp_max_tokens(&body_bytes, degrade_tokens);
                    metrics::counter!("sentinel_degraded_total", "reason" => "queueing")
                        .increment(1);
                    degraded = true;
                } else if tokens > degrade_tokens {
                    body_bytes = admission::clamp_max_tokens(&body_bytes, degrade_tokens);
                    metrics::counter!("sentinel_degraded_total", "reason" => "tokens").increment(1);
                    degraded = true;
                }
                if degraded {
                    metrics::counter!("sentinel_admission_total", "decision" => "degrade", "reason" => "policy").increment(1);
                }
                // hold guard for lifetime of request
                let _admission_guard = guard;
            }
            Err(_) => {
                let mut resp = (StatusCode::TOO_MANY_REQUESTS, "overloaded").into_response();
                resp.headers_mut()
                    .insert(header::RETRY_AFTER, "1".parse().unwrap());
                return resp;
            }
        }
    }

    let mut upstream_req = Request::from_parts(parts, Body::from(body_bytes));
    *upstream_req.uri_mut() = upstream_uri.clone();

    let resp = match st.client.request(upstream_req).await {
        Ok(r) => {
            metrics::counter!(
                "sentinel_upstream_responses_total",
                "status" => r.status().as_u16().to_string()
            )
            .increment(1);
            if upstream_uri.path() == "/v1/chat/completions"
                && st.breaker.state() == breaker::State::HalfOpen
            {
                st.breaker.on_success();
            }
            r
        }
        Err(e) => {
            let kind = classify_upstream_error(&e);
            metrics::counter!("sentinel_upstream_errors_total", "kind" => kind).increment(1);
            if upstream_uri.path() == "/v1/chat/completions" {
                if st.breaker.state() == breaker::State::HalfOpen {
                    st.breaker.open();
                } else {
                    st.breaker.on_failure(&BreakerConfig::default());
                }
            }
            return (StatusCode::BAD_GATEWAY, format!("upstream error: {e}")).into_response();
        }
    };

    let latency_ms = start.elapsed().as_secs_f64() * 1000.0;
    metrics::histogram!("sentinel_request_latency_ms").record(latency_ms);

    resp.into_response()
}

fn classify_upstream_error(e: &hyper_util::client::legacy::Error) -> &'static str {
    // Walk error chain and classify by common substrings
    let mut cur: Option<&(dyn StdError + 'static)> = Some(e);
    while let Some(err) = cur {
        let msg = format!("{err}").to_lowercase();
        if msg.contains("dns") || msg.contains("no such host") || msg.contains("resolver") {
            return "dns";
        }
        if msg.contains("timed out") || msg.contains("timeout") {
            return "timeout";
        }
        if msg.contains("connect")
            || msg.contains("connection refused")
            || msg.contains("connection reset")
            || msg.contains("broken pipe")
        {
            return "connect";
        }
        cur = err.source();
    }
    "other"
}

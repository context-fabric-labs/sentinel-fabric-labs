use std::{net::SocketAddr, sync::Arc};

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
use hyper_util::client::legacy::Client;
use hyper_util::rt::TokioExecutor;
use metrics_exporter_prometheus::PrometheusBuilder;
use tokio::sync::Semaphore;
use tower::ServiceBuilder;
use tower_http::{limit::RequestBodyLimitLayer, trace::TraceLayer};
use tracing::{info, Level};
use tracing_subscriber::EnvFilter;

#[derive(Parser, Debug)]
struct Args {
    /// Bind address, e.g. 127.0.0.1:8080
    #[arg(long, default_value = "127.0.0.1:8080")]
    bind: String,

    /// Max inflight requests
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
    metrics: Arc<metrics_exporter_prometheus::PrometheusHandle>,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive(Level::INFO.into()))
        .init();

    let args = Args::parse();

    let upstream_base =
        std::env::var("UPSTREAM_URL").unwrap_or_else(|_| "http://127.0.0.1:8000".to_string());

    let recorder = PrometheusBuilder::new().install_recorder().expect("prom recorder");
    let handle = Arc::new(recorder);

    let client = Client::builder(TokioExecutor::new()).build_http();

    let state = AppState {
        upstream_base,
        client,
        inflight: Arc::new(Semaphore::new(args.max_inflight)),
        metrics: handle,
    };

    let middleware = ServiceBuilder::new()
        .layer(TraceLayer::new_for_http())
        .layer(RequestBodyLimitLayer::new(args.max_body_bytes));


    let app = Router::new()
        .route("/health", get(|| async { "ok" }))
        .route("/metrics", get(metrics_handler))
        .route("/v1/chat/completions", any(proxy_handler))
        .with_state(state)
        .layer(middleware);

    let addr: SocketAddr = args.bind.parse().expect("bind parse");
    info!("sentinel listening on {addr} upstream={}", std::env::var("UPSTREAM_URL").ok().unwrap_or_default());
    axum::serve(tokio::net::TcpListener::bind(addr).await.unwrap(), app)
        .await
        .unwrap();
}

async fn metrics_handler(State(st): State<AppState>) -> impl IntoResponse {
    st.metrics.render()
}

async fn proxy_handler(State(st): State<AppState>, req: Request<Body>) -> impl IntoResponse {
    // Boundedness: semaphore (inflight cap)
    let _permit = match st.inflight.clone().try_acquire_owned() {
        Ok(p) => p,
        Err(_) => {
            metrics::counter!("sentinel_rejected_overload").increment(1);

            return (StatusCode::TOO_MANY_REQUESTS, "overloaded").into_response();
        }
    };

    metrics::counter!("sentinel_requests_total").increment(1);
    let start = std::time::Instant::now();

    // Build upstream URI: upstream_base + req.uri().path_and_query()
    let pq = req
        .uri()
        .path_and_query()
        .map(|v| v.as_str())
        .unwrap_or("/");
    let upstream_uri: Uri = format!("{}{}", st.upstream_base, pq).parse().unwrap();

    let (mut parts, body) = req.into_parts();

    // Set HOST to upstream host (helps some upstreams)
    parts.headers.remove(HOST);
    let mut upstream_req = Request::from_parts(parts, Body::from(body.collect().await.unwrap().to_bytes()));
    *upstream_req.uri_mut() = upstream_uri;

    // Forward
    let resp = match st.client.request(upstream_req).await {
        Ok(r) => r,
        Err(e) => {
            metrics::counter!("sentinel_upstream_errors_total").increment(1);
            return (StatusCode::BAD_GATEWAY, format!("upstream error: {e}")).into_response();
        }
    };

    let latency_ms = start.elapsed().as_millis() as f64;
    metrics::histogram!("sentinel_request_latency_ms").record(latency_ms);

    resp.into_response()
}

/*
Examples:
- Health GET:    cargo run -p bench -- run --scenario health --url http://127.0.0.1:8080/health
- Chat POST:     cargo run -p bench -- run --scenario chat --url http://127.0.0.1:8080/v1/chat/completions
- Chat with JSON: cargo run -p bench -- run --scenario chat --url http://127.0.0.1:8080/v1/chat/completions --body-json '{"model":"stub","messages":[{"role":"user","content":"hi"}],"max_tokens":16,"stream":false}'
*/

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use clap::{Parser, Subcommand};
use hdrhistogram::Histogram;
use reqwest::Method;
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    path::PathBuf,
    process::Command,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    time::{Duration, Instant},
};
use tokio::sync::{mpsc, Semaphore};

#[derive(Parser, Debug)]
#[command(name = "bench")]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand, Debug)]
enum Cmd {
    /// Run a load test and write JSON result
    Run {
        #[arg(long, default_value = "health")]
        scenario: String,

        #[arg(long)]
        url: String,

        #[arg(long, default_value_t = 32)]
        concurrency: usize,

        #[arg(long, default_value_t = 2000)]
        requests: usize,

        #[arg(long, default_value_t = 5)]
        timeout_s: u64,

        #[arg(long, default_value_t = 200)]
        warmup_requests: usize,

        /// Optional body file for chat scenarios (POST JSON)
        #[arg(long)]
        body_file: Option<PathBuf>,

        /// Optional raw JSON string for chat scenarios (POST JSON)
        #[arg(long)]
        body_json: Option<String>,

        /// Optional output path. If omitted, writes to bench/results/<scenario>-<ts>.json
        #[arg(long)]
        out: Option<PathBuf>,
    },

    /// Pretty-print a JSON result
    Report { file: PathBuf },
}

#[derive(Debug, Serialize, Deserialize)]
struct BenchResult {
    scenario: String,
    url: String,
    concurrency: usize,
    requests: usize,
    warmup_requests: usize,
    timeout_s: u64,
    timestamp_utc: DateTime<Utc>,

    wall_time_ms: u64,
    rps: f64,

    errors: u64,
    status_counts: BTreeMap<String, u64>,

    latency_us: LatencyStats,

    env: EnvInfo,
}

#[derive(Debug, Serialize, Deserialize)]
struct LatencyStats {
    p50: u64,
    p95: u64,
    p99: u64,
    max: u64,
}

#[derive(Debug, Serialize, Deserialize)]
struct EnvInfo {
    os: String,
    arch: String,
    cpu_count: usize,
    git_commit: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.cmd {
        Cmd::Run {
            scenario,
            url,
            concurrency,
            requests,
            timeout_s,
            warmup_requests,
            body_file,
            body_json,
            out,
        } => {
            let res = run_bench(
                scenario,
                url,
                concurrency,
                requests,
                warmup_requests,
                Duration::from_secs(timeout_s),
                BenchBody { file: body_file, json: body_json },
            )
            .await?;

            let out_path = out.unwrap_or_else(|| default_out_path(&res.scenario));
            if let Some(parent) = out_path.parent() {
                std::fs::create_dir_all(parent)
                    .with_context(|| format!("create dir {:?}", parent))?;
            }
            std::fs::write(&out_path, serde_json::to_vec_pretty(&res)?)?;
            println!("wrote {:?}", out_path);
            print_summary(&res);
        }
        Cmd::Report { file } => {
            let bytes = std::fs::read(&file)?;
            let res: BenchResult = serde_json::from_slice(&bytes)?;
            print_summary(&res);
            println!("{}", serde_json::to_string_pretty(&res)?);
        }
    }

    Ok(())
}

fn default_out_path(scenario: &str) -> PathBuf {
    let ts = Utc::now().format("%Y%m%d-%H%M%S").to_string();
    PathBuf::from(format!("bench/results/{}-{}.json", scenario, ts))
}

fn env_info() -> EnvInfo {
    let git_commit = Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()
        .and_then(|o| {
            if o.status.success() {
                Some(String::from_utf8_lossy(&o.stdout).trim().to_string())
            } else {
                None
            }
        });

    EnvInfo {
        os: std::env::consts::OS.to_string(),
        arch: std::env::consts::ARCH.to_string(),
        cpu_count: num_cpus::get(),
        git_commit,
    }
}

struct BenchBody {
    file: Option<PathBuf>,
    json: Option<String>,
}

async fn run_bench(
    scenario: String,
    url: String,
    concurrency: usize,
    requests: usize,
    warmup_requests: usize,
    timeout: Duration,
    body: BenchBody,
) -> Result<BenchResult> {
    let body_file = body.file;
    let body_json = body.json;
    let client = reqwest::Client::builder().timeout(timeout).build()?;

    // Build request spec based on scenario
    let is_chat = scenario.to_lowercase().starts_with("chat");
    let spec = if is_chat {
        // Determine JSON body precedence: body_json > body_file > default
        let body = {
            if let Some(s) = body_json.clone() {
                Some(s)
            } else if let Some(p) = body_file.clone() {
                Some(std::fs::read_to_string(p)?)
            } else {
                Some("{\"model\":\"stub\",\"messages\":[{\"role\":\"user\",\"content\":\"hello\"}],\"max_tokens\":16,\"stream\":false}".to_string())
            }
        };
        RequestSpec {
            method: Method::POST,
            body,
            is_json: true,
        }
    } else {
        RequestSpec {
            method: Method::GET,
            body: None,
            is_json: false,
        }
    };

    // Warmup (ignore measurements)
    run_load(&client, &url, concurrency, warmup_requests, true, &spec).await?;

    // Measured run
    let t0 = Instant::now();
    let (hist, errors, status_counts, completed) =
        run_load(&client, &url, concurrency, requests, false, &spec).await?;
    let wall = t0.elapsed();

    let wall_ms = wall.as_millis() as u64;
    let rps = if wall.as_secs_f64() > 0.0 {
        completed as f64 / wall.as_secs_f64()
    } else {
        0.0
    };

    let p50 = hist.value_at_quantile(0.50);
    let p95 = hist.value_at_quantile(0.95);
    let p99 = hist.value_at_quantile(0.99);
    let max = hist.max();

    Ok(BenchResult {
        scenario,
        url,
        concurrency,
        requests,
        warmup_requests,
        timeout_s: timeout.as_secs(),
        timestamp_utc: Utc::now(),
        wall_time_ms: wall_ms,
        rps,
        errors,
        status_counts,
        latency_us: LatencyStats { p50, p95, p99, max },
        env: env_info(),
    })
}

async fn run_load(
    client: &reqwest::Client,
    url: &str,
    concurrency: usize,
    requests: usize,
    warmup: bool,
    spec: &RequestSpec,
) -> Result<(Histogram<u64>, u64, BTreeMap<String, u64>, usize)> {
    let sem = Arc::new(Semaphore::new(concurrency));
    let (tx, mut rx) = mpsc::unbounded_channel::<(u16, u64, bool)>();
    let in_flight = Arc::new(AtomicUsize::new(0));

    for _ in 0..requests {
        let permit = sem.clone().acquire_owned().await?;
        let tx = tx.clone();
        let client = client.clone();
        let url = url.to_string();
        let in_flight = in_flight.clone();
        let spec = spec.clone();

        in_flight.fetch_add(1, Ordering::Relaxed);
        tokio::spawn(async move {
            let _permit = permit;
            let start = Instant::now();
            let mut req = client.request(spec.method.clone(), url);
            if let Some(body) = &spec.body {
                req = req.body(body.clone());
            }
            if spec.is_json {
                req = req.header("content-type", "application/json");
            }
            let ok = match req.send().await {
                Ok(resp) => {
                    let status = resp.status().as_u16();
                    let _ = resp.bytes().await; // drain
                    let us = start.elapsed().as_micros() as u64;
                    let _ = tx.send((status, us, true));
                    true
                }
                Err(_) => {
                    let us = start.elapsed().as_micros() as u64;
                    let _ = tx.send((0, us, false));
                    false
                }
            };
            let _ = ok;
            in_flight.fetch_sub(1, Ordering::Relaxed);
        });
    }
    drop(tx);

    let mut hist = Histogram::<u64>::new_with_max(60_000_000, 3)?; // up to 60s in us
    let mut errors: u64 = 0;
    let mut status_counts: BTreeMap<String, u64> = BTreeMap::new();
    let mut completed: usize = 0;

    while let Some((status, us, ok)) = rx.recv().await {
        completed += 1;
        if !warmup {
            let _ = hist.record(us);
        }
        if !ok || status >= 500 || status == 0 {
            errors += 1;
        }
        let key = if status == 0 {
            "ERR".to_string()
        } else {
            status.to_string()
        };
        *status_counts.entry(key).or_insert(0) += 1;
    }

    Ok((hist, errors, status_counts, completed))
}

#[derive(Clone, Debug)]
struct RequestSpec {
    method: Method,
    body: Option<String>,
    is_json: bool,
}

fn print_summary(res: &BenchResult) {
    println!(
        "scenario={} conc={} req={} rps={:.1} errors={} p50={}us p95={}us p99={}us max={}us wall={}ms",
        res.scenario,
        res.concurrency,
        res.requests,
        res.rps,
        res.errors,
        res.latency_us.p50,
        res.latency_us.p95,
        res.latency_us.p99,
        res.latency_us.max,
        res.wall_time_ms
    );
}

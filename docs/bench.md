## Validate S1.3 (metrics + tracing)

- Start upstream stub: `cargo run -p upstream_stub`
- Start sentinel: `cargo run -p sentinel -- --bind 127.0.0.1:8080`
- Health: `curl -i http://127.0.0.1:8080/health`
- Metrics: `curl -s http://127.0.0.1:8080/metrics | grep sentinel_`
- Chat (POST):
	`curl -i -H 'content-type: application/json' \
		-d '{"model":"stub","messages":[{"role":"user","content":"hello"}],"max_tokens":16,"stream":false}' \
		http://127.0.0.1:8080/v1/chat/completions`

## Bench: health baseline

```
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace

cargo run -q -p bench -- run --scenario health --url http://127.0.0.1:8080/health --concurrency 32 --requests 2000 --out bench/results/cand.json
./scripts/regress.py --baseline bench/baselines/local_health.json --candidate bench/results/cand.json --budget_p99 0.10
```

## Bench: chat scenario + sweep

Run a single chat run (defaults provided):

```
cargo run -q -p bench -- run --scenario chat --url http://127.0.0.1:8080/v1/chat/completions
```

Sweep common concurrencies and store results in `bench/results/`:

```
./scripts/sweep_chat.sh http://127.0.0.1:8080/v1/chat/completions
```

Baseline suggestion (commit to repo when stable):

- `bench/baselines/local_chat.json`

## Breaker test (S1.6)

1. Start sentinel and upstream.
2. Kill upstream; send a few chat requests: expect fast 503 and `sentinel_breaker_fast_fail_total` increases.
3. Restart upstream; breaker should recover (HALF_OPEN then CLOSED).

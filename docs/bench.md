cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace


cargo run -q -p bench -- run --scenario health --url http://127.0.0.1:8080/health --concurrency 32 --requests 2000 --out bench/results/cand.json
./scripts/regress.py --baseline bench/baselines/local_health.json --candidate bench/results/cand.json --budget_p99 0.10

#!/usr/bin/env python3
import argparse, json, sys

def load(p):
    with open(p, "r") as f:
        return json.load(f)

def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--baseline", required=True)
    ap.add_argument("--candidate", required=True)
    ap.add_argument("--budget_p99", type=float, default=0.10, help="allowed increase fraction (0.10 = +10%)")
    args = ap.parse_args()

    b = load(args.baseline)
    c = load(args.candidate)

    bp99 = b["latency_us"]["p99"]
    cp99 = c["latency_us"]["p99"]

    limit = int(bp99 * (1.0 + args.budget_p99))

    print(f"baseline p99={bp99}us  candidate p99={cp99}us  limit={limit}us  budget={args.budget_p99*100:.1f}%")

    if c["errors"] != 0:
        print(f"FAIL: candidate errors={c['errors']}")
        sys.exit(2)

    if cp99 > limit:
        print("FAIL: p99 regression")
        sys.exit(3)

    print("OK")
    return 0

if __name__ == "__main__":
    raise SystemExit(main())

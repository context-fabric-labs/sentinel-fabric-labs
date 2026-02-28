#!/usr/bin/env python3
import sys

with open('bench/src/main.rs', 'r') as f:
    content = f.read()

print(f"File length: {len(content)} chars, {content.count(chr(10))} lines")

# Check what's there
if 'body_file: Option<PathBuf>' in content:
    print("Found old 8-arg signature")
if 'BenchBody' in content:
    print("Found BenchBody (already patched)")
if 'let mut spec' in content:
    print("Found let mut spec")
if 'let spec' in content:
    print("Found let spec")

# Fix 1: Replace old 8-arg run_bench with BenchBody version
old1 = 'async fn run_bench(\n    scenario: String,\n    url: String,\n    concurrency: usize,\n    requests: usize,\n    warmup_requests: usize,\n    timeout: Duration,\n    body_file: Option<PathBuf>,\n    body_json: Option<String>,\n) -> Result<BenchResult> {'
new1 = 'struct BenchBody {\n    file: Option<PathBuf>,\n    json: Option<String>,\n}\n\nasync fn run_bench(\n    scenario: String,\n    url: String,\n    concurrency: usize,\n    requests: usize,\n    warmup_requests: usize,\n    timeout: Duration,\n    body: BenchBody,\n) -> Result<BenchResult> {\n    let body_file = body.file;\n    let body_json = body.json;'

if old1 in content:
    content = content.replace(old1, new1, 1)
    print("OK: Fixed run_bench signature")
elif 'BenchBody' not in content:
    print("FAIL: Could not find old run_bench signature to replace")
    sys.exit(1)

# Fix 2: let mut spec -> let spec
old2 = '    let mut spec = if is_chat {'
new2 = '    let spec = if is_chat {'
if old2 in content:
    content = content.replace(old2, new2, 1)
    print("OK: Fixed let mut spec")
else:
    print("NOTE: let mut spec not found (may already be fixed)")

# Fix 3: Update call site
old3 = '                body_file,\n                body_json,'
new3 = '                BenchBody { file: body_file, json: body_json },'
if old3 in content:
    content = content.replace(old3, new3, 1)
    print("OK: Fixed call site")
elif 'BenchBody { file:' in content:
    print("NOTE: call site already fixed")
else:
    print("FAIL: Could not find call site")

with open('bench/src/main.rs', 'w') as f:
    f.write(content)
print("Written successfully")

#!/usr/bin/env python3

# Fix sentinel/src/breaker.rs: 
# 1. half_open_slots: Semaphore -> Arc<Semaphore>
# 2. State::X as f64 -> State::X as u8 as f64
with open('sentinel/src/breaker.rs', 'r') as f:
    content = f.read()

print(f"breaker.rs: {content.count(chr(10))} lines")

# Fix 1: struct field
old1 = '    half_open_slots: Semaphore,'
new1 = '    half_open_slots: Arc<Semaphore>,'
if old1 in content:
    content = content.replace(old1, new1, 1)
    print("OK: fixed half_open_slots field type")
elif 'half_open_slots: Arc<Semaphore>' in content:
    print("NOTE: already Arc<Semaphore>")
else:
    print("FAIL: half_open_slots field not found")

# Fix 2: Breaker::new initialization
old2 = '            half_open_slots: Semaphore::new(cfg.half_open_max_inflight),'
new2 = '            half_open_slots: Arc::new(Semaphore::new(cfg.half_open_max_inflight)),'
if old2 in content:
    content = content.replace(old2, new2, 1)
    print("OK: fixed Semaphore::new in Breaker::new")
elif 'Arc::new(Semaphore::new' in content:
    print("NOTE: already Arc::new(Semaphore::new")
else:
    print("FAIL: Semaphore::new initialization not found")

# Fix 3: State enum casts (as f64 -> as u8 as f64)
import re
old_casts = [
    ('State::Closed as f64', 'State::Closed as u8 as f64'),
    ('State::Open as f64', 'State::Open as u8 as f64'),
    ('State::HalfOpen as f64', 'State::HalfOpen as u8 as f64'),
]
for old, new in old_casts:
    count = content.count(old)
    if count > 0:
        content = content.replace(old, new)
        print(f"OK: replaced {count}x '{old}'")
    else:
        print(f"NOTE: '{old}' not found (may already be fixed)")

with open('sentinel/src/breaker.rs', 'w') as f:
    f.write(content)
print("breaker.rs written")

# Also check/fix sentinel/src/main.rs for upstream_uri move
with open('sentinel/src/main.rs', 'r') as f:
    content = f.read()

print(f"\nmain.rs: {content.count(chr(10))} lines")

# Fix upstream_uri move
old_uri = '*upstream_req.uri_mut() = upstream_uri;'
new_uri = '*upstream_req.uri_mut() = upstream_uri.clone();'
if old_uri in content:
    content = content.replace(old_uri, new_uri, 1)
    print("OK: cloned upstream_uri before move")
elif 'upstream_uri.clone()' in content:
    print("NOTE: upstream_uri.clone() already present")
else:
    print("NOTE: uri assignment not found with expected text")

with open('sentinel/src/main.rs', 'w') as f:
    f.write(content)
print("main.rs written")

#!/usr/bin/env python3

# Fix sentinel/src/admission.rs: add #[allow(dead_code)] to AdmissionDecision and inflight_permit
with open('sentinel/src/admission.rs', 'r') as f:
    content = f.read()

print(f"admission.rs: {content.count(chr(10))} lines")

# Add allow on AdmissionDecision enum
old = '#[derive(Debug, Clone, Copy, PartialEq, Eq)]\npub enum AdmissionDecision {'
new = '#[derive(Debug, Clone, Copy, PartialEq, Eq)]\n#[allow(dead_code)]\npub enum AdmissionDecision {'
if old in content:
    content = content.replace(old, new, 1)
    print("OK: Added #[allow(dead_code)] to AdmissionDecision")
else:
    # Try without derive
    old2 = 'pub enum AdmissionDecision {'
    if old2 in content and '#[allow(dead_code)]' not in content:
        content = content.replace(old2, '#[allow(dead_code)]\n' + old2, 1)
        print("OK: Added #[allow(dead_code)] to AdmissionDecision (no derive before)")
    else:
        print("NOTE: AdmissionDecision already has allow or not found with expected pattern")

# Add allow on inflight_permit field
old3 = '    pub inflight_permit: OwnedSemaphorePermit,'
new3 = '    #[allow(dead_code)]\n    pub inflight_permit: OwnedSemaphorePermit,'
if old3 in content:
    content = content.replace(old3, new3, 1)
    print("OK: Added #[allow(dead_code)] to inflight_permit")
else:
    print("NOTE: inflight_permit not found")

with open('sentinel/src/admission.rs', 'w') as f:
    f.write(content)
print("admission.rs written")

# Fix sentinel/src/observability.rs: add #[allow(dead_code)] to on_response_log
with open('sentinel/src/observability.rs', 'r') as f:
    content = f.read()

print(f"observability.rs: {content.count(chr(10))} lines")

old4 = 'pub fn on_response_log('
new4 = '#[allow(dead_code)]\npub fn on_response_log('
if old4 in content and '#[allow(dead_code)]' not in content:
    content = content.replace(old4, new4, 1)
    print("OK: Added #[allow(dead_code)] to on_response_log")
else:
    print("NOTE: on_response_log already has allow or not found")

with open('sentinel/src/observability.rs', 'w') as f:
    f.write(content)
print("observability.rs written")

# Check sentinel/src/main.rs for on_response_log import
with open('sentinel/src/main.rs', 'r') as f:
    content = f.read()
if 'on_response_log' in content:
    print("WARNING: on_response_log still in main.rs imports!")
    # Remove it
    import re
    content = re.sub(r',\s*on_response_log', '', content)
    content = re.sub(r'on_response_log,\s*', '', content)
    with open('sentinel/src/main.rs', 'w') as f:
        f.write(content)
    print("Removed on_response_log from main.rs")
else:
    print("OK: on_response_log not in main.rs")

## Implementation

- [x] [serial] I1 Model requested and enforced network policy in action specs, build requests, and build/proof report data. r[build_correctness.no_network_by_default]
- [x] [serial] I2 Keep builtin fetchers on the declared fixed-output network boundary and deny ordinary derivation network access by default. r[build_correctness.no_network_by_default]
- [x] [serial] I3 Add scoped compatibility-capability reporting for any explicitly allowed build-time network exception. r[build_correctness.no_network_by_default]

## Verification

- [x] [serial] V1 Positive: prove a fixed-output fetcher can acquire declared source material and bind its expected digest. r[build_correctness.no_network_by_default]
- [x] [serial] V2 Negative: run a normal builder that attempts network access and prove strict/default policy blocks it with a deterministic diagnostic. r[build_correctness.no_network_by_default]
- [x] [serial] V3 Run focused network-policy tests plus Cairn validate and proposal/design/tasks gates for this change. r[build_correctness.no_network_by_default]

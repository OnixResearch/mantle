## Implementation

- [x] [serial] I1 Add a strict normalization policy model for time, timezone, locale, umask, temp roots, host/user metadata, modeled randomness, and deterministic ordering. r[build_correctness.determinism_normalization_policy]
- [x] [serial] I2 Apply normalization in build requests and proof sandbox envelopes while reporting unsupported controls as strong-claim blockers. r[build_correctness.determinism_normalization_policy]
- [x] [serial] I3 Bind the selected normalization policy and output-divergence diagnostics into build/proof receipts and docs. r[build_correctness.determinism_normalization_policy]

## Verification

- [x] [serial] V1 Positive: run equivalent strict builds under varied ambient time, locale, umask, temp roots, and user metadata and prove admitted evidence converges. r[build_correctness.determinism_normalization_policy]
- [x] [serial] V2 Negative: simulate unsupported normalization and divergent output digests and prove strong self-hosting/release claims are blocked. r[build_correctness.determinism_normalization_policy]
- [x] [serial] V3 Run focused normalization/perturbation tests plus Cairn validate and proposal/design/tasks gates for this change. r[build_correctness.determinism_normalization_policy]

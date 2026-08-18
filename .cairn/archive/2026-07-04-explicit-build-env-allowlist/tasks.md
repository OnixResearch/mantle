## Implementation

- [x] [serial] I1 Define the strict build environment policy data model and pure normalizer for allowed, required, denied, normalized, and redacted variables. r[build_correctness.explicit_environment_allowlist]
- [x] [serial] I2 Route strict build requests through constructed child environments and emit deterministic diagnostics for denied or missing entries. r[build_correctness.explicit_environment_allowlist]
- [x] [serial] I3 Bind the normalized environment digest and redacted rejection summaries into action/build reports and docs. r[build_correctness.explicit_environment_allowlist]

## Verification

- [x] [serial] V1 Positive: run a fixture whose declared environment normalizes to a stable digest and succeeds under strict mode. r[build_correctness.explicit_environment_allowlist]
- [x] [serial] V2 Negative: poison `LD_PRELOAD`, Rust/Cargo wrapper vars, proxy vars, token-like vars, and locale vars, and prove strict mode rejects or redacts them before execution. r[build_correctness.explicit_environment_allowlist]
- [x] [serial] V3 Run focused environment-normalization tests plus Cairn validate and proposal/design/tasks gates for this change. r[build_correctness.explicit_environment_allowlist]

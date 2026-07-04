## Why

Ambient process environments are a common hermeticity leak. Build actions need a constructed environment whose entries are declared, normalized, receipt-bound, and redacted where necessary. Hidden variables such as dynamic linker controls, Cargo/Rust wrappers, proxy settings, tokens, locale, and temp roots should not silently influence strong build-correctness claims.

## What Changes

- Define an explicit build-environment allowlist policy for strict hermetic builds.
- Construct builder environments from declared action inputs and normalized deterministic defaults instead of inheriting ambient values.
- Reject or redact denied/secret-bearing variables before execution and report deterministic diagnostics.
- Bind the normalized environment digest into action and build receipts.

## Impact

- **Files**: build-request environment normalization, pure policy classifier, report/audit events, docs, and Cairn build-correctness spec delta.
- **Testing**: positive declared-env build fixture; negative poisoned `LD_PRELOAD`, wrapper, token, proxy, and locale fixtures; Cairn validation and gates.

## Out of Scope

- Supporting arbitrary user environment passthrough in strict mode.
- Auditing secrets outside the declared build environment boundary.

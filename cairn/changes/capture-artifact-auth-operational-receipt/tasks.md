## Phase 1: Trust and persistence

- [x] [serial] I1 Implement pure full-key trust/currentness derivation and deterministic receipt construction/validation. r[mantle.artifact_auth_operational_receipt.trust]
- [x] [depends:mantle.artifact_auth_operational_receipt.trust] I2 Add bounded local action-result receipt persistence and fresh-context reload verification. r[mantle.artifact_auth_operational_receipt.persistence]
- [x] [depends:mantle.artifact_auth_operational_receipt.persistence] I3 Add positive and negative tests for restart replay, wrong same-name key, revoked/unknown/stale trust, malformed/tampered carrier, receipt drift, and unrelated failure parity. r[mantle.artifact_auth_operational_receipt.replay]
- [x] [depends:mantle.artifact_auth_operational_receipt.replay] I4 Document operational evidence and explicit authority non-admission. r[mantle.artifact_auth_operational_receipt.authority]

## Phase 2: Validation

- [x] [parallel] V1 Run focused tests, rustfmt, strict first-party Clippy, and Tiger Style. r[mantle.artifact_auth_operational_receipt.replay]
- [ ] [serial] V2 Run full workspace, Cairn, and Nix gates; sync and archive accepted requirements. r[mantle.artifact_auth_operational_receipt.authority]

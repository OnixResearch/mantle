# Tasks: Add lock-driven vendor fetches

Pure planning and its bounded negative controls are implemented; actual build
admission, vendor-tree hydration, profile parity, and acceptance remain open.

## Phase 1: Baseline and contract

- [x] [serial] T1.1 Create an isolated worktree from current `origin/main`. Record the vendored-tree payload size in the current profiles, the checked vendor-input validation, the dynamic-derivation admission surface, and focused baseline test output. The isolated fresh-clone-inputs manifest records `vendored-cargo-inputs.payload_bytes=896632634`; see `evidence/baseline-2026-10-01.json` and `evidence/isolated-vendor-2026-10-01.json` for exact commands and limits (no reduction claim). r[mantle.lock_vendor_fetch.bundle_payload_reduction]
- [x] [serial] T1.2 Define the producer contract: bounds, lock grammar subset, artifact admission, assembler layout grammar, and typed denial catalog. r[mantle.lock_vendor_fetch.producer_derivation]
- [x] [serial] T1.3 Record the lock-hashes-only and Cargo-first decisions in an ADR, including the offline override interaction. r[mantle.lock_vendor_fetch.lock_hash_reuse]

## Phase 2: Core producer

- [x] [serial] T2.1 Implement pure lock parsing, artifact admission, and layout planning with typed denials. r[mantle.lock_vendor_fetch.producer_derivation]
- [ ] [parallel] T2.2 Add positive fixtures: valid Cargo lock to N fetch derivations, assembler layout equality with the vendored tree. r[mantle.lock_vendor_fetch.producer_derivation] r[mantle.lock_vendor_fetch.bundle_payload_reduction]
- [x] [parallel] T2.3 Add negative fixtures: oversized lock, over-bound artifact count, hash-less dependency without a table, path escape, duplicate identity, contradictory entries. r[mantle.lock_vendor_fetch.producer_derivation] r[mantle.lock_vendor_fetch.negative_controls]

## Phase 3: Build and bundle integration

- [ ] [serial] T3.1 Run the producer as a derivation through dynamic-derivation admission with complete parent identity, emitting fixed-output fetches through the fetch service. r[mantle.lock_vendor_fetch.producer_derivation]
- [ ] [serial] T3.2 Add the shared lock table format with sorted union-merge semantics and subset reads. r[mantle.lock_vendor_fetch.shared_lock_tables]
- [ ] [serial] T3.3 Add profile modes excluding vendored trees with recorded producer identity, and hydration materialization with per-artifact verification. r[mantle.lock_vendor_fetch.bundle_payload_reduction]
- [ ] [parallel] T3.4 Add integration negative controls: tampered lock hash, missing artifact after fetch, unpacked-tree hash mismatch, excluded-but-unmaterialized preflight failure. r[mantle.lock_vendor_fetch.lock_hash_reuse] r[mantle.lock_vendor_fetch.negative_controls]

## Phase 4: Verification

- [ ] [serial] T4.1 Prove evaluation never reads the lock and nothing fetches at evaluation time, with a focused test. r[mantle.lock_vendor_fetch.producer_derivation]
- [ ] [serial] T4.2 Run a fresh-clone hydration without vendored trees end to end and record bundle-size reduction and identity parity in the profile receipt. r[mantle.lock_vendor_fetch.bundle_payload_reduction]
- [ ] [serial] T4.3 Run focused core and shell tests before and after changes, strict Clippy, policy freshness, and relevant Nix checks. Preserve exact blockers. r[mantle.lock_vendor_fetch.producer_derivation]
- [ ] [serial] T4.4 Sync accepted specs and archive through the isolated branch workflow with retained completion evidence. r[mantle.lock_vendor_fetch.bundle_payload_reduction]

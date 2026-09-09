# Drain progress — complete-store-capability-migration

Branch: `drain/store-capability-migration` (worktree `.pi/worktrees/store-capability-migration`).
Gates: proposal PASS, design PASS, tasks PASS (after strict concurrency markers).

## Done

- I1 complete: capability inventory at `evidence/capability-inventory.md` (commit `5dce464d`).
- Slice "transfer" (remote_transfer) migrated off raw services: `TransferObjectStore`
  capability added in `crunch-store` (`6703693c`).
- Slice "remote build" production paths migrated: host-path ingest, NAR ingest,
  NAR-output ingestion, and castore export all go through `TransferObjectStore`
  (`1f39201f`).
- Incidental pre-existing breakage fixed: `src/remote_build/tests/external_batch_hardware_tests.rs`
  `include_str!` paths still pointed at the deleted legacy `cairn/archive/`;
  repointed to `.cairn/archive/`. NOTE: the same stale `cairn/` dir exists
  untracked in the main checkout and masks this breakage there — delete it or
  take this fix via merge.

## Evidence so far

- `cargo test --bin mantle remote_transfer::` → 18 passed
- `cargo test --bin mantle remote_build` → 153 passed
- `cargo test -p crunch-store` → 357 + 2 + 9 passed; doctests 9 passed
  (including new compile_fail capability fixtures)

## Remaining

- Slice 3: `src/store_cmd.rs`, `src/main.rs` (administration ops: sign/verify/GC/pull/push)
- Slice 4: `crunch-pipeline`, `crunch-rust-cache`, last orchestrate raw call
- Slice 5: attest/bootstrap/foreign shells (attestation lookup + archive access capabilities)
- I4: split output admission from publisher execution (typed publication effect plan + observations)
- I5: deterministic architecture checker + compile-fail fixtures; zero external raw-service count
- I6: ADR 0058 update + ownership docs
- V1–V4 verification phases; then sync, archive, integration

## Host notes for resuming

- `nix develop` in the worktree fails: crates.io returns 403 to Nix's fetchurl
  for new vendored crate tarballs. Build directly with
  `PATH=~/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin:$PATH`,
  `CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=cc`,
  `SNIX_BUILD_SANDBOX_SHELL=<busybox-static>`,
  `PKG_CONFIG_PATH=<openssl-dev>/lib/pkgconfig`, `CARGO_TARGET_DIR=/tmp/mantle-drain-target`.
- The final `nix flake check -L` gate (V4) will need the crates.io fetch issue
  resolved (UA or mirror) or a warmed store.

## Update 2026-09-09 (2)

- Slice 3 (partial): store_cmd PathInfo listing moved behind bounded shell op (6a108cf9).
- I5: tools/check_store_capability_boundary.rs added. Self-test covers positive
  fixture, raw-service/writable/construction negative fixtures, and test-region
  exemption. Full-tree run: 0 raw-service escapes, 0 writable-authority escapes,
  0 handle-construction escapes beyond declared owners. Declared owners are
  recorded in the checker and must be mirrored into ADR 0058 (I6).
- crunch-rust-cache declared as store-backed adapter owning a private store
  instance; crunch-build orchestrate declared writable owner for CA mappings.

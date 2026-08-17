# Validation evidence

## Checkout-local offline Cargo closure

The explicit ignored `vendor-deps/` directory was regenerated from the locked graph with Cargo rather than patched one crate at a time. Pueue task `83` ran locked offline metadata with `CARGO_NET_OFFLINE=true`, a fresh empty `CARGO_HOME`, and `.cargo/vendor-config.toml`; every registry/git package resolved from the directory source. Mantle's self-build validator subsequently accepted the lock/package/file checksum set during every proof attempt, including the successful task `33` run.

This proves the current checkout-local input. The directory remains ignored and this evidence does not make a fresh clone independently offline.

## Focused repairs

- Task `89` passed all 10 positive/negative remote-failure-debug tests after replacing unstable `char::MAX_LEN_UTF8` with stable `char::MAX.len_utf8()`.
- Task `22` passed the required-policy staging test, both staged-path policy tests, and all 10 vendor-validator positive/negative tests.
- Task `26` passed direct successful/existing-destination no-replace tests plus OCI, release-publication, attempt-log, and remote-failure no-clobber fixtures.
- Task `32` passed both positive and negative typed Nickel dispatcher contract tests after production stdlib lookup stopped embedding `CARGO_MANIFEST_DIR`.

## Canonical fixed-point proof

Pueue task `33` ran:

```text
CRUNCH_NO_FUSE=1 ./scripts/prove-self-hosting.sh
```

The documented `CRUNCH_NO_FUSE=1` selection materialized sandbox inputs because this host cannot use the FUSE route; it did not relax the proof's path-leak, fixed-point, or later-stage strict-hermeticity checks.

The exact test result was:

```text
test self_hosting_stage0_stage1_stage2 ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 51 filtered out; finished in 2111.22s
```

Proof bundle:

```text
target/self-hosting-proof/run-20260717T153536Z-3902008
```

Inspected evidence:

- proof schema: `mantle-self-hosting-proof-v2`
- proof mode: `FixedPoint`
- selected provider kind: `legacy-fetch`
- protected-exec result: `success`
- staged source: `mjai2dkm5mdr7c8pdj832j5h9li9yvcj-mantle-src`
- stage1 BLAKE3: `8e9d0530274c3bb51969db33078822f42ea4dae2c4df2005ab4f4312aff2cba4`
- stage2 BLAKE3: `8e9d0530274c3bb51969db33078822f42ea4dae2c4df2005ab4f4312aff2cba4`
- stage1 equals stage2: `true`
- stage0 bwrap equals stage2 bwrap: `true`
- stage0 busybox equals stage2 busybox: `true`
- stage1/stage2 embedded store paths: empty
- stage0 hermeticity: `practical`, with explicit host-bwrap and checkout-source discovery fallbacks
- stage2 hermeticity: `strict`, with zero fallback events

`evidence/fixed-point-summary.txt` is the exact bundle `summary.txt`; task `55` passed a byte-for-byte `cmp`.

## Claim boundary

The proof establishes a current seed-assisted fixed point for the staged checkout and explicit checkout-local Cargo directory source under the selected materialized-input transport. It does not establish fresh-clone offline completeness, compiler correctness, seed trust removal, release reproducibility, independent rebuild agreement, deployment success, or full Cargo compatibility.

## Quality and dependency rails

Pueue task `63` ran `nix develop -c ./scripts/check-first-party-quality.sh` and passed all three configured legs:

```text
[1/3] rustfmt
running: cargo fmt --check -p mantle -p crunch-attestation -p crunch-build -p crunch-delta -p crunch-eval -p crunch-glue -p crunch-pipeline -p crunch-project -p crunch-shell -p crunch-store
[2/3] clippy
running: cargo clippy --workspace --all-targets --no-deps --exclude fuse-backend-rs --exclude nix-compat --exclude nix-compat-derive --exclude snix-build --exclude snix-castore --exclude snix-store --exclude snix-tracing -- -D warnings
[3/3] first-party workspace tests (serialized; vendored members excluded)
running: cargo test --workspace --lib --tests --exclude fuse-backend-rs --exclude nix-compat --exclude nix-compat-derive --exclude snix-build --exclude snix-castore --exclude snix-store --exclude snix-tracing -- --test-threads 1
test result: ok. 1547 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 18.57s
test result: ok. 1547 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 20.32s
```

Task `64` ran the documented dependency policy command:

```text
SNIX_BUILD_SANDBOX_SHELL=/bin/sh nix develop -c cargo-deny check --config deny.toml
```

It finished with:

```text
advisories ok, bans ok, licenses ok, sources ok
```

Task `72` passed `nix develop -c ./scripts/check-first-party-tigerstyle.sh` and `git diff --check`.

## Lifecycle

- Task `62` returned `valid: true` for Cairn validation and `PASS` for proposal, design, and tasks gates before sync. It reported 8/9 tasks complete, with only lifecycle closeout open.
- Task `74` ran sync dry-run and execution. The execute receipt hash was `9adc8089061f8b8d9fdd06a7d306c9cf00f69800a5421de18b6c8ad2dded6174`.
- The accepted requirement `r[bootstrap_inventory.self_build_source_closure]` was inspected in `cairn/specs/bootstrap-inventory/spec.md`; all six scenarios and the original spec wrapper are present.
- Evidence-backed implementation and verification bridges were added to `tools/tracey_refs.rs` only after accepted requirement and proof evidence existed.
- Task `76` reran Tracey, Cairn validation, and all three gates. Tracey reported `145/145 referenced`; validation returned `valid: true`; proposal, design, and tasks gates all returned `PASS`.
- Task `79` was the final pre-archive packet after all task evidence was recorded. `git diff --check` passed, validation remained `valid: true`, all three gates remained `PASS`, the tasks gate reported 9/9 complete with zero remaining, and Tracey remained `145/145 referenced`.

## Archive and exact post-archive state

Task `84` ran archive dry-run and execution with `CAIRN_ARCHIVE_DATE=2026-07-17`. The execute receipt hash was `c8d6e0254472a6f787e3a5ea7d6d5d597f5301d133d6c473f1033671961dae39`.

Task `85` produced the first exact machine-readable post-archive receipts. After this transcript was updated, task `87` refreshed those receipts and added the exact Tracey result beside them:

- `post-archive-validation.json` reports `"changes": 0`, empty issue/finding lists, and `"valid": true`.
- `post-archive-change-list.json` reports an empty `changes` array.
- `post-archive-tracey.txt` reports `traceability coverage ok: 145/145 referenced (profile mantle-default)`.

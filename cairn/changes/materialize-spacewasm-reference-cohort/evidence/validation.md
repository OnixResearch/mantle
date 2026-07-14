# SpaceWasm reference cohort validation evidence

Date: 2026-07-14
Change: `materialize-spacewasm-reference-cohort`
Status: implementation complete except for the explicitly blocked repository-wide machine-contract freshness task; not synced or archived.

## Source review replacement and replay

- Earlier proposal review revision: `30cd6e9b91f84a39278edcb5d66514b773011ccc`.
- Selected NASA SpaceWasm revision from Octet's reviewed `spacewasm-mvp` projection: `e24cf09355a90497148eb5029fdb8e3400bd63e3`.
- Fixed source archive BLAKE3: `0db57636fb83a8c47f0c55601236cab648dd5edfdd2567cae0619aa74b8c4da3`.
- Fixed Cargo.lock BLAKE3: `4e6a07910125d4c921a62cec03d8bc9acb3bf341ebd1a11999df08d6c6c5c5f1`.
- The replacement was not assumed equivalent. Bundle member `reports/replay-evidence.json` records `replay_required: true`, `replay_status: complete`, the old/new revisions, and eight passing fixture results.
- The validated core/profile/fixture/runner checkpoint is commit `b7bde701b37abc09b65c101f17dd1798d59d9350`.

## Focused Rust/profile checks

The following commands completed successfully:

```text
nix develop -c cargo fmt -p crunch-spacewasm-core -p crunch-spacewasm -- --check
nix develop -c cargo test -p crunch-spacewasm-core -p crunch-spacewasm
nix develop -c cargo clippy -p crunch-spacewasm-core -p crunch-spacewasm --all-targets --no-deps -- -D warnings
```

Evidence: pueue task `1086` exited successfully after formatting, focused positive/negative tests, and strict first-party clippy.

The pure `#![no_std]` core also compiled for the declared wasm target:

```text
nix shell .#spacewasm-reference-rust-toolchain nixpkgs#clang nixpkgs#mold \
  --option builders '' --option secret-key-files '' \
  -c env RUSTC_BOOTSTRAP=1 CARGO_TARGET_DIR=/tmp/mantle-spacewasm-wasm-check \
  cargo check -p crunch-spacewasm-core --target wasm32-unknown-unknown
```

Evidence: pueue task `1109` exited successfully.

## Nix materialization and bundle evidence

The fixed-output source fetch, Cargo-lock import, Cargo-offline host/wasm builds, upstream unit tests, bounded `address` spectest, deterministic fixture generation, diagnostic replay, corpus tar materialization, and immutable bundle assembly completed through:

```text
nix shell nixpkgs#nickel -c nickel export --format json \
  packages/spacewasm-reference/profile.ncl \
  > packages/spacewasm-reference/generated/profile.json
nix build .#spacewasm-reference-bundle --no-link --print-out-paths -L \
  --option builders '' --option secret-key-files ''
```

Evidence: pueue task `1106` exited successfully and produced; task `1186` rebuilt twice and confirmed the identical output path:

```text
/nix/store/zxs87x9m4xfjrs8gyvc21k8acmk20dhn-mantle-spacewasm-reference-bundle-e24cf09355a90497148eb5029fdb8e3400bd63e3
```

Measured identities from that bundle:

- profile identity BLAKE3: `cceb1bd03f90dd382d4c7ff6e79266650830a7c87e9fae07db2e48773469eb67`
- cohort identity BLAKE3: `fb2c9e84459271828ad6840093c80af359508f47b06125f6741b14a31c9cc103`
- report identity BLAKE3: `64eae12a64f3eace64505b3d8a10dc439c71b5a347c4e3ba86eb96eba64245b8`
- bundle identity BLAKE3: `865152f8bd3b33414dc0b65e786e211a5d17830bcdfaa70ca31680c2de3e0c78`
- materialization disposition: `complete`
- source admitted: `true`
- support projection matched: `true`

The exact recorded check states are six `passed`, one `skipped`, two `unavailable`, and one `unsupported`. The eight fixture-class results are all `passed`. No absent upstream workflow or continuous fuzzing run was synthesized.

Independent relative-path remeasurement was run through a user-namespace bind at `/tmp/mantle-spacewasm-portable-bundle`, outside the producing store path:

```text
unshare --mount --map-root-user ... mantle-spacewasm-reference verify \
  /tmp/mantle-spacewasm-portable-bundle
```

Evidence: pueue task `1154` returned `valid: true`, bundle identity `865152f8bd3b33414dc0b65e786e211a5d17830bcdfaa70ca31680c2de3e0c78`, and no diagnostics.

The Nix negative rail replaced the source archive member with unrelated fixture bytes and was rejected with `source-archive-drift`:

```text
nix build .#checks.x86_64-linux.spacewasm-reference-negative --no-link -L \
  --option builders '' --option secret-key-files ''
```

Evidence: pueue task `1108` exited successfully only after confirming the inner materialization failed closed.

## Lifecycle validation

All current lifecycle structure and advisory gates passed:

- `cairn validate --root .`: pueue task `1156`, `valid: true`, no issues.
- `cairn gate proposal materialize-spacewasm-reference-cohort --root .`: task `1159`, `PASS`.
- `cairn gate design materialize-spacewasm-reference-cohort --root .`: task `1158`, `PASS`.
- `cairn gate tasks materialize-spacewasm-reference-cohort --root .`: task `1157`, `PASS`; post-evidence rerun task `1175`, `PASS`.
- Post-evidence `cairn validate --root .`: task `1173`, `valid: true`, no issues.

## Exact blocker

Repository-wide machine-contract freshness remains blocked by an unrelated, pre-existing digest mismatch:

```text
nix develop -c cargo -Zscript scripts/check-machine-schema-contracts.rs
```

Pueue task `1110` failed with:

```text
release.function-address-binding [digest] /freshness/producer_identity_blake3:
stale BLAKE3 binding:
recorded=629db7b17f6cdaaaee33b4ddb5aae92b975a1a894736ddff1213831700b9699f
expected=6d1618b3abcd3df6dbdeafc52d72b296615390038c9591526a5c9035470abe0c
```

This mismatch was present in the baseline and is outside this change's files. It was not regenerated or hidden. The final comprehensive-validation task remains unchecked. The next best check—focused SpaceWasm profile/core/build/fixture/bundle verification plus Cairn validation and all three lifecycle gates—passed as recorded above.

## Machine-contract/release boundary decision

No root machine-contract registry or release-evidence extension was needed. The new surface is an opt-in, separately named Nix package and diagnostic CLI with self-contained versioned profile, report, manifest, receipt, and verification schemas. It does not alter root `mantle --json`, release assembly, release verification, or consumer admission semantics.

## Non-claims

The verified bundle proves only exact materialization and linkage facts. It does not prove SpaceWasm correctness, memory safety, WebAssembly conformance, flight qualification, sandbox effectiveness, consumer runtime admission, production readiness, or release eligibility.

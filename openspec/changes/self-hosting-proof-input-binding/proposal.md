## Why

The self-hosting proof already shows stage0 -> stage1 -> stage2, but two
holes can still make the evidence weaker than it looks:

- stage2 can drift back toward the checkout tree unless the proof reuses the
  exact staged source tree from stage0
- the final self-build derivation can rediscover bootstrap tools by scanning a
  shared store, which lets stale `*-bwrap` or `*-busybox` siblings leak in

This needs to be tracked as its own self-build-proof hardening change. It is
not part of native provenance attestations.

## What Changes

- Record the staged source tree used by `crunch self-build`
- Let stage2 reuse that exact staged source tree via an internal
  `--source-store-path` input
- Validate a reused staged source tree against its staged store-name hash
- Thread the exact step-2 `bwrap` and `busybox` root outputs into the final
  self-build derivation instead of rescanning the store
- Run the proof's stage2 from outside the repo with `PATH` cleared so host
  source-staging fallbacks fail instead of passing silently

## Capabilities

### New Capabilities
- `self-hosting-proof-source-binding`: stage2 can reuse the exact staged
  source tree from stage0
- `self-build-exact-bootstrap-inputs`: final self-build derivation binds to
  the exact exported `bwrap` and `busybox` outputs selected in step 2

### Modified Capabilities
- `self-build-proof-report`: includes the staged source path in addition to
  invoking binary, bwrap source, busybox path, and final output binary
- `self-hosting-proof`: asserts exact staged-source reuse and exact
  crunch-built tool reuse during stage2

## Impact

- **Files**: `src/main.rs`, `src/self_build.rs`, `tests/self_hosting.rs`
- **APIs**: hidden internal `self-build --source-store-path` plumbing and
  expanded proof-report markers
- **Dependencies**: none new
- **Testing**: self-build unit coverage plus self-hosting breadcrumb and proof
  assertions

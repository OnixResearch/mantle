## Why

The self-hosting proof now demonstrates a real fixed point: stage1 rebuilds
stage2, and the crunch-built `bwrap` and `busybox` outputs match across the
proof. That is strong evidence for self-hosting, but it is still not the same
claim as a truly Nix-free first bootstrap.

Today the first bootstrap path still depends on undeclared or Nix-shaped host
assumptions:

- `scripts/prove-self-hosting.sh` can still fall back to `nix-build` to
  realize `pkgsStatic.busybox` when no static busybox is already available.
- the proof helper and self-build path probe NixOS- and Nix-specific
  locations such as `/run/wrappers/bin`, `/run/current-system/sw/bin`, and
  `/nix/store/*` while resolving tools.
- stage0 source staging still depends on host `git`, `tar`, `cp`, `sh`, and
  `cargo vendor`.
- the checked-in proof runs on a NixOS host and proves self-hosting, not that
  the initial bootstrap path works on a host where Nix commands are absent.

That leaves an important gap in the bootstrap story. A contributor can now ask
an exact question — “is first bootstrap Nix-free yet?” — and the honest
answer is still “not proven.” We should track the missing work explicitly
instead of leaving it implied by the existing self-hosting proof.

## What Changes

- Add bootstrap-spec requirements for a stricter stage0 contract: the first
  bootstrap path must not invoke `nix-build`, `nix-store`, `nix-shell`, or
  `nix develop`, even as hidden fallback behavior.
- Require stage0 prerequisites to be classified explicitly as either host
  prerequisites or pinned fetched bootstrap artifacts, with no implicit Nix
  realization step.
- Require a distinct proof path for “Nix-free first bootstrap” so the repo
  stops conflating self-hosting with a host-independent first bootstrap.
- Track the remaining trust-reduction implementation work: seed provisioning,
  source staging without host shell glue, vendored Rust source preparation,
  and non-Nix host proof coverage.

## Capabilities

### New Capabilities
- `stage0-nix-free-contract`: contributors can inspect one precise definition
  of what counts as a Nix-free first bootstrap
- `stage0-explicit-seeds`: missing bootstrap seeds fail fast instead of being
  realized through hidden Nix fallbacks
- `stage0-non-nix-proof`: a future proof path can distinguish “self-hosting”
  from “works without Nix on the host”

### Modified Capabilities
- `self-hosting-proof`: stays the fixed-point proof, but no longer carries the
  stronger first-bootstrap claim by implication
- `bootstrap-trust-inventory`: now has to classify stage0 prerequisites and
  hidden fallbacks more strictly

## Impact

- **Files**: `openspec/specs/bootstrap/spec.md`, bootstrap docs/README, proof
  helper docs, and likely `scripts/prove-self-hosting.sh` / `src/self_build.rs`
  in follow-up implementation work
- **APIs**: possible future CLI/helper flags for explicit bootstrap seed paths
- **Dependencies**: removes implicit Nix fallback expectations over time
- **Testing**: `openspec validate non-nix-stage0-bootstrap`, plus future proof
  runs with Nix commands intentionally absent from `PATH`

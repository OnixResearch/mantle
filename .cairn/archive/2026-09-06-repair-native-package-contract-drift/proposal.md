# Repair native package contract drift

## Why

The native package check at `4a433bf1c0662f423b6668cb1afe57070613d92c` reports nine failed integration targets.
The earlier fixture repair passed its selected suites but did not establish a usable default package.
`evidence/package-test-fixtures-2026-09-05.md` retains the exact historical observations.

Mantle owns these contracts. Neural Stream remains a blocked consumer and supplies no runtime or release authority.
This change restores parity with accepted contracts instead of adjusting assertions to arbitrary outcomes.

## What Changes

- Correct invalid foreign derivation fixtures and test the intended rejection boundary without weakening admission.
- Reconcile example, machine-contract, and operator inventories with their declared owners and generators.
- Restore composed store observations through narrow capabilities, without base mutation, backfill, or trust fallback.
- Remove the six baseline gateway store-authority findings through a minimal capability adapter. Preserve admission and signature checks.
- Preserve worker capture facts separately from coordinator diagnostics and the original build failure.
- Keep documentation and bounded architecture tests current. Run the complete no-fail-fast native package gate.

## Impact

- **Prerequisite**: published repair branch revision `5dfb37becbd09b895e84b8ba4af911a7f7513d91`, incorporated explicitly after branch creation from `origin/main`.
- **Files**: the nine named integration targets, their fixtures and inventories, `crates/crunch-store`, `src/remote_gateway.rs`, and the remote failure reporting shell as indicated by focused evidence.
- **Contracts**: existing store-lifecycle, store-transports, remote-builds, foreign-derivation-import, machine-artifact-contracts, examples, operator-diagnostics, and build-tool-boundary requirements remain authoritative.
- **Testing**: focused positive/negative controls, strict scoped Clippy and Tiger Style, generated-artifact freshness, lifecycle gates, and the full native package check.

## Non-goals

No new dependency, format version, trust root, module-layer semantics, global admission, training, tensor output, or production promotion belongs here.
The change does not repair remote Nix signature policy or accept an ambient worktree as a downstream dependency.
It does not archive the unrelated `add-dev-cache-cross-run-resume` change.

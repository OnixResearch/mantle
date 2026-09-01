# Committed-source lifecycle validation

## Verdict

The promotion implementation remains valid from committed source. Cairn
validation, Tracey coverage, and all three gates pass with every task checked.
The focused Nix-shell tests and formatting check also pass.

## Passing checks

- `nix flake check --no-build -L`;
- `nix build .#wasi-virt -L --no-link`;
- Nix-shell bootstrap parity core: 91 tests;
- Nix-shell bootstrap parity CLI: 19 tests;
- Nix-shell root-package formatting;
- machine-contract generation and check;
- strict first-party Clippy and `git diff --check`;
- Cairn validation;
- Tracey coverage: 155/155;
- proposal, design, and tasks gates.

## Nix transport repair

The initial `nix flake check -L` stopped when `buildRustPackage` requested
locked `wasi-virt` crates through a blocked `crates.io/api` endpoint. The
component now uses the existing Crane vendor path and pinned Rust 1.90.0
component toolchain. The source revision, Cargo lockfile, package, and feature
selection did not change.

The direct `wasi-virt` Nix build now passes. ADR 0102 records this bounded
transport decision.

## Preserved full-check blocker

A local-builder `nix flake check -L --builders ''` now gets beyond the former
`wasi-virt` fetch blocker. It stops at the existing
`bootstrap-blocker-inventory` check:

```text
bootstrap blocker inventory: 115 findings across 3 classes,
355 evidence-backed suppressions, 0 promotion claims, enforce=true
```

The full-check status is `1`. `nix-full.log` preserves the exact output. This
change does not disable, baseline, or weaken that repository-wide gate. The
promotion-specific checks remain valid and complete.

## Review checkpoint

- **Question:** Does the completed promotion remain valid from committed source,
  with Nix and lifecycle failures classified without weakening a gate?
- **Inspected evidence:** direct and Nix-shell tests, independent bundle replay,
  machine contracts, formatting, strict Clippy, Nix evaluation, direct
  `wasi-virt` build, full Nix check, Tracey, Cairn validation, and all three
  gates.
- **Decision:** accept the promotion evidence and preserve the unrelated
  bootstrap inventory as a bounded repository-wide blocker.
- **Owner:** `promote-full-bootstrap-parity`.
- **Next action:** sync the accepted delta, archive the change, rerun
  post-archive validation, and record the final repository status.

## Non-claims

This evidence does not claim that every repository-wide Nix check passes. It
does not prove compiler correctness, seed correctness, kernel isolation,
independent rebuild agreement, release reproducibility, deployment success, or
full Cargo compatibility.

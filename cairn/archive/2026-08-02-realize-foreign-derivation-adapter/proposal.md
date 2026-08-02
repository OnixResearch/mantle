# Proposal: Realize compiled foreign derivations through a Mantle adapter

## Why

A compiled foreign executable plan still lacks source material, execution policy, scheduler registration, and store effects. Treating that plan as build evidence would cross the current import trust boundary.

Mantle needs a thin realization adapter. It must feed resolved native units into the existing scheduler and store without adding Guix or Nix frontend semantics.

Foreign builders also need explicit sandbox behavior. They must not inherit Mantle’s current `/bin/sh`, environment, or work-directory defaults without receipt-bound policy.

## What Changes

- Add source-bundle and fixed-output materialization for compiled foreign source requirements.
- Add typed, per-derivation execution profiles with bounded environment, work-directory, shell, network, syscall, and writable-path policy.
- Bind each execution-profile digest into the target derivation identity.
- Register resolved units in `DerivationRegistry` and realize selected roots through `Builder::build_all`.
- Add `mantle foreign-import realize` as a thin local adapter command.
- Emit a separate `mantle-foreign-realization-receipt-v1` with source, scheduler, store, profile, and non-claim facts.

## Dependencies

- `compile-foreign-derivation-graphs`.

## Non-Goals

- Adding a second scheduler or foreign store implementation.
- Running Guix, Nix, flakes, overlays, or package-module evaluation during consumption.
- Providing global `/bin/sh` compatibility for Guix profiles.
- Supporting remote foreign realization in the first version.
- Claiming output provenance, package correctness, bootstrap parity, reproducibility, or OS bootability.
- Owning Shepherd, activation, account setup, initrd construction, or VM assembly.

## Impact

- **Files**: `src/foreign_import_cmd.rs`, a foreign realization core and shell, `crates/crunch-build/`, `crates/crunch-glue/`, `crates/crunch-store/`, typed Nickel policy, source-bundle integration, CLI tests, documentation, and lifecycle evidence.
- **Testing**: source admission, profile identity, no-`/bin/sh` Guix behavior, scheduler registration, fixed-output mismatch, cancellation, partial failure, store persistence, receipt identity, and explicit non-claims.

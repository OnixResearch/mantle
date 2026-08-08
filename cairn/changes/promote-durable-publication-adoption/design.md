# Design

## Context

The reviewed Mantle adoption commit is `d1f3d6d96e2b0d9cd8497cd89b8e5a93d4a7dfaf`. Current canonical `main` is a descendant of the old merge base, but not of the reviewed commit. Current `main` already carries the same producer revision and newer evidence identities from later credential and integration work.

Onix Core now preserves its reviewed durable-publication admission and concurrent Choregraph admission on canonical `main` through `bc4629c9e766d3db82e4dab9fe8c166c360b8435`.

## Decisions

### Preserve both Mantle histories

Use current fetched `origin/main` as the first merge parent and the exact reviewed adoption commit as the second parent. The merge must preserve both ancestors. It must not force-push, rebase, squash, or replace either history.

### Keep the newer canonical tree

For duplicate adoption files, keep current canonical bytes. Current `main` contains newer audited Cargo, Nix, receipt, and documentation identities. The merge candidate may differ from its first parent only in the named promotion lifecycle paths, the three adoption receipt files under `evidence/radicle/`, and `lib/durable-file-publication-adoption-receipt.ncl`. The receipt and validator refresh may update only the BLAKE3 bindings for `Cargo.toml`, `Cargo.lock`, `flake.nix`, and `flake.lock`; it must preserve the accepted producer identity, mapping, validation record, authority boundary, and non-claims.

### Require canonical producer ordering

Before Mantle mutation, fresh Onix Core `origin/main` must contain both `bc4629c9e766d3db82e4dab9fe8c166c360b8435` and accepted admission commit `b8387cd7d59fa3b0d4ea67646352dd27c4f7d7ed`.

### Re-evaluate remote state before push

Fetch Mantle `origin`, resolve `origin/main`, and prove that it is an ancestor of the exact merge candidate immediately before mutation. A moved or divergent target stops promotion. Force-push and pull-request paths remain forbidden.

### Gate promotion with focused accepted evidence

Run the shared-backend tests, Mantle binary tests sequentially, package build, version probe, formatting, product-owned Clippy, Tiger Style, Nickel evidence, focused Nix adoption check, Cairn validation, and focused traceability.

Focused backend, package, product Clippy, Nickel, Nix, Cairn, and traceability checks must pass. An exact pre-existing broad failure may remain only when first-parent path review proves that the candidate did not change its implementation surface and the evidence records the full failure signature. Current accepted blockers are the three bootstrap-parity binary tests, formatting in `src/source_built_fixed_point_shell.rs`, and Tiger Style findings outside the durable-publication surface.

`restore-durable-publication-broad-validation` owns those broad blockers plus unrelated source-filter, blocker-inventory, vendored-dependency, and Octet failures. Those failures remain visible.

## Failure handling

Any producer-order, ancestry, first-parent path, focused-check, new broad-check, push, or remote-verification failure stops promotion without weakening policy or rewriting canonical history. An exact recorded pre-existing broad failure does not block this ancestry-only promotion when the candidate does not change that failure's implementation surface.

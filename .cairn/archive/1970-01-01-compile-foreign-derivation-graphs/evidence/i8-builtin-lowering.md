# I8 bounded builtin lowering evidence

Task-ID: I8
Covers: r[foreign_derivation_import.foreign_builtin_lowering]
Date: 2026-08-01

## Question

Can Mantle lower declared foreign download and Git download nodes into bounded native fetch facts without a foreign fetcher?

## Inspected evidence

- `src/foreign_graph_compiler.rs`
- `crates/crunch-build/src/fetcher.rs`
- `crates/crunch-build/src/network_policy.rs`
- Pueue task `7910`: focused graph compiler tests
- Pueue task `7912`: strict first-party Clippy

## Decision

I8 is complete.

The compiler accepts explicit `builtin:download` and `builtin:git-download` forms with fixed-output metadata. It lowers them to Mantle’s `builtin:fetchurl` unit shape.

Download facts bind ordered candidates, flat or recursive mode, and the executable flag. Git facts bind ordered candidates, revision, recursive hash mode, and the `checkout-no-dot-git` export policy.

Candidate count is bounded. Empty, invalid, or repeated candidates fail closed. Unsupported builtins fail before resolved registration. The pure compiler performs no network or process operation.

The focused suite passed seven graph compiler tests. It also proved that the lowered executable and Git units parse through Mantle’s native fetch parser.

The strict first-party Clippy command completed with `-D warnings`. Vendored `snix-castore` emitted one existing `dead_code` warning.

## Owner

Mantle foreign builtin lowering.

## Next action

Finish I9 and I10 when the executable plan adds its role-labeled BLAKE3 plan identity and validator tests.

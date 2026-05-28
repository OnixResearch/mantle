# rust-topology-proc-macro-target-normalization

## Why

Native host-unit planning matches Cargo proc-macro units by package id, target name, and target kind. Cargo's unit graph reports lib/proc-macro target names with crate-name spelling (`curve25519_dalek_derive`), while manifest-derived native package facts can retain package-target spelling (`curve25519-dalek-derive`). That mismatch caused the selected `curve25519-dalek-derive` proc macro to be omitted even though Cargo selected it, leaving curve25519-dalek's SIMD specialization attributes unexpanded.

## Change

Normalize proc-macro target names through Rust crate-name spelling when matching selected Cargo host units to native host-unit facts. Preserve custom-build normalization and do not plan absent host units merely because a proc-macro package exists in the source closure.

## Success

- A focused unit test proves hyphenated proc-macro package facts match underscore Cargo target names.
- A negative assertion proves unselected proc-macro packages remain omitted.
- Clean self-probe advances beyond the curve25519 duplicate-import blocker or records the next frontier.

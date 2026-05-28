# rust-topology-proc-macro-crate-type

## Why

Native topology reaches `spez@0.1.2` after custom-build alias binding, then invokes rustc with `--crate-type lib` even though `vendor-deps/spez/src/lib.rs` contains `#[proc_macro]`. The dirty probe showed Mantle's native package target planning classified `spez` as a normal lib because the vendored, Cargo-normalized manifest spells the proc-macro library flag as `proc_macro = true`, while Mantle only recognized `proc-macro = true`.

A related Cargo-unit seam is the same classification boundary: if Cargo unit material exposes a lib-shaped target with proc-macro crate type, Mantle must classify it consistently as a host proc macro rather than target lib.

## Change

Teach Rust proc-macro target classification to accept both manifest spellings (`proc-macro` and `proc_macro`) and to treat lib/rlib-shaped Cargo unit targets with `crate_types = ["proc-macro"]` as proc-macro host units. Apply the same bounded classification to unit derivation, host artifact discovery, and selected host-unit matching.

## Success

- Focused tests prove the normalized `proc_macro = true` manifest spelling feeds proc-macro target planning.
- Focused tests prove proc-macro crate type overrides lib-shaped target kind for unit derivation, host artifact discovery, and selected native host matching.
- Negative tests prove ordinary lib crate types remain target units and malformed non-lib targets do not become proc macros.
- Clean self-probe advances beyond the `spez` proc-macro crate-type blocker or records the next deterministic frontier.

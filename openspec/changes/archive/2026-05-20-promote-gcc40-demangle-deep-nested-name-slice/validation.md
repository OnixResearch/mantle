# Validation

- `cargo fmt --check` — passed.
- `cargo test --bin mantle bootstrap_parity::tests::gcc40 -- --nocapture` — 34 passed.
- `./scripts/check-bootstrap-parity-snapshot.sh` — passed; report BLAKE3 `2d7a65542fecfe1225fa5a0e1715fa2817b96f793d4e1c1339286c96daf1a8e6`; blocker counts live-bootstrap 5, guix 6, stagex 2.
- `openspec validate promote-gcc40-demangle-deep-nested-name-slice --strict` — passed.
- `openspec validate --all --strict` — 50 passed, 0 failed.
- `git diff --check` — passed.

Bounded claim: this promotes one checked GCC 4.0 libiberty deep nested zero-argument Itanium demangle slice (`_ZN3foo3bar3bazEv -> foo::bar::baz()`) and preserves flat/two-component regressions. It does not prove full `cp-demangle` or native GCC 4.0 correctness; `gcc.4.0` remains evidence-backed partial and still blocks live-bootstrap/Guix parity.

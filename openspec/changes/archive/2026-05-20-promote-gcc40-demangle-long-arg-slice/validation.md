# Validation

Selected bounded input/output promoted:

- `_ZN3foo3bar3bazEl -> foo::bar::baz(long)`

Preserved regressions:

- `_Z3foov -> foo()`
- `_ZN3foo3barEv -> foo::bar()`
- `_ZN3foo3bar3bazEv -> foo::bar::baz()`
- `_Z3fooi -> foo(int)`
- `_ZN3foo3barEi -> foo::bar(int)`
- `_ZN3foo3bar3bazEi -> foo::bar::baz(int)`
- `_Z3fooc -> foo(char)`
- `_ZN3foo3barEc -> foo::bar(char)`
- `_ZN3foo3bar3bazEc -> foo::bar::baz(char)`

Rejected unsupported shapes remain fail-closed, including unsupported `float`/`long long` type encodings and too-deep nested names.

Validation run before archive:

```text
cargo fmt --check
cargo test --bin mantle bootstrap_parity::tests::gcc40 -- --nocapture
./scripts/check-bootstrap-parity-snapshot.sh
openspec validate promote-gcc40-demangle-long-arg-slice --strict
git diff --check
```

Observed results:

- focused GCC 4.0 Rust tests: 34 passed, 0 failed
- bootstrap parity snapshot: passed
- report BLAKE3: `2d7a65542fecfe1225fa5a0e1715fa2817b96f793d4e1c1339286c96daf1a8e6`
- live-bootstrap blockers: 5
- Guix blockers: 6
- StageX blockers: 2
- active OpenSpec strict validation: passed
- whitespace diff check: passed

Bounded non-claims:

- Does not prove arbitrary type decoding.
- Does not prove full native `cp-demangle`.
- Does not prove full native GCC 4.0 correctness.
- Does not complete live-bootstrap/Guix/StageX parity.
- `gcc.4.0` remains evidence-backed partial.

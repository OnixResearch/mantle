# Validation — promote-gcc40-demangle-char-arg-slice

Selected bounded input/output:

- `_ZN3foo3bar3bazEc -> foo::bar::baz(char)`

Preserved regressions:

- `_Z3foov -> foo()`
- `_ZN3foo3barEv -> foo::bar()`
- `_ZN3foo3bar3bazEv -> foo::bar::baz()`
- `_Z3fooi -> foo(int)`
- `_ZN3foo3barEi -> foo::bar(int)`
- `_ZN3foo3bar3bazEi -> foo::bar::baz(int)`

Rejected unsupported examples include non-char/non-int type code and too-deep nested shapes.

Commands run before archive:

```text
cargo fmt --check
cargo test --bin mantle bootstrap_parity::tests::gcc40 -- --nocapture
./scripts/check-bootstrap-parity-snapshot.sh
openspec validate promote-gcc40-demangle-char-arg-slice --strict
openspec validate --all --strict
git diff --check
```

Results:

- `cargo test --bin mantle bootstrap_parity::tests::gcc40 -- --nocapture`: 34 passed, 0 failed.
- Bootstrap parity snapshot: passed.
- Snapshot report BLAKE3: `2d7a65542fecfe1225fa5a0e1715fa2817b96f793d4e1c1339286c96daf1a8e6`.
- Blocker counts: live-bootstrap 5, Guix 6, StageX 2.
- OpenSpec strict validation: change valid; all validation 50 passed, 0 failed.
- `git diff --check`: passed.

Bounded non-claim:

- This does not prove arbitrary type decoding.
- This does not prove full native `cp-demangle`.
- This does not prove full native GCC 4.0 correctness.
- `gcc.4.0` remains evidence-backed `partial`; live-bootstrap/Guix parity remain incomplete.

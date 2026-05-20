# Validation Evidence

Change: `promote-gcc40-demangle-int-arg-slice`

## Bounded claim

Promotes only a selected bounded GCC 4.0 `libiberty` Itanium single-`int` argument demangle slice:

- selected: `_ZN3foo3bar3bazEi -> foo::bar::baz(int)`
- int regressions: `_ZN3foo3barEi -> foo::bar(int)`, `_Z3fooi -> foo(int)`
- zero-argument regressions preserved: `_ZN3foo3bar3bazEv -> foo::bar::baz()`, `_ZN3foo3barEv -> foo::bar()`, `_Z3foov -> foo()`
- unsupported/deeper/non-int shapes rejected, including `_ZN3foo3bar3bazEf`, `_ZN3foo3bar3baz3quxEi`, and `_ZN3foo3bar3baz3quxEv`

Non-claim: this does not prove full `cp-demangle`, arbitrary type-list decoding, full native GCC 4.0 correctness, live-bootstrap parity, or Guix parity.

## Commands run

Environment used the repo-local Cargo target dir, pinned nightly Rust, clang/mold/pkg-config/OpenSSL paths, static `SNIX_BUILD_SANDBOX_SHELL`, and disabled rustc wrappers.

- `cargo fmt --check` — passed
- `cargo test --bin mantle bootstrap_parity::tests::gcc40 -- --nocapture` — passed, 34 tests
- `./scripts/check-bootstrap-parity-snapshot.sh` — passed
  - report: `target/bootstrap-parity-snapshot/latest/report.json`
  - receipt: `target/bootstrap-parity-snapshot/latest/receipt.json`
  - report BLAKE3: `2d7a65542fecfe1225fa5a0e1715fa2817b96f793d4e1c1339286c96daf1a8e6`
  - live-bootstrap: `complete=false blockers=5`
  - guix: `complete=false blockers=6`
  - stagex: `complete=false blockers=2`
- `openspec validate promote-gcc40-demangle-int-arg-slice --strict` — passed
- `openspec validate --all --strict` — passed, 50 passed / 0 failed before archive
- `git diff --check` — passed

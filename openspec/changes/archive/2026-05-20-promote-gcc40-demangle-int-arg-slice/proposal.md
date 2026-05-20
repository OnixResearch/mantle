## Why

GCC 4.0 `libiberty` demangle support is advancing through evidence-backed bounded slices. The current checked frontier covers flat, two-component nested, and selected three-component nested zero-argument Itanium names, but still rejects the next small argument-bearing shape.

## What Changes

- Promote one bounded native demangle shape: selected single-`int` Itanium function arguments.
- Preserve the already checked flat, two-component nested, and three-component nested zero-argument regressions.
- Keep unsupported signatures and deeper nesting rejected.
- Refresh receipt/frontier evidence and fail-closed parity tests.

## Scope

In scope: `_ZN3foo3bar3bazEi -> foo::bar::baz(int)` plus bounded flat/nested int regressions if used by the smoke.

Out of scope: full `cp-demangle`, arbitrary type decoding, overload sets, templates, constructors/destructors, and full native GCC 4.0 correctness.

## Verification

- `cargo fmt --check`
- `cargo test --bin mantle bootstrap_parity::tests::gcc40 -- --nocapture`
- `./scripts/check-bootstrap-parity-snapshot.sh`
- `openspec validate promote-gcc40-demangle-int-arg-slice --strict`
- `openspec validate --all --strict`
- `git diff --check`

## Why

All existing tests are unit tests — they construct Rust structs, call
functions, and assert results. No test exercises the path a user takes:

```
crunch eval hello.ncl    →  Nickel source → JSON output
crunch build hello.ncl   →  Nickel source → sandbox → store path on disk
crunch bootstrap         →  Nix query → seed.ncl file
```

The gaps between crates are where bugs hide:

- crunch-eval deserializes to `Expr`, main.rs calls `.to_serde::<CrunchDerivation>()`.
  If the Nickel contract and the Rust struct diverge, neither crate's unit
  tests catch it.
- crunch-glue produces a `Derivation` with input_derivations referencing
  other derivations. crunch-build's `collect_input_paths()` resolves them
  via `KnownPaths`. If the store path format between glue and build diverges,
  nothing catches it.
- The CLI parses `--store`, `--verbose`, `-I` flags and threads them through
  to the pipeline. Untested.
- Multi-derivation output (package set mode) is detected by checking for a
  `name` field. This heuristic has no test.

## What Changes

- Add an integration test crate or `tests/` directory exercising the full
  CLI binary.
- Test `crunch eval` on `.ncl` files that use the stdlib.
- Test `crunch build` on trivial derivations (requires Linux + bwrap).
- Test `crunch bootstrap` produces importable seed files.
- Test multi-derivation output.
- Test error exit codes match the spec (1 build, 2 eval, 3 internal).
- Test `--json` error output is parseable JSON.

## Capabilities

### New Capabilities
- `test-e2e-eval`: End-to-end evaluation through the binary.
- `test-e2e-build`: End-to-end build of a trivial derivation.
- `test-e2e-bootstrap`: End-to-end bootstrap generating valid seed.ncl.
- `test-e2e-multi-drv`: Package set mode (multiple derivations from one file).
- `test-e2e-exit-codes`: Exit code correctness for all error classes.
- `test-e2e-json-errors`: `--json` flag produces parseable error objects.

## Impact

- **Files**: `tests/integration.rs` (new), possibly `tests/fixtures/` for
  `.ncl` test files
- **APIs**: None changed
- **Dependencies**: `assert_cmd` and `predicates` as dev-dependencies for
  CLI testing
- **Testing**: `cargo test --test integration` (some tests Linux-only)

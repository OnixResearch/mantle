## Why

mkDerivation exists but requires every phase to be written by hand.
A C package that uses autotools still needs explicit `configurePhase`,
`buildPhase`, and `installPhase` strings. The whole point of
mkDerivation is providing sensible defaults so authors only override
what differs from the standard build flow.

The `src` field is declared in the contract but the default unpack
phase doesn't exist — there's no standard way to extract a tarball
source and `cd` into it. `mkShell` is specced but unimplemented.

## What Changes

- **Default phase implementations in `mk_derivation.ncl`.** When a
  phase field is absent, `make_script` inserts the standard default:
  unpack (`cp -r` or `tar xf`), configure (`./configure --prefix=$out`
  if it exists), build (`make`), install (`make install`).
- **`mkShell` function.** Creates a derivation whose builder fails
  with a message ("this derivation is not meant to be built"). The
  record carries the environment for `crunch shell` (future).
- **`src` wiring.** When `src` is set, it's added to `inputs` and
  exported as `$src` in the environment. The default unpack phase
  uses it.
- **Integration tests.** E2E tests that build real packages through
  mkDerivation with default phases, verifying the output exists and
  contains expected files. Tests for mkShell, multi-output,
  output selection, and the new --no-substitute flag.

## Capabilities

### New Capabilities
- `default-phases`: standard unpack/configure/build/install defaults
- `mkshell`: development environment derivation constructor
- `src-wiring`: automatic src input + $src env var

### Modified Capabilities
- `mk-derivation`: gains default phase behavior when phases are absent

## Impact

- **Files**: `lib/mk_derivation.ncl`, `lib/lib.ncl`, `tests/integration.rs`
- **APIs**: no breaking changes — existing explicit phases still work
- **Dependencies**: none
- **Testing**: new integration tests for mkDerivation with defaults

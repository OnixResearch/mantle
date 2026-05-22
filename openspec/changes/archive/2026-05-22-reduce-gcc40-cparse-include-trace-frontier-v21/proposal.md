## Why

v20 showed that generated `config.h` ordering tweaks are not moving the real GCC 4.0 `c-parse.o` make frontier: prepending `<sys/types.h>` still fails with rc=2 and the same two truncated include-flood diagnostics. The next useful evidence is not another config-order tweak, but recovering bounded include-trace payload from the compiler path immediately before the flood.

## What Changes

- Add one compact diagnostic probe to the GCC 4.0 c-parse boundary derivation that runs the real c-parse compile command in preprocessor mode and summarizes emitted line markers.
- Preserve the v20 real make frontier and baseline `c-parse.o` failure markers.
- Update the source-frontier receipt and parity checks to schema v21, requiring the include-trace markers and keeping the claim diagnostic-only.
- Archive the change after verification.

## Scope

In scope:
- `bootstrap/diag-gcc40-c-parse-boundary.ncl`
- `bootstrap/evidence/gcc-4.0-native-cc1-build-frontier.json`
- `src/bootstrap_parity.rs`
- `openspec/specs/bootstrap/spec.md`

Out of scope:
- Production GCC correctness fixes.
- More generated-config ordering probes.
- Promotion of GCC 4.0 beyond partial/frontier evidence.

## Verification

- `openspec validate reduce-gcc40-cparse-include-trace-frontier-v21 --strict`
- diagnostic eval/build of `bootstrap/diag-gcc40-c-parse-boundary.ncl`
- focused parity tests/report
- source-pin and blocker-inventory checks
- `openspec validate --all --strict`
- `git diff --check`

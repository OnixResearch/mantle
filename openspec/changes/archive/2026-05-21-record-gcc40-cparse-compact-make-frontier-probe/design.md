# Design: compact GCC 4.0 c-parse make frontier

## Contract

The receipt schema `mantle-gcc40-native-cc1-source-frontier-reduction-v8` records that v5/v6 generated-header probes are archived evidence and the active diagnostic derivation is intentionally compact. `diagnostic_markers` must only contain source-resident markers that remain in `bootstrap/diag-gcc40-c-parse-boundary.ncl` after pruning.

## Implementation notes

- Keep the focused make invocation and rc/tail capture from v7.
- Add a compact source marker documenting that the archived broad c-parse matrix was pruned to avoid Linux `MAX_ARGSTRLEN`/`Argument list too long` failures.
- Remove source-bound requirements for individual archived v6 probe calls from `diagnostic_markers`; keep their outcomes in `observed_frontier`.
- Validator regressions must reject stale v7 schema and missing compact marker text.

## Verification

- JSON receipt check.
- Derivation eval and extracted script byte count below the known argument-size danger zone, plus `/bin/sh -n` on the extracted builder when available.
- Focused bootstrap parity tests and CLI tests.
- Blocker inventory/source-pin self-tests if derivation/evidence metadata changes.
- Parity report confirms `gcc.4.0` remains `partial`.
- OpenSpec strict validation, archive, `openspec validate --all --strict`, and `git diff --check`.

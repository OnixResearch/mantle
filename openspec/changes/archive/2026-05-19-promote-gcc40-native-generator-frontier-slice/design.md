## Context

Current bootstrap parity evidence already distinguishes several GCC 4.0 pass1 improvements from full native correctness: libgcc member semantics, driver query semantics, object-output `cc1`, bounded native `cc1` arithmetic/control-flow, and fail-closed native frontier receipts.

The current `bootstrap/evidence/gcc-4.0-native-boundary.json` still lists `generator-empty-boundaries` as a frontier blocker. The corresponding derivation markers in `bootstrap/gcc-4.0.ncl` describe checked empty generator outputs such as the `genattrtab` boundary. This change should promote exactly one selected generator member and keep the remaining generator family partial.

## Approach

1. Add a focused receipt for a bounded `genattrtab` native-generator output slice, or extend the native boundary receipt with a separately checked promoted slice entry if that keeps the evidence model simpler.
2. Require the receipt to name:
   - schema/version
   - derivation path
   - selected generator member (`genattrtab`)
   - exact source/derivation marker(s)
   - bounded output contract
   - transcript or digest evidence sufficient to detect stale generated output
   - explicit non-claim that GCC 4.0 remains partial
3. Update parity validation to fail closed when the receipt is missing, malformed, stale, or overclaims completion.
4. Add negative tests for missing receipt, marker drift, stale digest/transcript, unsupported schema, and accidental parity completion.
5. Leave the broader native frontier blocker in place for unpromoted generator members and any remaining native compiler correctness gaps.

## Non-goals

- Do not claim full native GCC 4.0 correctness.
- Do not complete `gcc.4.0` for live-bootstrap or Guix parity.
- Do not promote the entire generator family in one patch.
- Do not weaken existing boundary receipts or placeholder inventory checks.

## Risks

- The slice can become another boundary-label rename without new evidence. Mitigation: require a receipt with drift/negative tests and a bounded output contract.
- Generator output may depend on a larger GCC machine-description surface than expected. Mitigation: keep the selected contract narrow and fail closed rather than broadening the claim.
- Parity notes can imply completion accidentally. Mitigation: test that `gcc.4.0` remains `partial` and blocks live-bootstrap/Guix.

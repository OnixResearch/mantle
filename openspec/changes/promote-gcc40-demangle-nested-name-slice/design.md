## Context

Previous demangle work replaced a disabled `cp-demangle` boundary with a bounded zero-argument Itanium function slice. Current source evidence includes:

- `gcc40_cplus_demangle_bounded_itanium_v0_boundary`
- `gcc40_cp_demangle_bounded_itanium_v0_boundary`
- smoke coverage for `_Z3foov -> foo()` and `_Z3barv -> bar()`
- negative coverage for malformed or unsupported names returning null

After the generator-frontier drains, `bootstrap/evidence/gcc-4.0-native-boundary.json` now leaves `libiberty-demangle-bounded-semantics` as the remaining native-frontier blocker. The slice should advance that seam without broadening into full `cp-demangle` correctness.

## Decision

Promote one additional deterministic Itanium nested-name zero-argument function shape, such as:

- input: `_ZN3foo3barEv`
- output: `foo::bar()`

This is narrow enough for a deterministic parser/smoke and meaningfully stronger than the existing flat-name-only slice. It should be represented as a bounded semantic slice, not as full C++ demangler support.

## Implementation Notes

- Keep the functional core small and testable: parse exactly the supported nested-name form and return null for unsupported or malformed inputs.
- Preserve existing flat zero-argument behavior and negative cases.
- Add checked receipt/frontier metadata that records:
  - schema/status
  - selected demangle shape
  - derivation/source marker(s)
  - bounded input/output contract
  - smoke transcript and BLAKE3 digest or equivalent deterministic evidence
  - explicit `parity_effect` stating evidence-backed partial only
- Parity validation should fail closed on missing receipt, unsupported schema, missing marker, stale digest/transcript, unsupported selected shape, missing bounded contract, forbidden stale marker, or parity overclaim.
- Keep `gcc.4.0` `partial` and blocking for live-bootstrap and Guix.

## Non-Goals

- Full Itanium ABI demangling.
- General namespace/class/template/operator/argument demangling.
- Replacing GCC 4.0 pass1 bridge or proving full native GCC 4.0 correctness.
- Completing live-bootstrap, Guix, or StageX parity.

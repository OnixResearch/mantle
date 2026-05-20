## Context

Previous demangle work replaced disabled `cp-demangle` boundaries with bounded Itanium zero-argument function slices. Current source/evidence includes:

- flat zero-argument support such as `_Z3foov -> foo()`
- two-component nested support such as `_ZN3foo3barEv -> foo::bar()`
- checked marker `gcc40_cplus_demangle_nested_itanium_v1_boundary`
- negative coverage for malformed or unsupported names returning null
- `bootstrap/evidence/gcc-4.0-native-demangle-slice.json` and native-frontier receipt wiring

`bootstrap/evidence/gcc-4.0-native-boundary.json` still leaves `libiberty-demangle-bounded-semantics` as a native-frontier blocker. The slice should advance that seam without broadening into full `cp-demangle` correctness.

## Decision

Promote one additional deterministic deeper Itanium nested-name zero-argument function shape, such as:

- input: `_ZN3foo3bar3bazEv`
- output: `foo::bar::baz()`

This is narrow enough for a deterministic parser/smoke and meaningfully stronger than the existing two-component nested-name slice. It should be represented as a bounded semantic slice, not as full C++ demangler support.

## Implementation Notes

- Keep the functional core small and testable: parse exactly the supported flat/two-component/deeper nested zero-argument forms and return null for unsupported or malformed inputs.
- Preserve existing flat and two-component nested behavior and negative cases.
- Prefer static buffers and local comparison helpers in derivation smokes; do not introduce malloc/strcmp/free dependence unless that runtime path is separately proven.
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

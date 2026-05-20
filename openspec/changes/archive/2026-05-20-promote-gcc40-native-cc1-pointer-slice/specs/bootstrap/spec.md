# Bootstrap Specification Delta

## MODIFIED Requirements

### Requirement: GCC version ladder

Mantle MUST keep GCC version rows evidence-backed and fail-closed while preserving partial/blocking status until full native/source-built correctness evidence exists.

#### Scenario: GCC 4.0 native cc1 pointer-deref slice is bounded and non-promoting

- GIVEN `bootstrap/evidence/gcc-4.0-native-cc1-arithmetic.json` declares schema `mantle-gcc40-native-cc1-arithmetic-v7`
- AND the selected slice is `pointer-deref-v7`
- AND the bounded input contains a local `int`, a pointer initialized with `&value`, a dereference store through `*slot`, a dereference load from `*slot`, and a return expression using the loaded value
- WHEN `mantle bootstrap parity-report` checks the GCC 4.0 row
- THEN the row accepts the pointer-deref slice only if the derivation marker, object marker, transcript digest, output digest, and prior v1-v6 regression slices all match
- AND `gcc.4.0` remains partial/blocking with no native compiler/source-build correctness claim.

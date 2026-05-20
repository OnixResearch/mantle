## MODIFIED Requirements

### Requirement: GCC version ladder

GCC 4.0 native demangle promotions MUST be evidence-backed one bounded semantic slice at a time. A promoted native libiberty single-long-argument demangle slice MUST prove a selected bounded Itanium single-`long` function-argument shape with checked receipt evidence, while leaving arbitrary type decoding, full `cp-demangle`, and native GCC 4.0 correctness blocked. The slice MUST require checked source markers, a native demangle receipt, transcript/digest evidence, preserved regressions for prior zero-argument, single-`int`, and single-`char` slices, stale-marker denial for older demangle markers, and bounded non-claim wording.

#### Scenario: GCC 4.0 native demangle single-long-argument slice is accepted [r[bootstrap.gcc.version-ladder.gcc40-native-demangle-long-arg-slice]]

- GIVEN `bootstrap/gcc-4.0.ncl` contains checked bounded libiberty demangle semantic markers for the selected single-`long` Itanium shape
- AND a checked native-demangle receipt names schema `mantle-gcc40-native-demangle-slice-v5`, the selected shape, source markers, bounded input/output contract, preserved regressions, rejected unsupported shapes, and transcript digest evidence
- WHEN the bootstrap parity report evaluates the `gcc.4.0` row
- THEN the row may report evidence-backed `partial` with native demangle single-`long` argument slice evidence

#### Scenario: GCC 4.0 native demangle single-long-argument slice rejects stale boundary evidence [r[bootstrap.gcc.version-ladder.gcc40-native-demangle-long-arg-slice-drift]]

- GIVEN the native demangle receipt references stale v4 single-`char` boundary markers or omits the v5 long markers
- WHEN the bootstrap parity report evaluates the `gcc.4.0` row
- THEN the row MUST fail closed instead of accepting stale demangle evidence

#### Scenario: GCC 4.0 native demangle single-long-argument slice cannot complete GCC 4.0 parity [r[bootstrap.gcc.version-ladder.gcc40-native-demangle-long-arg-slice-no-overclaim]]

- GIVEN bounded libiberty demangle evidence exists for the selected single-`long` argument shape
- WHEN live-bootstrap, Guix, or StageX parity is required
- THEN `gcc.4.0` MUST remain `partial` until native compiler/generator correctness and broader `cp-demangle` evidence are complete

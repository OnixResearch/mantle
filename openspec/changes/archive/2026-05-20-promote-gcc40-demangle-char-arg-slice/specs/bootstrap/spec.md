## MODIFIED Requirements

### Requirement: GCC version ladder

GCC 4.0 native demangle promotions MUST be evidence-backed one bounded semantic slice at a time. A promoted native libiberty single-char-argument demangle slice MUST prove a selected bounded Itanium single-`char` function-argument shape with checked receipt evidence, while leaving arbitrary type decoding, full `cp-demangle`, and native GCC 4.0 correctness blocked. The slice MUST require checked source markers, a native demangle receipt, transcript/digest evidence, preserved regressions for prior zero-argument and single-`int` slices, stale-marker denial for older demangle markers, and bounded non-claim wording.

#### Scenario: GCC 4.0 native demangle single-char-argument slice is accepted [r[bootstrap.gcc.version-ladder.gcc40-native-demangle-char-arg-slice]]

- GIVEN `bootstrap/gcc-4.0.ncl` contains checked bounded libiberty demangle semantic markers for the selected single-`char` Itanium shape
- AND a checked native-demangle receipt names schema `mantle-gcc40-native-demangle-slice-v4`, the selected shape, source markers, bounded input/output contract, preserved regressions, rejected unsupported shapes, and transcript digest evidence
- WHEN the bootstrap parity report evaluates the `gcc.4.0` row
- THEN the row may report evidence-backed `partial` with native demangle single-`char` argument slice evidence

#### Scenario: GCC 4.0 native demangle single-char-argument slice rejects stale boundary evidence [r[bootstrap.gcc.version-ladder.gcc40-native-demangle-char-arg-slice-drift]]

- GIVEN the native demangle receipt references stale v3 single-`int` boundary markers or omits the v4 char markers
- WHEN the bootstrap parity report evaluates the `gcc.4.0` row
- THEN the row MUST fail closed instead of accepting stale demangle evidence

#### Scenario: GCC 4.0 native demangle single-char-argument slice cannot complete GCC 4.0 parity [r[bootstrap.gcc.version-ladder.gcc40-native-demangle-char-arg-slice-no-overclaim]]

- GIVEN bounded libiberty demangle evidence exists for the selected single-`char` argument shape
- WHEN live-bootstrap, Guix, or StageX parity is required
- THEN `gcc.4.0` MUST remain `partial` until native compiler/generator correctness and broader `cp-demangle` evidence are complete

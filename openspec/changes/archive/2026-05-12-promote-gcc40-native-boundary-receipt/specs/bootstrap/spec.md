## MODIFIED Requirements

### Requirement: GCC version ladder

GCC 4.0 pass1 libgcc member promotions MUST be evidence-backed one member at a time. The `__gcc_bcmp` member MUST implement byte-wise comparison semantics: it MUST return `0` for equal byte ranges and a non-zero value for the first differing byte over the requested length. The promotion MUST include a derivation-local smoke that exercises equal and unequal comparisons. GCC 4.0 pass1 driver query semantics MUST also be evidence-backed: `-dumpversion`, `-dumpmachine`, `-print-libgcc-file-name`, and `-print-search-dirs` MUST return deterministic GCC-shaped values tied to the installed artifact, and the derivation MUST verify the queried libgcc path exists. GCC 4.0 pass1 `cc1` object semantics MUST be evidence-backed: the installed `cc1` MUST accept a bounded GCC-shaped `-quiet <input> -o <object>` invocation, emit non-empty object through the validated TinyCC handoff, and be covered by derivation-local smoke. GCC 4.0 native-boundary evidence MUST be checked by a receipt that records the current boundary as `boundary-only`, proves the derivation still contains the native `make -C gcc` attempt, the pass1 bridge handoff diagnostic, and the installed bridge semantics, and keeps `gcc.4.0` partial until native `cc1` and broader compiler correctness are proven. These promotions MUST NOT mark `gcc.4.0` complete or unblock live-bootstrap/Guix parity by themselves.

#### Scenario: GCC 4.0 native boundary receipt is checked

- GIVEN the GCC 4.0 parity row is evaluated
- WHEN the native-boundary receipt is present
- THEN the receipt status is `boundary-only`
- AND the receipt-required derivation markers are present in `bootstrap/gcc-4.0.ncl`
- AND the `gcc.4.0` parity row remains partial until native `cc1` and broader compiler correctness are proven

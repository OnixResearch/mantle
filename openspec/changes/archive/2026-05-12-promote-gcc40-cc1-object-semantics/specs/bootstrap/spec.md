## MODIFIED Requirements

### Requirement: GCC version ladder

GCC 4.0 pass1 libgcc member promotions MUST be evidence-backed one member at a time. The `__gcc_bcmp` member MUST implement byte-wise comparison semantics: it MUST return `0` for equal byte ranges and a non-zero value for the first differing byte over the requested length. The promotion MUST include a derivation-local smoke that exercises equal and unequal comparisons. GCC 4.0 pass1 driver query semantics MUST also be evidence-backed: `-dumpversion`, `-dumpmachine`, `-print-libgcc-file-name`, and `-print-search-dirs` MUST return deterministic GCC-shaped values tied to the installed artifact, and the derivation MUST verify the queried libgcc path exists. GCC 4.0 pass1 `cc1` object semantics MUST be evidence-backed: the installed `cc1` MUST accept a bounded GCC-shaped `-quiet <input> -o <object>` invocation, emit non-empty object through the validated TinyCC handoff, and be covered by derivation-local smoke. These promotions MUST NOT mark `gcc.4.0` complete or unblock live-bootstrap/Guix parity by themselves.

#### Scenario: GCC 4.0 cc1 emits object for a bounded C input

- GIVEN the GCC 4.0 pass1 artifact is installed
- WHEN the installed `cc1` is invoked with `-quiet /tmp/input.c -o /tmp/input.o`
- THEN the command succeeds
- AND `/tmp/input.o` is non-empty
- AND the `gcc.4.0` parity row remains partial until native `cc1` and broader compiler correctness are proven

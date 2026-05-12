## MODIFIED Requirements

### Requirement: GCC version ladder

GCC 4.0 pass1 libgcc member promotions MUST be evidence-backed one member at a time. The `__gcc_bcmp` member MUST implement byte-wise comparison semantics: it MUST return `0` for equal byte ranges and a non-zero value for the first differing byte over the requested length. The promotion MUST include a derivation-local smoke that exercises equal and unequal comparisons. GCC 4.0 pass1 driver query semantics MUST also be evidence-backed: `-dumpversion`, `-dumpmachine`, `-print-libgcc-file-name`, and `-print-search-dirs` MUST return deterministic GCC-shaped values tied to the installed artifact, and the derivation MUST verify the queried libgcc path exists. GCC 4.0 pass1 `cc1` object semantics MUST be evidence-backed: the installed `cc1` MUST accept a bounded GCC-shaped `-quiet <input> -o <object>` invocation, emit non-empty object through the validated TinyCC handoff, and be covered by derivation-local smoke. GCC 4.0 native-boundary evidence MUST be checked by a receipt that records the current boundary as `boundary-only`, proves the derivation still contains the native `make -C gcc` attempt, the pass1 bridge handoff diagnostic, and the installed bridge semantics, and keeps `gcc.4.0` partial until native `cc1` and broader compiler correctness are proven. GCC 4.0 generator-header boundary promotions MUST be derivation-checked: the `genconstants`/`genflags` seed path MUST emit guarded empty-machine boundary headers, MUST reject the prior generic stub labels, and MUST NOT mark real native generator correctness complete. GCC 4.0 `gencheck` boundary promotions MUST be derivation-checked: the `gencheck` path MUST emit a guarded disabled-tree-checking boundary header, MUST reject the prior generic `gencheck` stub label, and MUST NOT mark real native `gencheck` correctness complete. These promotions MUST NOT mark `gcc.4.0` complete or unblock live-bootstrap/Guix parity by themselves.

#### Scenario: GCC 4.0 gencheck emits checked disabled-tree-checking boundary

- GIVEN the GCC 4.0 pass1 derivation bridges `gencheck` before native generator correctness is complete
- WHEN `tree-check.h` is emitted
- THEN it contains the `GCC_TREE_CHECK_H` include guard
- AND it contains a disabled-tree-checking boundary marker
- AND it does not contain the prior generic `gencheck` stub label
- AND the `gcc.4.0` parity row remains partial until real native generator and compiler correctness are proven

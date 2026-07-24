## ADDED Requirements

### Requirement: Configure preprocessing bridge confinement

r[gcc40_bridge.configure_preprocess_confinement] Mantle MUST confine the provisional GCC 4.0 preprocessing bridge to bounded Autoconf probes under explicit build-local authority and MUST reject any attempt to use it as a general or provider preprocessor.

#### Scenario: Declared configure probe is admitted

GIVEN the GCC 4.0 predecessor invokes `-E` for canonical `conftest.c` in the explicitly active `libiberty`, `libcpp`, or `gcc` configure directory
AND the invocation has no explicit output, fits the source-byte bound, and remains within the invocation-count bound
WHEN the wrapper evaluates the request
THEN it MUST record the admitted class and canonical source in the build-local audit before invoking the declared source-built compiler
AND the parent build MUST verify the bounded audit after all configure phases.

#### Scenario: Preprocessing authority escape fails closed

GIVEN a preprocessing request has missing authority, a different current directory, a path or symlink escape, a non-`conftest.c` source, an explicit output, an oversized source, an unknown configure class, or an exhausted invocation budget
WHEN the wrapper evaluates the request
THEN it MUST fail before invoking any compiler or emitting successful preprocessing output
AND it MUST NOT fall back to another compiler, host preprocessor, or unbounded source transformation.

#### Scenario: Bridge claim remains provisional

GIVEN the bounded configure bridge and its checker pass
WHEN bootstrap parity or operator evidence describes the result
THEN Mantle MUST continue classifying the mechanism as predecessor frontier debt with an explicit retirement condition
AND it MUST NOT claim provider eligibility, general preprocessing correctness, native GCC 4.0 correctness, seed trust removal, or independent reproducibility.
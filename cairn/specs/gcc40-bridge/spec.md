# Gcc40 Bridge Specification

## Purpose

Defines the `gcc40-bridge` capability.

## Requirements

### Requirement: GCC 4.0 native cc1 source-frontier reduction

r[gcc40_bridge.source_frontier_reduction] Mantle MUST treat each GCC 4.0 native `cc1` source-frontier reduction as a bounded evidence slice that may narrow a pass1 bridge blocker but MUST NOT claim full native GCC 4.0 compiler correctness.

#### Scenario: selected frontier is accepted

GIVEN `bootstrap/gcc-4.0.ncl` or a focused diagnostic derivation contains exact markers for one selected native `cc1` source-build frontier
AND checked evidence names the prior frontier, attempted probe, observed result, transcript digest, exact source markers, partial-only parity effect, and retirement condition
WHEN the bootstrap parity report evaluates `gcc.4.0`
THEN the row MAY report an evidence-backed partial source-frontier reduction
AND it MUST continue blocking live-bootstrap, Guix, and StageX until full native GCC 4.0 source-build and compiler-correctness evidence exists.

#### Scenario: stale or delegated evidence is rejected

GIVEN the source-frontier evidence is stale, omits the selected marker, has digest drift, delegates the selected probe to TinyCC, or broadens beyond the selected frontier
WHEN the parity report evaluates `gcc.4.0`
THEN the row MUST remain blocked
AND the diagnostic MUST name the failed source-frontier evidence check.

#### Scenario: bridge markers stay bounded

GIVEN pass1 bridge markers remain in `bootstrap/gcc-4.0.ncl`
WHEN a bounded source-frontier reduction is accepted
THEN Mantle MUST keep the bridge markers classified as checked frontier debt
AND it MUST NOT remove blocker-inventory visibility until replacement evidence retires the bridge condition.

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

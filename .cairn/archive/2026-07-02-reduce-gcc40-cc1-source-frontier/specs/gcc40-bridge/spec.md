## ADDED Requirements

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

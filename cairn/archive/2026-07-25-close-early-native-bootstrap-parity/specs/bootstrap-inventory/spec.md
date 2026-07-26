## ADDED Requirements

### Requirement: Early native bootstrap parity is artifact-backed

r[bootstrap_inventory.early_native_bootstrap_parity] Mantle MUST mark the `binutils.tcc` and `gcc.4.0` bootstrap parity rows complete only from independently validated current source-built artifacts and BLAKE3-bound receipts, never from later-provider success, metadata shape, version output, or compatibility-bridge prose.

#### Scenario: TCC binutils handoff is complete

GIVEN authenticated TCC-era sources and predecessor tools
WHEN Mantle builds and evaluates the `binutils.tcc` row
THEN the declared assembler, linker, archive, ranlib, nm, objcopy, object-format, and relocation surfaces MUST be real source-built outputs with positive and malformed-input behavior evidence
AND omitted tools, predecessor delegation, host fallback, incomplete archive members, or stale receipt identities MUST keep the row blocked.

#### Scenario: GCC 4.0 native handoff is complete

GIVEN the admitted early binutils handoff and authenticated regenerated GCC 4.0 sources
WHEN Mantle builds and evaluates the `gcc.4.0` row
THEN the C and C++ drivers, compiler internals, required generators, demangler, libgcc and exception/runtime artifacts MUST be built from declared sources and pass the receipt-defined positive and rejection matrix
AND TinyCC delegation, fabricated objects, release-generated substitution, missing compiler internals, or use of the configure preprocessing bridge beyond its audited authority MUST keep the row blocked.

#### Scenario: row evidence is not interchangeable

GIVEN one early row has complete evidence and the other is absent, stale, malformed, or incomplete
WHEN the bootstrap parity report is generated
THEN Mantle MUST promote only the independently complete row
AND axis status MUST retain the incomplete row as a blocker with a deterministic reason.

#### Scenario: early parity claims remain bounded

GIVEN both early rows are complete
WHEN the evidence is cited
THEN the claim MUST identify sources, predecessors, generated artifacts, outputs, behavior tests, rejection tests, and fallback results
AND it MUST NOT claim general compiler, assembler, linker, seed, or bootstrap correctness.
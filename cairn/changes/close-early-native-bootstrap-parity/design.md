## Context

`binutils.tcc` has a checked smoke transcript but remains partial because bridge or omitted-member output is not a complete native binutils handoff. GCC 4.0 has genuine bounded native slices and an explicitly confined configure preprocessing bridge, but current parity evidence still records unresolved generator/compiler/demangler correctness and placeholder debt. Later provider admission cannot retroactively prove these predecessor artifacts.

## Decisions

### Decision: use output-derived completion criteria

**Choice:** Define each row's admission from produced binaries/libraries, dependency closure, source-generation receipts, and semantic tests. Do not mark a row complete because a later compiler builds or because a contract file contains expected text.

**Rationale:** Later success can mask delegation, generated stubs, or omitted tools in an earlier stage.

### Decision: preserve the bounded configure bridge but exclude it from compiler semantics

**Choice:** The audited GCC 4.0 configure preprocessing bridge may remain only within its existing bounded probe authority. Compiler objects, generators, demangler behavior, libgcc, and C/C++ outputs must come from declared source builds and may not be synthesized by that bridge.

**Rationale:** The existing bridge is an explicit predecessor bootstrap concession, not evidence of GCC semantics.

### Decision: require independent row receipts

**Choice:** Emit separate receipts for TCC-built binutils and GCC 4.0. Each receipt binds source records, predecessor identities, generated-source identities, output digests, runtime tests, rejection tests, and forbidden-marker scans.

**Rationale:** Failure or incompleteness in one row must not be hidden by a combined provider receipt.

### Decision: define bounded native correctness without claiming compiler soundness

**Choice:** Row completion requires the full declared artifacts plus a reviewable behavioral matrix covering ordinary compile/assemble/link/archive/inspect flows and negative malformed-input behavior. Documentation must continue to disclaim general compiler, assembler, and linker correctness.

**Rationale:** “Compiler correctness” is not established by finite smoke tests, but parity still needs stronger executable evidence than metadata or version output.

## Risks / Trade-offs

- GCC 4.0 source regeneration can reopen Perl/autotools/parser-generator failures; triage must stop at the first predecessor failure.
- Expanding the behavior matrix can reveal real unsupported features; those remain blockers rather than candidates for wrapper delegation.
- Row promotion must not weaken the existing source-built-provider admission contract.
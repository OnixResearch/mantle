# Design: musl 1.1.24 tcc runtime validation

## Context

The parent derivation now fails closed if required musl outputs are missing. Actual runtime proof still depends on earlier bootstrap executables, including make 3.82 and TinyCC amd64 repair work.

## Decisions

### 1. Defer runtime proof until prerequisites are proven

**Choice:** track build/smoke/leakage validation in this follow-up rather than treating the hardened source-level derivation as runtime proof.

**Rationale:** musl pass1 is downstream of make/tcc execution, so runtime evidence must be collected after those repairs land.

### 2. Validate the first-musl contract directly

**Choice:** smoke tests must check `lib/libc.a`, installed headers, and at least one startup object (`crt1.o` or `Scrt1.o`).

**Rationale:** later compiler/libc stages depend on these specific artifacts.

## Validation

Build `bootstrap/musl-1.1.24-tcc.ncl`, inspect output contract, scan for leakage, and run OpenSpec validation/gates.

### 3. Close current drain as checkpointed prerequisite timeout

**Choice:** archive this runtime-validation follow-up as an explicit incomplete/hung validation attempt rather than treating the source-level derivation as runtime proof.

**Rationale:** the prerequisite make/TinyCC chain is now archived enough to attempt musl, but the musl validation runner remained in Mes prerequisite work past the local drain budget and produced no musl output. Archiving the blocker preserves the exact checkpoint evidence and prevents accidental promotion.

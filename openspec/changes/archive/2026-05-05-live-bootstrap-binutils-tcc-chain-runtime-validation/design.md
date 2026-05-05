# Design: Binutils-tcc runtime validation

## Context

The parent drain attempt proved that the local environment can satisfy `crunch doctor` once `bubblewrap` is present on PATH. The remaining issue is validation runtime: `bootstrap/bzip2-tcc.ncl` spends more than the local drain budget building the Mes prerequisite before any epoch transcript can complete.

## Decisions

### 1. Use a longer-running validation runner

**Choice:** run V2 epoch builds under a durable background runner with explicit state/store directories and per-root logs.

**Rationale:** the Mes prerequisite is CPU-heavy and may be legitimate work rather than a hang.

### 2. Split prerequisite roots if needed

**Choice:** if the first epoch still cannot complete in one run, build/cache prerequisite roots such as `mes` separately where the current `.ncl` graph exposes them, then resume the epoch chain.

**Rationale:** separating prerequisite proof from epoch proof gives useful evidence without hiding runtime failures.

## Validation

Complete the parent V2-V5 evidence set: epoch transcript table, host-leakage scan, musl linkage checks, and binutils assembler smoke.

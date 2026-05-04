# Design: rebuilt musl 1.1.24 runtime validation

## Context

The parent derivation now fails closed if required rebuilt-musl outputs are missing. Runtime proof still depends on upstream bootstrap runtime validation for tcc/make/first-musl stages.

## Decisions

### 1. Defer runtime proof until prerequisites are proven

**Choice:** track build/smoke/leakage validation in this follow-up rather than treating source-level hardening as runtime proof.

**Rationale:** the rebuilt musl pass consumes earlier runtime outputs and must not be promoted independently of them.

### 2. Validate rebuilt-musl contract directly

**Choice:** smoke tests must check `lib/libc.a`, installed headers, and at least one startup object (`crt1.o` or `Scrt1.o`).

**Rationale:** downstream tcc-musl-v2 and post-musl tools depend on a complete static libc install.

## Validation

Build `bootstrap/musl-1.1.24-tcc-musl.ncl`, inspect output contract, scan for leakage, and run OpenSpec validation/gates.

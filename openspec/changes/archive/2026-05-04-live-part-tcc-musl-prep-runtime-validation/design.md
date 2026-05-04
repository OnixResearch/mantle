# Design: tcc musl prep runtime validation

## Context

The parent derivation now checks both installed bridge compiler names and Mes carry-forward artifacts. Runtime proof still depends on prerequisite bootstrap runtime validation for make and TinyCC amd64 behavior.

## Decisions

### 1. Defer runtime proof until prerequisites are proven

**Choice:** track build/smoke/leakage validation in this follow-up while the parent archives source-level hardening.

**Rationale:** musl-prep consumes earlier runtime outputs and must not be promoted independently of them.

### 2. Validate bridge contract

**Choice:** validation must prove installed `tcc`, `tcc-musl-prep`, carried Mes libc, carried Mes headers, and `tcc -v`.

**Rationale:** the first musl pass depends on this bridge compiler and Mes carry-forward surface.

## Validation

Build `bootstrap/tcc-musl-prep.ncl`, smoke `tcc -v`, inspect Mes carry-forward artifacts, scan for leakage, and run OpenSpec validation/gates.

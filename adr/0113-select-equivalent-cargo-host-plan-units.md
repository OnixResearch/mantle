# ADR 0113: Select only equivalent Rust-plan host units for Cargo graphs

- **Status:** Proposed
- **Date:** 2026-10-11

## Context

The opt-in Cargo-unit lane uses Cargo's offline unit graph for topology and
feature partitions and Rust-plan derivations for bounded compiler inputs. A
single package's host build-script target can occur more than once in the
Rust-plan receipt, with unified planner features but separate Cargo feature
partitions. Matching only by package, target and host role is ambiguous;
requiring exact planner feature equality also rejects valid Cargo graphs.

## Decision drivers

- Cargo, not Rust-plan, owns this lane's feature and dependency decisions.
- Different compiler arguments or package facts must not be silently mapped to
  one chosen host unit.
- Receipt ordering must not affect dynamic-plan identity.
- The separate host-executed `mantle rust-plan` lane retains its own semantics.

## Decision

First match the exact Cargo package ID, target name and kind, and execution
role. A single candidate is accepted. With multiple candidates, prefer a
unique exact Cargo feature match. Otherwise compare every candidate's compiler
arguments after the lane's existing Cargo-owned substitutions: source path,
feature flags, direct `--extern` bindings, metadata hash, and declared linker.
Also require equal `CARGO_PKG_*` package facts. Choose the lowest stable
planner unit ID only when these inputs are equivalent; reject divergent or
unresolvable candidates with `unit-plan-mode-unsupported`.

The choice does not import the planner's dependency edges or feature
selection; Cargo's unit graph still supplies both. Native inputs for build
scripts remain keyed by the exact Cargo package ID, not by crate basename or
version alone.

## Consequences

Duplicate host plan instances with equivalent compiler inputs can share the
Cargo-directed compilation boundary without receipt-order dependence. A
future difference outside the explicitly normalized Cargo-owned inputs remains
a typed blocker; this decision does not assert general Cargo parity,
cross-target correctness, or equivalence of arbitrary Rust-plan units. The
regression fixture covers stable candidate order and a divergent package fact;
the signed real-graph replay separately exercises host build scripts.

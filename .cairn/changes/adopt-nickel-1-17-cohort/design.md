# Design: Adopt the Nickel 1.17 evaluator cohort

## Context

Mantle uses Nickel in its evaluation core, command-line checks, vendored bootstrap source, examples, and release evidence.

The current embedded cohort is `nickel-lang 2.0.0` with `nickel-lang-core 0.16.1`. The current command-line tool is Nickel `1.16.0`.

## Decisions

### Decision: Update embedded and command-line evaluators together

**Choice:** Pin `nickel-lang 2.2.0`, `nickel-lang-core 0.18.0`, and Nickel CLI `1.17.0` as one cohort.

**Rationale:** Mantle must not validate sources with one evaluator and execute them with another unreviewed cohort.

### Decision: Use the existing vendoring authority

**Choice:** Refresh Nickel source through Mantle's repository-owned vendor process. Record the upstream commit, file manifest, checksums, and licenses.

Do not edit normalized vendored manifests as the source of truth.

**Rationale:** Vendored bootstrap inputs require reproducible provenance and preserved notices.

### Decision: Preserve the evaluation boundary

**Choice:** Keep source parsing, direct deserialization, diagnostic rendering, resource budgets, and worker teardown in their current owners.

API changes in Nickel must be adapted at Mantle's evaluation boundary. They must not leak into build or store cores.

**Rationale:** The version update does not transfer build, store, or sandbox authority to Nickel.

### Decision: Compare stable outcomes

**Choice:** Compatibility tests compare decoded values, stable error classes, redaction, bounds, and selected diagnostic context.

Tests must not bind complete incidental diagnostic text unless that text is a public Mantle contract.

**Rationale:** Upstream diagnostics can improve without changing Mantle's stable error meaning.

### Decision: Keep bootstrap evidence exact

**Choice:** Bootstrap and release evidence record the source commit, crate versions, CLI version, Rust requirement, and vendor manifest identity.

A passing evaluator test does not prove bootstrap correctness or fixed-point reproducibility.

## Flow

```text
Nickel 1.17.0 source
  -> repository-owned vendor refresh
  -> embedded evaluator adapters
  -> CLI and fixture evaluator
  -> positive and negative evaluation rails
  -> bootstrap and release evidence
```

## Risks and trade-offs

- The new core API can require adapter changes around diagnostics or deserialization.
- The upstream Rust requirement can affect bootstrap toolchain inputs.
- Vendor refresh can change a large source closure. The manifest and license audit must remain exact.
- Evaluation success does not prove derivation correctness, build hermeticity, or output trust.

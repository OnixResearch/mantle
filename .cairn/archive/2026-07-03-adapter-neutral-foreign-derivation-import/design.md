## Context

The GuixPkgs pattern is useful because it treats a package frontend as a derivation graph producer. The same boundary applies to Nix: a Nix evaluator can produce `.drv` files or derivation JSON, but the consuming build system should not need to evaluate Nix expressions, speak flake outputs, apply overlays, understand Guix package modules, or hard-code a package universe. It should consume a bounded graph artifact with explicit provenance and policy.

This change deliberately separates three layers:

1. **Foreign frontend generator shell**: optional tooling that may invoke Guix time-machine, Nix derivation export, another package frontend, or fixture data to produce raw derivation graph facts.
2. **Adapter-neutral translation core**: pure logic over in-memory graph facts and policy. This is reusable outside Mantle.
3. **Host build-tool adapter**: thin shell that maps accepted translated graph artifacts into Mantle build plans, source bundles, store imports, or another consumer's build API.

## Decisions

### 1. The core artifact is a foreign derivation graph IR, not a flake

**Choice:** Define a versioned `foreign-derivation-graph-v1` IR and `foreign-package-index-v1` by-name index. Nix flakes, Guix package modules, overlays, and Onix package selection remain producer/consumer adapter concerns.

**Rationale:** Flakes and overlays are Nix frontend surfaces. Treating them as the stable ABI would couple this work to Nix composition semantics instead of the lower derivation graph insight. Nix can still be a producer by exporting concrete derivation graph facts before the adapter-neutral import core runs.

### 2. The IR is graph facts plus small companion artifacts

**Choice:** `foreign-derivation-graph-v1` contains only canonical graph facts: schema version, producer/provenance summary, source store prefixes, target prefix when already known, root derivation IDs, bounded nodes, source payload descriptors, and unsupported feature records. Each node records a stable node ID, original derivation identity, name, system, builder, args, environment, output declarations, input derivation edges, source/input refs, fixed-output metadata, builtin operation identifiers, declared references, sandbox capability needs, and unsupported feature classes. `foreign-package-index-v1` and `foreign-derivation-import-receipt-v1` are companion artifacts, not executable frontend code.

**Rationale:** Naming the shape prevents implementers from treating Mantle `.ncl`, Nix expressions, flakes, overlays, or Guix modules as the IR. Policy, acquisition, and realization stay outside the graph facts unless lowered into explicit data fields.

### 3. Translation is functional core; acquisition and realization are shells

**Choice:** The translation core receives graph nodes, source payload descriptors, store-prefix rewrite policy, builtin mappings, fetch policy, sandbox compatibility policy, and package-index facts as data. It returns a canonical translated graph, deterministic diagnostics, and a receipt. It performs no filesystem, network, process, environment, clock, or store access.

**Rationale:** This keeps the logic testable without Guix, Nix, Mantle, network, or writable stores. Generators and consumers can be swapped without changing the core semantics.

### 4. Store-prefix rewriting is explicit and audited

**Choice:** Rewrites require declared source and target store prefixes, per-field rewrite rules, embedded source-payload rewrite permissions, and output-path recomputation mode. Unknown foreign store references fail closed unless explicitly modeled.

**Rationale:** Silent path rewriting is the highest-risk part of foreign graph import. Prefix policy must be reviewable and receipt-bound.

### 5. Receipts bind provenance but do not overclaim

**Choice:** Import receipts bind producer identity, channel or source revision facts, root derivations, raw graph digest, translation policy digest, translated graph digest, package index digest when present, fetch/cache policy, sandbox compatibility allowances, and disabled-test policy. They explicitly do not claim build success, correctness, bootstrap parity, or reproducibility.

**Rationale:** A translated graph is useful build input material, not proof that the resulting package is correct or reproduced.

### 6. Cache and mirror support is policy data

**Choice:** Fixed-output source fetchers can carry ordered mirror candidates and content refs. Binary cache/substitution hints can be recorded as trust-scoped metadata, but acceptance remains under the existing store/substitution trust model.

**Rationale:** Whole foreign bootstrap closures are expensive. Usability needs caches, but import metadata must not bypass output trust.

### 7. Sandbox compatibility is per-derivation capability data

**Choice:** Compatibility exceptions such as Guix bootstrap setuid chmod behavior are modeled as explicit per-derivation sandbox capabilities with audit output. They are not hidden global impurity knobs.

**Rationale:** Foreign graphs may need compatibility affordances, but those affordances must remain visible and bounded.

## Risks / Trade-offs

- A generic IR may initially feel heavier than emitting Mantle `.ncl`; the payoff is that adapters stay replaceable.
- Faithful graph import can surface sandbox gaps that ordinary Mantle derivations do not need.
- Package-set scale requires indexing and cache work before it is operator-friendly.
- Some foreign frontend semantics may not round-trip into derivation graph facts; those must become explicit blockers, not hidden coupling.
- Nix derivation import may be tempting to shortcut through ambient `nix-store` or flake evaluation during consumption; boundary tests must reject that coupling.

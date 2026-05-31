# Design: Onix module layer over Mantle build tool

## Context

Mantle currently has a phase-1 `system` command path that resembles a small system configuration/module evaluator. It uses a Mantle-owned inventory shape:

```nickel
{ machines, services.<module>.instances[] }
```

That shape is not Onix, and more importantly it is not where module semantics should live. Mantle's intended role is closer to Nix as a build tool: evaluate build expressions, realize derivations/build plans, manage store state, and report build outcomes.

Onix uses its own module language and inventory model:

```nickel
{ machines, instances.<name>.service, instances.<name>.roles.<role> }
```

Onix owns tag expansion, per-tag settings, package/artifact selection, provider validation, upstream export shape, role interface defaults/contracts, and real service `impl` calls. Those are the Onix module layer, not Mantle build-tool responsibilities.

## Decisions

### 1. Mantle stays a build tool, not a module system

**Choice:** Mantle will not define or own a generic module ABI, module invocation unit schema, role/settings merge semantics, provider/export topology, or NixOS-like assembly layer for Onix.

**Rationale:** Those concepts belong to the module layer above the build tool. Putting them in Mantle repeats the NixOS/Nix layering mistake and makes Mantle a second Onix evaluator.

### 2. Onix owns the module layer and lowers to Mantle

**Choice:** Onix, or an Onix-owned adapter/tool, will evaluate Onix modules, expand inventory roles/tags, merge and validate settings through Nickel, call real module `impl` functions, collect exports/providers, and lower the result into concrete Mantle build inputs.

**Rationale:** Onix is the product/module language. It has the context needed to preserve its contracts and diagnostics without leaking policy into Mantle.

### 3. Mantle exposes only frontend-neutral build surfaces

**Choice:** The stable Mantle integration target should be build-tool shaped: derivation records, build plans, source inputs, store operations, build reports, and diagnostics. Any Onix-specific data reaching Mantle must be opaque build input data or already compiled into derivations/artifacts.

**Rationale:** This keeps Mantle useful to Onix and other frontends without hard-coding a system configuration model.

### 4. The current `system eval` scaffold is not the Onix path

**Choice:** The existing `crates/crunch-system` / `src/system_cmd.rs` path should not be extended into an Onix bridge. Before deployable artifact work depends on it, it should be marked experimental/demo-only, moved to an external layer, or removed. Its synthetic `JsonEvalBoundary` behavior must not be used as proof of module evaluation.

**Rationale:** The path looks like a module layer inside Mantle and already fakes implementation output. Continuing there would make artifact work package a fake config surface.

## Risks / Trade-offs

- Removing module-layer scope from Mantle means Onix needs its own reviewed adapter/lowering implementation before it can use Mantle end to end.
- Mantle may still need small build-tool API improvements for external frontends, but those improvements must not introduce roles/tags/providers/settings semantics.
- Existing `mantle system` examples may confuse operators; documentation must state clearly that they are not the Onix integration layer.

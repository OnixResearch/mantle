# ADR 0057: Keep composition plans concrete and frontend-neutral

## Status

Proposed

## Context

Mantle can identify immutable castore objects and directory trees. It does not have a generic way to combine several roots into one filesystem root.

A higher-level frontend can use such a root for systems, development environments, containers, tests, or recovery images. That does not make frontend policy part of composition.

OnixOS must continue to own package selection, provider policy, activation, deployment, and rollback. Kamacite may provide Preserves interchange without becoming a Mantle runtime dependency.

ADR 0012 composes store backends under one logical store prefix. It does not construct one immutable filesystem tree from several object roots.

## Decision Drivers

- Reuse immutable program objects in several filesystem environments without rebuilding unchanged bytes.
- Preserve Mantle's frontend-neutral build-tool boundary.
- Compute stable identities across equivalent input formats and list orders.
- Expose conflicts instead of hiding them through ordered overlay behavior.
- Keep castore identity, frontend intent identity, evidence identity, and deployment history separate.
- Introduce the smallest useful experiment before path-free execution or system deployment.

## Decision

Mantle will define an experimental concrete composition plan over exact castore root references, normalized root-relative mount points, and explicit path collision decisions. A separate realization policy will contain named limits and admission settings.

The core will derive stable binding references from root references and mount targets. Caller labels will remain display data.

A pure core will compute a domain-separated BLAKE3 `plan_ref` from normalized merge semantics. Raw transport bytes, list order, store prefixes, resource limits, caller labels, display metadata, and source envelopes will not affect that identity.

The core will compute a separate `realization_policy_ref` for limits and admission settings.

The castore directory root object reference will remain the filesystem identity. A deterministic realization receipt will bind the plan, realization policy, and merge outcomes to that root.

Binding order and caller labels will not imply precedence. Directories can merge recursively, and identical leaves can deduplicate. Every non-identical leaf collision will require one exact decision that selects a derived binding reference.

A thin shell will load complete bounded castore facts, persist planned directory objects, recheck the result, and emit the receipt. Store access and other effects will remain outside the pure core.

Mantle core will not require OnixOS, Kamacite, Preserves, OCI, Nix-compatible paths, or a specific frontend encoding. An optional adapter may map another format into the same core model.

The first change will stop at root realization, inspection, and export. Path-free action results, composed-root execution, Kamacite mapping, and OnixOS lowering require later Cairn changes after the core contract is stable.

## Alternatives Considered

### Put OnixOS intent in the Mantle plan

Rejected. It would move package, provider, machine, deployment, and rollback semantics below their owner.

### Make Preserves the required Mantle input

Rejected. Preserves is useful interchange, but a required codec would couple Mantle core to one adapter and identity domain.

### Use ordered last-writer-wins overlays

Rejected. Input order would become hidden replacement policy and could hide conflicts or provider changes.

### Hash the raw plan file or caller labels

Rejected. Equivalent JSON, Preserves, Rust, reordered, or relabeled projections would receive different cache keys.

### Define a new digest over the realized tree

Rejected. The castore Merkle root already identifies the supported filesystem facts. Another digest would create an unnecessary competing identity.

### Add execution and deployment in the first change

Rejected. Current action-result, build-request, execution-profile, and Unix metadata limits need separate compatibility and safety reviews.

## Consequences

Mantle gains a reusable object-composition primitive without becoming a system manager. Frontends can evolve independently and can share composition cache entries when they lower to the same concrete plan.

The plan and realization-policy formats must remain versioned and bounded. Explicit conflict decisions add work for callers, but they make replacements inspectable and deterministic.

The initial root cannot claim full Unix filesystem fidelity, ABI compatibility, dependency completeness, safe execution, bootability, deployment success, or release eligibility.

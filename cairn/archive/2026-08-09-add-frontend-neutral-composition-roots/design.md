# Design: Frontend-neutral composition roots

## Context

Mantle already separates immutable objects, action-result discovery, and execution. Castore directory roots identify exact trees, but Mantle has no generic operation that combines several roots.

A composition mechanism is useful only if Mantle receives concrete object facts. Package selection and system policy must remain above the build-tool boundary.

## Goals

- Combine exact immutable object roots without rebuilding their contents.
- Produce the same plan and root identities from equivalent frontend projections.
- Make every conflict and limit decision explicit and deterministic.
- Keep policy and transport adapters outside the composition core.

## Decisions

### Decision 1: Accept only concrete composition facts

**Choice:** The core plan contains a schema version, exact root object references, root-relative mount points, and explicit collision decisions. A separate realization policy contains named limits and admission settings.

The core derives each `binding_ref` from the exact root reference and normalized mount target. Caller labels are optional display data and cannot select collisions or enter identity.

The plan does not contain package constraints, provider searches, machine roles, tags, deployment generations, activation rules, or rollback policy.

**Rationale:** Mantle can validate and realize exact object placement. A frontend must resolve higher-level intent before it calls Mantle.

### Decision 2: Keep plan identity independent of transport

**Choice:** A pure core normalizes the merge semantics and computes `plan_ref` with domain-separated BLAKE3 canonical bytes.

Normalization sorts derived binding references and collision decisions by their semantic keys. Raw JSON bytes, Preserves bytes, source-envelope identities, caller labels, physical paths, resource limits, and list order do not enter `plan_ref`.

The core computes a separate `realization_policy_ref` over named limits and admission settings. A receipt binds both identities.

**Rationale:** Equivalent Rust, JSON, Nickel-derived, or Preserves-derived projections must not create different composition cache keys. Different resource budgets must remain visible without changing the requested tree semantics.

### Decision 3: Use the castore root as filesystem identity

**Choice:** The realized composition identity is the resulting castore directory root object reference. Mantle does not add a second digest over the same tree.

A separate receipt binds `plan_ref`, `realization_policy_ref`, input root references, the merge-policy version, conflict outcomes, limit usage, and the resulting root reference.

**Rationale:** The castore Merkle root already commits to supported path, file-content, executable-bit, directory, and symlink-target facts.

### Decision 4: Make object loading a thin shell

**Choice:** The shell loads a bounded, complete snapshot of every referenced directory and blob. The pure core validates paths, plans the merge, and returns new directory facts and a receipt preimage.

The shell persists planned directories and blobs, rechecks the returned root, and writes the receipt. The core does not read files, inspect environment state, access a store, spawn processes, or print output.

**Rationale:** Merge behavior must be testable from in-memory facts without a running store or filesystem.

### Decision 5: Reject ambient overlay semantics

**Choice:** Binding order is not precedence. Two directories at one path merge recursively. Identical leaves deduplicate.

A non-identical leaf collision requires one exact path decision that selects a derived `binding_ref`. Missing, duplicate, inapplicable, or stale decisions fail closed.

**Rationale:** Last-writer-wins behavior would make input order part of policy and could hide provider or artifact replacement.

### Decision 6: Validate logical paths without following symlinks

**Choice:** Mount points use an explicit root target or normalized relative components. Absolute components, parent traversal, empty internal components, and platform-dependent prefixes fail.

Existing symlink nodes remain opaque leaves. Planning and realization do not follow their targets.

**Rationale:** The composition operation owns logical tree construction, not host filesystem traversal or symlink authorization.

### Decision 7: Keep adapters optional and subordinate

**Choice:** The initial shell may accept a deterministic JSON projection for tests and machine callers. It maps that projection into the core model before identity computation.

Kamacite may later map canonical Preserves values into the same model. Mantle core will not require Kamacite, Preserves, or an OnixOS schema.

**Rationale:** Interchange formats are useful adapters. They must not become composition policy or change semantic identity.

### Decision 8: Keep the experiment separate from execution and deployment

**Choice:** This change stops after root realization, inspection, and export. It does not mount the root as a build environment or deploy it as a system.

A path-free action-result change needs its own compatibility review. A composed-root execution profile needs its own sandbox and metadata review.

OnixOS lowering must wait for a stable generic plan and a supported execution profile. Kamacite adaptation must wait for a stable plan schema.

**Rationale:** Independent follow-on changes prevent an identity prototype from becoming an unreviewed system manager.

## Failure Semantics

- An invalid plan or realization-policy limit violation fails before store reads.
- A missing or incomplete input object fails before output persistence.
- An unresolved, stale, duplicate, or inapplicable collision decision fails before output persistence.
- A persistence or root-recheck failure does not emit a successful realization receipt.
- A valid receipt does not authorize execution, deployment, activation, or release.

## Compatibility

Existing Nix-compatible store paths and action-result version 1 remain valid. Their identities do not become composition plan identities.

OCI remains an export adapter. Overlay store composition remains a backend read-through and write-routing mechanism under one logical prefix.

Optional Nix views, source-envelope references, and frontend evidence can appear in separate sidecars. They do not affect `plan_ref` or the castore root.

## Risks and Trade-offs

- Full tree snapshots can consume memory unless named limits bound the load.
- Explicit collision decisions are more verbose than ordered overlays.
- Castore does not preserve every Unix metadata class required for a production system root.
- Equal plan identity does not prove equal behavior on different kernels or runtimes.
- A generic contract adds another API that must remain compatible or receive a versioned successor.

## Follow-on Triggers

Create a separate Mantle action-result change only after fixtures prove stable object-root and receipt identities.

Create a separate Mantle execution-profile change only after the metadata support matrix and sandbox mount rules are explicit.

Create a Kamacite adapter change only after the plan schema is stable enough for positive and negative projection fixtures.

Create an OnixOS lowering change only after Mantle can consume the generic plan without OnixOS fields and can provide the required execution evidence.

# Proposal: Add frontend-neutral composition roots

## Why

Mantle can store immutable directory objects, but it cannot yet combine several object roots into one deterministic filesystem root.

Higher-level tools need this mechanism without moving package selection, provider policy, activation, deployment, or rollback into Mantle.

The first change must establish the generic identity and merge boundary. It must not require OnixOS, Kamacite, Preserves, or Nix store paths.

## What Changes

- Add a bounded composition plan over exact castore object roots, root-relative mount points, and explicit collision decisions.
- Derive stable binding references from each root and mount target instead of caller labels.
- Compute one BLAKE3 plan reference from normalized merge semantics, independent of input encoding, list order, store prefixes, resource limits, caller labels, and display names.
- Compute a separate realization-policy reference for named limits and admission policy.
- Merge complete castore directory objects into one immutable castore directory root.
- Reject unsafe mount paths, incomplete object graphs, stale collision decisions, unresolved conflicts, duplicate bindings, and limit violations.
- Emit a deterministic realization receipt that binds the plan and realization-policy references to the resulting castore root object reference.
- Add a generic experimental CLI projection for fixtures and external frontends.
- Keep source envelopes, frontend policy, deployment history, signatures, and compatibility views outside composition identity.

## Dependencies

This change applies ADR 0010 and ADR 0024. It is distinct from ADR 0012, which composes store backends under one logical prefix.

The initial implementation can consume existing castore roots. It does not depend on path-free action results or a composed-root execution profile.

## Non-Goals

- OnixOS package, provider, inventory, deployment, activation, or rollback behavior.
- A required Kamacite or Preserves dependency.
- A replacement for Nix-compatible store paths or action-result version 1.
- ABI, dependency-closure, runtime, boot, or release-eligibility proof.
- Production system-root support with full Unix ownership, ACL, capability, security-xattr, hard-link, or device-node semantics.
- FUSE access tracing, ELF compatibility policy, or automatic provider selection.

## Impact

- **Planned files**: a pure composition core, a thin castore shell, generic CLI wiring, focused fixtures, documentation, and Cairn evidence.
- **Compatibility**: existing store paths, builds, action results, OCI exports, and overlay store behavior remain unchanged.
- **Testing**: baseline tests before core changes, positive and negative identity fixtures, merge and conflict tests, store completeness tests, CLI tests, and lifecycle gates.
- **Follow-on work**: separate Cairn changes may add path-free action results, composed-root execution, a Kamacite adapter, or an OnixOS lowerer after this contract is stable.

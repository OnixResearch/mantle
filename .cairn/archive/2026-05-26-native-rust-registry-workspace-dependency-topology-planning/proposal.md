# Proposal: Native Rust registry workspace dependency topology planning

## Problem

Mantle can now execute bounded vendored-registry Rust topology graphs from explicit native source, feature, target-cfg, host, proc-macro, and transitive dependency facts. Real Cargo workspaces often centralize registry dependency declarations under `[workspace.dependencies]` and let member crates inherit them with `{ workspace = true }`.

If Mantle does not model that inheritance explicitly, workspace-inherited registry edges remain hidden Cargo resolver behavior. That creates an unsafe gap between the native planning receipts and the topology execution claim.

## Change

Add a bounded native planning slice for workspace-inherited vendored-registry dependencies:

- parse explicit `[workspace.dependencies]` entries from the root workspace manifest;
- resolve member dependency declarations using `{ workspace = true }` only when the inherited dependency is explicit and within Mantle's supported path-or-vendored-registry fragment;
- require ready native registry source facts for inherited vendored-registry packages before topology execution;
- emit deterministic receipt evidence for inherited workspace dependency decisions;
- block before `rustc` when inheritance requires unsupported Cargo resolver behavior, ambiguous member/root scope, missing workspace entries, feature/default-feature inheritance beyond the bounded fragment, network/index access, `$CARGO_HOME`, ambient registry cache lookup, or version solving.

## Non-goals

- Full Cargo workspace dependency resolver compatibility.
- Workspace package inheritance outside dependency declarations.
- General feature unification or target-specific workspace dependency inheritance unless separately modeled.
- Network/index access, `$CARGO_HOME`, ambient registry cache fallback, or version solving.
- Cargo as the build orchestrator.

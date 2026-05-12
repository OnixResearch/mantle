# Add distributed build interfaces

## Why

Crunch already has the right product boundary for distributed builds: derivations, PathInfo, castore content, sandboxed builders, and a lazy goal scheduler. The next distributed-build work should not copy Rust-specific `sccache-dist` behavior or import Buck/Bazel policy. It should define small Crunch-native seams that let local substitution, artifact publishing, distributed derivation realization scheduling, and future remote realization evolve independently.

Without this OpenSpec, it would be easy to hard-code a single cache URL, remote-realization protocol, Rust/Cargo wrapper, or worker transport into the scheduler. That would fight Crunch's goal of replacing Nix at the store/build layer and would make non-Rust bootstrap packages second-class.

## What Changes

- Add a modular distributed-build architecture baseline centered on derivation-level artifact reuse and realization.
- Define provider-neutral interfaces for realization-key derivation, artifact lookup, artifact publication, realizer selection, and remote realization backends.
- Preserve local sandbox realization and existing binary substitution as the default behavior.
- Require remote realization to be introduced behind adapters only after substitute lookup, input-closure materialization, output verification, and log/receipt capture are interface-backed.

## Scope

In scope:

- Content-addressed realization cache semantics at the derivation/output level.
- Local and remote artifact resolver/publisher seams.
- Scheduler dispatch policy for local versus remote derivation realization.
- Remote realization backend interface shape and trust boundaries.
- Positive/negative tests that prove defaults stay local-only and provider-neutral.

Out of scope:

- Implementing a concrete REAPI, S3, Redis, BuildBuddy, EngFlow, `sccache`, Buck2, Bazel, or Cargo-specific integration in this change.
- Splitting Rust compilation below the crate boundary.
- Changing delta protocol behavior or artifact identity rules.
- Promoting remote realization as enabled by default.

## Capabilities

### New Capabilities

- `distributed-builds`: Provider-neutral distributed artifact reuse and future remote realization seams.

### Modified Capabilities

- `build-pipeline`: Uses resolver/realizer abstractions before falling back to local sandbox realization while preserving derivation realization as the unit of work.
- `binary-cache`: Continues to provide substitution/publish primitives without becoming the only cache backend.

## Verification

- `openspec validate add-distributed-build-interfaces --strict`
- Future implementation must include pure unit tests for key derivation and policy selection, adapter contract tests for resolver/publisher/realizer traits, and integration tests showing local-only defaults do not contact the network.

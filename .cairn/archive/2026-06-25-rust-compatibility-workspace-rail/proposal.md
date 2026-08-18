# Proposal: Representative Rust compatibility workspace rail

## Summary

Create a maintained compatibility rail around one representative Rust workspace that exercises the next practical project-build frontier: ordinary dependencies, a proc macro, a build script, vendored registry sources, and one runnable binary.

## Motivation

Toy crates prove the mechanics but not the product boundary users care about. Before Mantle claims practical Rust project support beyond simple examples, it needs a representative fixture and repeatable rail that shows what works, what remains blocked, and which evidence class each path earns.

## Scope

- Add or generate a representative Rust workspace fixture with a binary, library dependency, proc-macro dependency, build-script dependency, and vendored registry source material.
- Validate the offline Cargo project lane against that fixture without ambient network or Cargo cache inputs.
- Run rust-plan against the same fixture and record either bounded success for supported topology slices or deterministic blockers for unsupported surfaces.
- Add negative fixtures for missing vendor material, stale source digest, failing build script metadata, and unsupported proc-macro or native-link surfaces.
- Keep docs/status wording tied to the rail's current evidence.

## Non-goals

- No claim that passing this rail means full Cargo compatibility.
- No requirement that native rust-plan must pass every surface in the first implementation; deterministic blockers are valid evidence.
- No dependency on external network services during fast validation.
- No release reproducibility or bootstrap correctness claim.

## Target Spec Domains

- `rust-package-planning` for representative Rust compatibility evidence and rust-plan blocker handling.
- `examples` for making the user-facing example/gallery status match the validated rail.
- `verification-evidence` for proof-before-claim requirements.

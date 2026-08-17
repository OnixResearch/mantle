# Proposal: Offline Cargo builds for Mantle projects

## Summary

Make the practical Rust-project path first-class: Mantle projects can build a Rust package by running Cargo inside Mantle's sandbox with all sources, vendored dependencies, toolchain inputs, and output contracts declared up front.

## Motivation

Mantle can already build manually-authored derivations and has an example that compiles a real crate with `cargo build --offline` inside the sandbox. That is useful, but it is still a bespoke derivation pattern. Users need a boring project workflow before the Cargo-free native planner becomes the default path: define a Rust package in a Mantle project, build it offline, run the binary, and inspect a bounded receipt that says exactly what was proven.

## Scope

- Add a project-level Rust package model for sandboxed offline Cargo builds.
- Require explicit source closure material: package root, lockfile identity, vendored registry/git/path dependencies, Cargo config, selected target, and toolchain input.
- Generate or lower that model into ordinary Mantle derivations rather than adding module-layer semantics.
- Emit build reports/receipts that label the build as Cargo-orchestrated inside Mantle, not Cargo-free native planning.
- Add positive and negative tests for offline success, missing vendor/source material, lock/source mismatch, unsupported network access, and runnable binary output.
- Document the workflow as the recommended near-term Rust project build path.

## Non-goals

- No claim that this replaces Cargo planning or orchestration.
- No claim of full Cargo ecosystem compatibility.
- No hidden access to `$CARGO_HOME`, ambient registry caches, target directories, git checkouts, or network fetches during the sandboxed build.
- No Onix/NixOS-style module semantics in Mantle core.
- No release reproducibility, bootstrap correctness, or compiler correctness claim from this workflow alone.

## Target Spec Domains

- `project-workflows` for the user-facing project Rust build workflow.
- `verification-evidence` for bounded proof-before-claim behavior around Rust build receipts.

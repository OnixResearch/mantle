# Proposal: Nickel export actions

## Summary

Add a Mantle-native Nickel export action inspired by `rules_nickel`: declared Nickel sources, dependencies, import paths, export format, and optional output path should produce a deterministic export plus an evaluation receipt that downstream build-correctness evidence can cite.

## Motivation

Mantle already evaluates Nickel to build derivations, but operators also need a small, reviewable primitive for exporting typed Nickel data into JSON, TOML, YAML, or raw text for frontend handoff artifacts, generated config, and evidence fixtures. `rules_nickel` shows the valuable shape: make sources and imports explicit, choose a format, and run the Nickel evaluator hermetically enough that the output can be reasoned about.

Mantle should adopt the shape without importing Bazel concepts. The important pieces are the source closure, import-path safety, format selection, evaluator identity, and output digest.

## Scope

- Define a `mantle export` / export-action contract with declared `srcs`, `deps`, `imports`, `format`, and output target.
- Support a Nickel toolchain/evaluator descriptor containing binary identity, version, and evaluator options.
- Bind exports to deterministic evaluation receipts that may feed build-correctness receipts.
- Reject import paths that are absolute or escape the declared root.
- Keep export diagnostics stable for human and JSON output.

## Non-goals

- No Bazel rule implementation in Mantle core.
- No automatic project-file mutation; file writing is explicit through `--out` or a later file-generation workflow.
- No Onix/NixOS-style module-layer interpretation of exported data.
- No claim that exported config is deployable or correct beyond the declared Nickel evaluation receipt.

## Target Spec Domains

- `build-correctness` for declared Nickel export actions and toolchain/evaluator facts.
- `operator-diagnostics` for stable export diagnostics and JSON output boundaries.

# Proposal: Keep Onix modules above Mantle's build-tool boundary

## Why

Mantle should be a build tool in the same architectural slot as Nix: evaluate/build/store primitives, derivation/build-plan execution, reports, and artifact realization. It should not become the NixOS-like module layer.

Onix modules should sit above Mantle. The Onix layer owns module ABI, inventory expansion, role/tag semantics, settings merging, provider/export topology, package/artifact selection, and real Nickel `impl` invocation. After that layer produces concrete build inputs or derivations, Mantle builds them.

The current Mantle `system eval` path is a poor integration target for Onix because it looks like a module layer but returns synthetic fragments from `JsonEvalBoundary` instead of calling module implementations. Extending that path would push Mantle toward owning system/module semantics, which is the wrong layer.

## What Changes

- Define Mantle's boundary as build-tool surface only: external frontends may hand Mantle derivations, build plans, store inputs, or evaluated build expressions.
- Define the Onix module evaluator/lowering layer as outside Mantle core. That layer owns module ABI and real Nickel module semantics.
- Stop treating Mantle's phase-1 `system eval` scaffold as the Onix integration path. It should be quarantined, documented as experimental/demo-only, moved out of Mantle core, or removed before deployable artifact work depends on it.
- Keep Mantle APIs free of Onix/NixOS-style concepts such as machines, roles, tags, settings contracts, upstream exports, and providers except as opaque data already compiled into build inputs by a frontend.

## Impact

- **Mantle files**: docs and CLI/API boundaries around `src/system_cmd.rs`, `crates/crunch-system/`, `crunch-pipeline`, and build-report surfaces.
- **Onix files**: Onix module evaluation/lowering work belongs in `onix-modules` or a separate Onix-owned adapter crate/tool that calls Mantle as a build backend.
- **Testing**: Mantle tests should prove build-tool handoff and reject accidental raw module-layer coupling. Onix tests should prove module semantics, real `impl` invocation, settings/defaults/contracts, upstream/provider flow, and lowering into Mantle build inputs.

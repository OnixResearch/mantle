## Why

crunch can build packages and bootstrap itself, but it still lacks a native
system-configuration pipeline. onix-modules proves the Nickel module pattern
works, yet today that workflow still depends on a Nix shim for evaluation
orchestration and assembly.

crunch needs its own system-config machinery so it can evaluate Nickel service
modules against an inventory, merge their output fragments, and turn the result
into buildable derivations without a Nix runtime dependency.

## What Changes

- **Add a native system-config pipeline.** Introduce a new `crunch-system`
  crate that owns module loading, module evaluation, fragment collection,
  assembler dispatch, and shared result/error types.
- **Keep the module schema in Nickel.** Structural contracts for service
  modules and inventories live in Nickel stdlib files, while Rust owns graph
  logic, orchestration, and derivation assembly.
- **Split the Nickel boundary cleanly.** Raw `NickelValue` objects stay on a
  dedicated evaluator thread, while the async pipeline talks to that thread via
  `ValueId` handles and serialized JSON results.
- **Define a stable CLI/result contract.** `crunch system eval` and
  `crunch system build` return per-machine results plus structured warnings and
  errors, with stderr honoring crunch's global `--json` flag.
- **Ship a phase-1 NixOS assembler.** The first backend proves end-to-end
  modules → fragments → derivations → build without claiming full NixOS closure
  assembly.

## Non-Goals

- Importing existing NixOS modules or nixpkgs directly.
- Full NixOS closure assembly (`/etc`, systemd units, initrd, activation).
- Secrets management, deployment, or cloud provisioning.
- A parallel multi-threaded Nickel runtime. This change standardizes one
  dedicated evaluator thread first.

## Capabilities

### New Capabilities
- `system-config-loader`: discover module files and validate their structural
  shape.
- `system-config-evaluator`: order modules, validate settings, thread exports
  and providers, and serialize output fragments.
- `system-config-collector`: merge fragments per machine with provenance.
- `system-config-assembler`: turn merged configs into buildable derivations via
  pluggable backends.
- `system-config-cli`: add `crunch system eval` and `crunch system build`.

## Impact

- **Files**: new `crates/crunch-system/` crate, new Nickel contracts under
  `lib/`, CLI wiring in `src/system_cmd.rs` and `src/main.rs`, example system
  configs under `examples/system-config/`.
- **APIs**: new `NickelEvaluator`, `EvalThreadHandle`, `SystemPipelineResult`,
  `MachineOutcome`, and assembler traits/types.
- **Dependencies**: reuse `crunch-eval`, `crunch-glue`, and `crunch-pipeline`;
  no new runtime dependency on Nix.
- **Testing**: unit tests for each layer, integration tests for the example
  system-config workflow, and end-to-end dry-run/build coverage.

## Constraints

- crunch MUST NOT require Nix at runtime for this pipeline.
- Module schema MUST remain defined by Nickel contracts, not Rust structs.
- Raw `NickelValue` values MUST stay on the dedicated evaluator thread.
- CLI stdout payloads and stderr diagnostics MUST have a coherent partial-
  success contract before implementation starts.
- The first assembler backend MUST prove the pipeline end to end without
  over-claiming full system closure support.

## Traceability

| Proposal slice | Delta spec |
|---|---|
| CLI surface | `specs/cli-surface/spec.md` |
| Error and warning model | `specs/error-model/spec.md` |
| Evaluator trait and threading seam | `specs/evaluator-trait/spec.md` |
| Module loader | `specs/module-loader/spec.md` |
| Module evaluator | `specs/module-evaluator/spec.md` |
| Fragment collector | `specs/fragment-collector/spec.md` |
| Inventory contract | `specs/inventory/spec.md` |
| System assembler | `specs/system-assembler/spec.md` |

## How to validate

1. `openspec validate system-config-modules` succeeds.
2. `openspec_gate stage=proposal change=system-config-modules` passes.
3. Unit tests cover loader validation, dependency ordering, provider threading,
   fragment merging, fixed-limit enforcement, and assembler dry-run behavior.
4. Boundary-focused tests verify that async orchestration keeps raw
   `NickelValue` values on the dedicated evaluator thread, uses `ValueId`
   handles plus `EvalThreadHandle` requests across the thread boundary,
   enforces timeout behavior, and rejects imports outside the allowlist.
5. Integration tests cover at least one checked-in multi-machine fixture for
   `crunch system eval`, including partial-success reporting,
   `--stop-after=fragments`, fatal inventory failure with empty stdout, the
   deferred `--format nickel` empty-stdout/non-zero path, and invalid
   `--format` rejection before evaluation starts.
6. Integration tests cover at least one checked-in multi-machine fixture for
   `crunch system build`, proving successful machines reuse crunch's standard
   build pipeline without a Nix runtime dependency and that the non-`--json`
   summary plus the `--json` stdout/stderr split both match the CLI contract.
7. CLI-focused checks verify the ERR-3 JSON-diagnostic stderr contract for both
   eval and build when the global `--json` flag is active.
8. CLI-focused checks verify `--machine` filtering happens before backend
   override resolution, `--assembler` overrides per-machine `class` for the
   selected machine set when assembly is requested, `crunch system eval
   --stop-after=fragments` skips assembler lookup even if `--assembler` is
   present, and unknown backend names produce assembler diagnostics only for
   machine sets that actually request assembly work.
9. Save validator and proposal-gate rerun output under
   `evidence/proposal-validation-2026-04-21.md` so proposal-stage readiness is
   attached to the change artifacts.

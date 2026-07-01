# Design: Project input fetch policy

## Architecture

Fetch policy is project build data carried from manifest validation into refresh, generated inputs, build planning, source bundle planning, and offline preflight.

- Pure core: policy schema validation, policy compatibility checks, fetch requirement classification, generated-input planning, unsupported-combination diagnostics, and non-claim wording.
- Imperative shell: actual source fetching, source-state reads/writes, generated file writes, network policy enforcement, and CLI rendering.

Policy names should describe Mantle behavior, not Nix internals. The initial model should distinguish at least generation-time source material, build-time fetch actions, and imported/offline source-state requirements.

## Compatibility rules

Patched inputs require a policy that can apply patches deterministically before the input is consumed. Inputs needed by Nickel evaluation or generated `.mantle/inputs.ncl` must be available before evaluation of dependent project files. Offline/imported-only inputs must be satisfied by imported source state or fail preflight before build execution.

Policy resolution should produce a deterministic `InputFetchRequirement` for each input: already locked and present, generation fetch required, build fetch action required, source-state required, unsupported policy, or conflicting policy.

## Integration points

`mantle refresh` may perform generation-time fetches when explicitly requested. Build planning should lower build-time fetch inputs into ordinary Mantle fetch actions. Source bundle planning should report which fetches export would perform and which records must already be imported. Offline preflight should reject generation/build network requirements unless source state satisfies the input.

## Validation strategy

Add pure positive tests for policy defaults, per-input overrides, patched input compatibility, source-state classification, and deterministic generated-input plans. Add pure negative tests for unsupported policy names, patched inputs in incompatible modes, evaluation-required inputs without available source material, offline mode with build-time network requirements, and policy conflicts after import migration.

CLI tests should prove that check/list-stale do not fetch by default, refresh mutates only when requested, and offline preflight blocks before sandbox execution when policy requires unavailable source material.

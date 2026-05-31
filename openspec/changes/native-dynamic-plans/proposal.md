## Why

Mantle currently has a dynamic-derivation compatibility seam: build outputs named `.drv` are parsed as Nix ATerm derivations and scheduled by the lazy worker. That proves the scheduler can grow during a build, but it makes the core dynamic story depend on Nix-shaped artifacts.

Mantle needs a native dynamic-plan model so self-hosting bootstrap stages, package-manager resolvers, generated package sets, and frontends such as Onix can emit typed build-plan data without depending on Nix `.drv` files or Steel.

## What Changes

- **Native dynamic plan ABI**: define `mantle-plan-v1` as typed data owned by Mantle, with schema version, bounded units, outputs, dependencies, environment, and provenance metadata.
- **Declared producer outputs**: teach derivations to declare which outputs may contain dynamic plans, so post-build discovery is explicit and fail-closed.
- **Scheduler integration**: validate dynamic plan artifacts after producer completion, register native goals, and enqueue them through the existing lazy worker path.
- **Provenance and diagnostics**: record producer goal, plan artifact path, canonical BLAKE3 digest, registered units, rejected artifacts, and compatibility-mode `.drv` discoveries separately.

## Out of Scope

- Steel integration.
- Full Nix IFD / evaluator suspension and re-entry.
- Nix `.drv` path parity for native plans.
- Package-manager-specific resolver implementations.
- Remote trust policy for dynamic plans produced outside the current build sandbox.

## Capabilities

### New Capabilities

- `build.engine.dynamic.plans.abi`: Mantle can decode, validate, hash, and schedule native dynamic build plans.
- `build.engine.dynamic.plans.scheduler`: The worker can grow the build graph from validated native plan artifacts.
- `build.engine.dynamic.plans.provenance`: Build reports and attestations can explain which producer emitted which dynamic plan and which goals it registered.

### Modified Capabilities

- `defaults.dynamic.derivations`: Dynamic derivations become a native Mantle build-plan feature; `.drv` output scanning becomes compatibility/debug behavior, not the core API.

## Impact

- **Files**: Mantle build/eval/glue/pipeline crates (current exact paths: `crates/crunch-build/src/`, `crates/crunch-glue/src/`, `crates/crunch-pipeline/src/`, `crates/crunch-eval/src/`), `lib/*.ncl`, `src/build_report.rs`, tests, docs.
- **APIs**: new Rust dynamic-plan structs and validation errors; Nickel derivation contract gains declared dynamic-plan outputs.
- **Dependencies**: no new runtime language dependency; no Steel.
- **Testing**: pure ABI positive/negative tests, scheduler integration tests, declared-output fail-closed tests, report/provenance assertions, and compatibility tests proving old `.drv` detection stays separate.

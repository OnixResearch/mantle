## ADDED Requirements

### Requirement: Cargo project import scaffold [r[project_workflows.cargo_import_scaffold]]

Mantle MUST provide a no-mutate planning workflow and an explicit apply workflow that scaffold Mantle project files from supported Cargo workspace facts, and MUST keep the generated surface build-shaped rather than module-layer-shaped.

#### Scenario: Import plan is reviewable and side-effect free [r[project_workflows.cargo_import_scaffold.scenario.plan]]

- GIVEN a Cargo workspace has supported package, target, lockfile, and source-closure facts
- WHEN the operator requests an import plan
- THEN Mantle MUST report deterministic file operations, selected packages, default target choice, source inputs, content digests, and any blockers without mutating files or store state
- AND the plan MUST be sufficient for review before apply.

#### Scenario: Apply writes only accepted project files [r[project_workflows.cargo_import_scaffold.scenario.apply]]

- GIVEN an import plan has no blocking conflicts
- WHEN the operator explicitly applies that plan
- THEN Mantle MUST write only the bounded Mantle-owned project files named by the plan
- AND it MUST fail before writing if existing files, mixed legacy/canonical surfaces, unsupported source material, or ambiguous package selection make the plan unsafe.

#### Scenario: Unsupported Cargo surfaces block scaffold claims [r[project_workflows.cargo_import_scaffold.scenario.unsupported]]

- GIVEN a Cargo workspace requires behavior outside the supported import surface, such as undeclared registry material, unsupported target kinds, missing lockfile facts, or ambiguous binary selection
- WHEN Mantle plans the import
- THEN Mantle MUST emit deterministic blockers naming the unsupported surface
- AND it MUST NOT generate partial files that imply the workspace is ready for `mantle build`.

#### Scenario: Generated project remains a build-tool handoff [r[project_workflows.cargo_import_scaffold.scenario.boundary]]

- GIVEN generated project files are produced from Cargo workspace facts
- WHEN those files are evaluated by Mantle
- THEN they MUST describe concrete derivations, package outputs, source inputs, checks, or opaque build data
- AND they MUST NOT introduce Onix/NixOS-style module semantics into Mantle core.

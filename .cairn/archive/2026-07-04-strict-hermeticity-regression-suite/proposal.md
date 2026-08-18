## Why

A strict hermeticity contract needs executable adversarial coverage. Mantle should keep a maintained regression suite with one clean strict success path and negative fixtures for environment poisoning, PATH poisoning, undeclared host execution, network access, missing closure facts, umask drift, temp-root dependence, and nondeterministic output behavior.

## What Changes

- Define the strict hermeticity regression suite inventory and expected outcomes.
- Add isolated positive and negative fixtures for each hermeticity boundary.
- Emit deterministic suite evidence that can be cited by release/global reproducibility readiness without overclaiming beyond tested axes.
- Ensure failed or unsupported cases are reported as explicit blockers or non-claims.

## Impact

- **Files**: fixture declarations, runner/report integration, docs, and Cairn verification-evidence spec delta.
- **Testing**: clean strict success fixture plus negative env, PATH, host-exec, network, closure, umask, temp-root, and nondeterminism fixtures; Cairn validation and gates.

## Out of Scope

- Exhaustive coverage of every possible host impurity.
- Running heavyweight self-hosting proofs in every edit-time regression suite invocation.

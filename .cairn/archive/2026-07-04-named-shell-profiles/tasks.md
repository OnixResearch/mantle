# Tasks

## Contract

- [x] [serial] Define named shell profile declarations, `build`/`dev`/`default` conventions, profile selection, activation data, and bounded non-claims. r[project_workflows.named_shell_profiles]
- [x] [serial] Define dev-shell decoupling from build action identity, file generation, lock refresh, release evidence, and reproducibility claims. r[project_workflows.dev_shell_decoupling]
- [x] [serial] Define service lifecycle exclusion so Organist-like services remain frontend-owned or future adapter work, not Mantle core behavior. r[project_workflows.named_shell_profiles]

## Implementation

- [x] [serial] Implement pure profile validation and activation planning over owned data with explicit limits and deterministic diagnostics. r[project_workflows.named_shell_profiles]
- [x] [serial] Thread named profile resolution through `mantle shell [name]` while preserving existing shell adapter boundaries. r[project_workflows.named_shell_profiles]
- [x] [serial] Keep shell activation non-mutating by default and ensure package build identity is derived from package/action declarations, not `shells.dev`. r[project_workflows.dev_shell_decoupling]
- [x] [serial] Update project manifest contract and docs to show `build`, `dev`, and `default` shell profiles without Nix-flake or module-layer semantics. r[project_workflows.named_shell_profiles] r[project_workflows.dev_shell_decoupling]

## Verification

- [x] [serial] Add positive tests for default profile resolution, explicit `build`/`dev` selection, env/PATH planning, shell sidecar compatibility, and unchanged package action identity after dev-shell activation. r[project_workflows.named_shell_profiles] r[project_workflows.dev_shell_decoupling]
- [x] [serial] Add negative tests for missing profiles, invalid names, ambiguous defaults, duplicate env entries, unsupported service declarations, non-UTF-8 adapter paths, attempted filegen/lock mutation during shell activation, and overclaiming diagnostics. r[project_workflows.named_shell_profiles] r[project_workflows.dev_shell_decoupling]
- [x] [serial] Run `nix run path:/home/brittonr/git/cairn#cairn -- validate --root .` and proposal/design/tasks gates before implementation tasks are marked complete. r[project_workflows.named_shell_profiles] r[project_workflows.dev_shell_decoupling]

Evidence: `nix develop -c cargo test -p crunch-shell-core -p crunch-project-core --lib`, `nix develop -c cargo test -p mantle --bin mantle project_build`, and Cairn proposal/design/tasks gates passed for this change.

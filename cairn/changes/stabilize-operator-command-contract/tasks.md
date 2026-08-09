# Tasks: stabilize the Mantle operator command contract

## Phase 1: Baseline and contracts

- [x] [serial] I1 Inventory every public command path, alias, project file, environment input, machine schema, help source, exit class, and current remediation command. r[operator_diagnostics.command_catalog_contract]
- [x] [serial] I2 Add a typed Nickel operator-surface contract with explicit owners, support tiers, mutation classes, network classes, compatibility states, and named collection limits. r[operator_diagnostics.compatibility_surface_inventory]
- [x] [parallel] I3 Add positive inventory fixtures and negative duplicate, unknown-state, unowned, conflicting-role, oversized, and malformed fixtures. r[operator_diagnostics.compatibility_surface_inventory]

## Phase 2: Command catalog and documentation

- [x] [serial] I4 Export normalized command descriptors from the Clap graph and validate them through a pure deterministic catalog core. r[operator_diagnostics.command_catalog_contract]
- [x] [serial] I5 Generate a support-tiered command reference and canonical daily workflow from the accepted catalog. r[operator_diagnostics.canonical_operator_workflow]
- [x] [parallel] I6 Add drift checks for missing commands, stale flags, undocumented aliases, wrong exit contracts, missing JSON contracts, and command examples that no longer parse. r[operator_diagnostics.command_contract_validation]

## Phase 3: Structured remediation

- [x] [serial] I7 Add a versioned remediation model with stable code, phase, safe subject, evidence refs, and ordered next actions. r[operator_diagnostics.structured_remediation]
- [x] [serial] I8 Move stable suggestion selection into a pure classifier over normalized failure facts. Keep rendering free of policy decisions. r[operator_diagnostics.structured_remediation]
- [x] [parallel] I9 Add human and JSON fixtures for success, policy rejection, missing prerequisites, unsupported platforms, store blockers, source blockers, and remote blockers. r[operator_diagnostics.structured_remediation]
- [x] [parallel] I10 Add secret, raw argv, private path, malformed generated-data, stdout pollution, and legacy-command regression fixtures. r[operator_diagnostics.command_contract_validation]

## Phase 4: Compatibility and rollout

- [x] [serial] I11 Classify every retained `crunch-*` operator surface without changing existing wire or read compatibility. r[operator_diagnostics.compatibility_surface_inventory]
- [x] [serial] I12 Document the evidence required for compatibility-read-only and historical-only transitions. r[operator_diagnostics.compatibility_surface_inventory]
- [x] [serial] I13 Update README and operator workflows to use the generated catalog and one short canonical path. r[operator_diagnostics.canonical_operator_workflow]

## Phase 5: Verification

- [x] [serial] V1 Run `nix develop -c cargo test -p mantle --test operator_diagnostics`, focused CLI parser tests, focused error tests, and generated-catalog tests. r[operator_diagnostics.command_contract_validation]
- [x] [serial] V2 Run `nix develop -c sh -c 'rustc scripts/check-stale-branding.rs -o /tmp/mantle-check-stale-branding && /tmp/mantle-check-stale-branding'` and the same command with `--self-test`. r[operator_diagnostics.compatibility_surface_inventory]
- [x] [serial] V3 Run `nix develop -c cargo fmt --check -p mantle`, focused first-party Clippy with warnings denied, Nickel contract checks, and `git diff --check`. r[operator_diagnostics.command_contract_validation]
- [x] [serial] V4 Run `nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- validate --root .` plus proposal, design, and tasks gates for this change. Record exact outputs before archive. r[operator_diagnostics.command_contract_validation]

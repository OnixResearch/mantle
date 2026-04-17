## MODIFIED Requirements

### Requirement: README reflects the shipped CLI surface

The top-level README MUST describe the current shipped CLI commands, defaults,
and safety or integrity controls accurately.

At minimum it MUST stay aligned with:
- the default logical store prefix `/crunch/store`
- the `--nix-compat` shorthand for `/nix/store`
- planning and diagnostics entry points such as `crunch doctor` and
  `crunch build --plan`
- the project-management commands (`init`, `check`, `show`, `refresh`,
  `list-stale`, `upgrade`)
- dev-shell entry points (`shell` and `develop`)
- attestation and release-evidence commands (`attest` and `release`)
- signing and trust controls that affect build, self-build, and store
  verification workflows
- strict hermetic selection for build-entry commands
- README-linked focused workflow docs when a supported workflow no longer fits
  cleanly in the top-level summary

#### Scenario: Store-path documentation matches current defaults

- GIVEN a reader follows the README store-path documentation
- WHEN they read about logical store paths and defaults
- THEN it states that derivation hashes use `/crunch/store` by default
- AND it explains `--nix-compat` as the compatibility switch for `/nix/store`
- AND it distinguishes logical `--store-prefix` from physical `--store`

#### Scenario: README command list includes current operator workflows

- GIVEN the current CLI binary
- WHEN the README lists supported operator workflows
- THEN it includes `doctor`, `build --plan`, `shell`, `develop`, `attest`,
  and `release`
- AND it does not present those commands as future work or omit them entirely

#### Scenario: README documents signing and strict hermetic controls

- GIVEN the current CLI binary
- WHEN the README explains cache, verification, and build-entry behavior
- THEN it documents signing-key and trusted-key configuration
- AND it describes unsigned trust as an override rather than the default
- AND it documents `--strict-hermetic` as an opt-in stricter build mode

#### Scenario: README describes attestation-facing build output

- GIVEN the current CLI binary
- WHEN the README explains structured build reporting or attestation workflows
- THEN it says successful build outcomes expose artifact attestation references
- AND it gives the reader a path to the attestation and release workflows
  without requiring them to inspect source code first

#### Scenario: Focused workflow docs stay linked and current

- GIVEN the README moves a longer operator workflow into a focused doc
- WHEN a reader follows that README link
- THEN the focused doc includes the current command entry point for that
  workflow
- AND the README still gives enough context for the reader to find it

#### Scenario: Command coverage can be checked against help output

- GIVEN the current top-level `crunch --help` command list
- WHEN a reviewer compares it with the README command summary and any
  README-linked focused workflow docs
- THEN every shipped top-level operator command family appears in that doc set
- AND no shipped operator family is left undocumented
## ADDED Requirements

### Requirement: README reflects the shipped CLI surface

The top-level README MUST describe the current shipped CLI commands, defaults,
and safety/integrity controls accurately.

At minimum it MUST stay aligned with:
- the default logical store prefix `/crunch/store`
- the `--nix-compat` shorthand for `/nix/store`
- the project-management commands (`init`, `check`, `show`, `refresh`,
  `list-stale`, `upgrade`)
- signing and trust controls that affect build, self-build, and store
  verification workflows

#### Scenario: Store-path documentation matches current defaults

- GIVEN a reader follows the README store-path documentation
- WHEN they read about logical store paths and defaults
- THEN it states that derivation hashes use `/crunch/store` by default
- AND it explains `--nix-compat` as the compatibility switch for `/nix/store`
- AND it distinguishes logical `--store-prefix` from physical `--store`

#### Scenario: README command list includes project-management commands

- GIVEN the current CLI binary
- WHEN the README lists supported commands
- THEN it includes the project-management commands
- AND it does not describe them as future work or omit them entirely

#### Scenario: README documents signing controls as non-default overrides

- GIVEN the current CLI binary
- WHEN the README explains cache and verification behavior
- THEN it documents signing-key and trusted-key configuration
- AND it describes unsigned trust as an override rather than the default

## ADDED Requirements

### Requirement: Filtered Nix sources preserve reviewed self-description

r[mantle.flake_source_inventory.self_description] Mantle MUST include the repository-owned `flake.nix` file in the shared filtered source used by Nix-built checks while excluding Git metadata and avoiding broad ambient source inclusion.

#### Scenario: Inventory tests inspect the same filter that built them

- GIVEN a Nix-built inventory test reads the repository flake to validate project and transcript source rules
- WHEN `cleanSourceWith` constructs the test source
- THEN the exact `flake.nix` file SHALL be present, `.git` metadata SHALL remain absent, and no unrelated working-tree source SHALL be admitted implicitly.

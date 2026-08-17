## ADDED Requirements

### Requirement: `.drv` directory producer selects complete reachable closures

r[foreign_derivation_import.drv_dir_closure_producer] The Nix producer adapter MUST accept a directory of concrete Nix ATerm `.drv` files, map each direct `*.drv` child to its logical `/nix/store/<basename>` identity, and emit artifacts for the root-reachable closure selected by `--root-derivation`. The producer MUST NOT invoke `nix`, `nix-store`, flake evaluation, Nix expression evaluation, overlay logic, or host store closure discovery while parsing the directory bundle.

#### Scenario: directory bundle emits selected root closure

GIVEN a `.drv` directory contains the selected root derivation, every reachable input derivation, and unrelated derivation files
WHEN the Nix producer parses the directory and follows input derivation edges from the selected root
THEN the emitted graph MUST contain the selected root and its reachable input derivations
AND unrelated directory entries MUST NOT perturb the emitted graph identity for that selected root.

#### Scenario: directory bundle missing input fails closed

GIVEN a `.drv` directory contains the selected root derivation but omits a reachable input derivation referenced by that root or another reachable node
WHEN the Nix producer follows input derivation edges
THEN artifact production MUST fail with a deterministic missing-input diagnostic
AND it MUST NOT emit partial graph or package-index artifacts.

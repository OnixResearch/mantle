## ADDED Requirements

### Requirement: Direct `.drv` producer parses concrete derivation closures

r[foreign_derivation_import.direct_drv_producer] The Nix producer adapter MUST accept an explicit concrete `.drv` closure input mode that parses Nix ATerm derivation files with Rust crate code and emits the same `foreign-derivation-graph-v1` plus `foreign-package-index-v1` artifacts as derivation-JSON input. The producer MUST NOT invoke `nix`, `nix-store`, flake evaluation, Nix expression evaluation, overlay logic, or host store closure discovery while parsing those explicit files.

#### Scenario: explicit drv closure becomes graph data

GIVEN an operator supplies a root derivation identity and explicit logical `.drv` path-to-file mappings for every derivation in the closure
WHEN the Nix producer parses the `.drv` files
THEN the emitted graph MUST preserve derivation nodes, outputs, input derivation edges, fixed-output metadata, source refs, environment fields, and declared references as data
AND the package index MUST map the selected package name and system to the lowered root derivation identity.

#### Scenario: malformed drv input fails closed

GIVEN an explicit `.drv` input file is malformed, unreadable, or does not parse as a Nix derivation ATerm
WHEN the Nix producer parses the explicit closure
THEN artifact production MUST fail with a deterministic diagnostic
AND it MUST NOT emit partial graph or package-index artifacts.

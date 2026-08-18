## ADDED Requirements

### Requirement: Live Nixpkgs export-to-plan proof is captured

r[foreign_derivation_import.live_nixpkgs_export_to_plan_proof] Mantle SHOULD maintain current evidence that a real Nixpkgs package can be exported through host Nix into concrete derivation facts, lowered into foreign import artifacts, and consumed by Mantle validation/planning without Nix available during consumption. The proof MUST name the strongest proven state and MUST NOT claim substitution, local rebuild compatibility, output trust, package correctness, or reproducibility.

#### Scenario: live nixpkgs hello reaches planning without Nix during consumption

GIVEN host Nix resolves a live `nixpkgs#hello` derivation path
AND host Nix exports the recursive concrete derivation JSON before artifact emission
WHEN Mantle lowers that JSON with `foreign-import produce-nix`
THEN the evidence MUST include emitted `foreign-derivation-graph-v1` and `foreign-package-index-v1` artifact paths
AND later `foreign-import validate` and `foreign-import plan` proof commands MUST run with a PATH that lacks `nix` and `nix-store`.

#### Scenario: live proof stays admission/planning scoped

GIVEN the live proof validates and plans the lowered artifacts
WHEN the evidence summarizes the result
THEN it MUST report the strongest proven state as admitted/planned
AND it MUST list substitution, local rebuild compatibility, output trust, package correctness, and reproducibility as non-claims.

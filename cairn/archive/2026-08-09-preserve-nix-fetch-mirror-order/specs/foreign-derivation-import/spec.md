## ADDED Requirements

### Requirement: Nix fetch candidates use one canonical order

r[foreign_derivation_import.nix_fetch_candidate_normalization] The Nix producer adapter MUST normalize each supported concrete fixed-output fetch into one bounded ordered candidate list before graph publication. The compiler MUST bind that exact list into target derivation and executable-plan identity. The consumer MUST NOT interpret Nix-specific candidate encodings or ambient mirror state.

#### Scenario: Structured Nix candidates preserve order

GIVEN a supported concrete Nix fixed-output fetch has a structured `__json.urls` array of direct address strings
WHEN the ATerm or derivation-JSON producer lowers the node
THEN the emitted graph MUST contain the same candidates in the same order
AND candidate-like top-level environment fields alongside `__json` MUST be rejected as conflicting input
AND later validation, planning, and realization MUST NOT require Nix or nixpkgs.

#### Scenario: Unstructured Nix candidates normalize consistently

GIVEN a supported concrete Nix fixed-output fetch has one `url` or an unstructured `urls` value
WHEN the Nix producer normalizes its acquisition facts
THEN it MUST produce the same canonical candidate model used for structured input
AND if both fields exist, `url` MUST match the first normalized `urls` candidate.

#### Scenario: Existing canonical graphs remain readable

GIVEN an existing foreign graph has no canonical candidate field and uses the accepted primary-address plus payload-mirror representation
WHEN the foreign compiler reads the graph
THEN it MUST preserve the existing ordered candidate behavior
AND if canonical and legacy representations both exist, they MUST agree exactly or fail closed.

#### Scenario: Candidate policy changes recipe identity

GIVEN two otherwise equal supported Nix fetch nodes declare the same fixed-output digest with different candidate order
WHEN Mantle compiles both nodes
THEN their target derivation and executable-plan identities MUST differ
AND their required fixed-output content identity MUST remain unchanged.

#### Scenario: An unavailable candidate advances in order

GIVEN a canonical candidate list contains an unavailable first address and an available second address
WHEN the ordinary Mantle fetch service attempts acquisition
THEN it MUST try the first address before the second address
AND its bounded attempt evidence MUST record the unavailable and selected candidates in that order.

#### Scenario: Content mismatch stops fallback

GIVEN a candidate returns content that does not match the required fixed-output identity
WHEN shared fixed-output verification checks the acquired content
THEN the build MUST fail without trying a later candidate in the same attempt
AND the mismatched content MUST NOT enter PathInfo admission.

#### Scenario: Invalid or ambiguous candidate data fails closed

GIVEN Nix candidate data is empty, whitespace-only, malformed, non-string, duplicate, oversized, conflicting, or uses Mantle's reserved private candidate field
WHEN the producer or compiler validates the node
THEN it MUST return a deterministic diagnostic before artifact publication or build dispatch
AND it MUST NOT emit a partial candidate list or accept foreign control of the private binding.

#### Scenario: Mirror aliases remain producer policy

GIVEN a Nix candidate requires `mirror://`, a hashed-mirror table, or an ambient `NIX_MIRRORS_*` override
WHEN the producer cannot emit an explicit supported address expansion with recorded provenance
THEN it MUST mark the node unsupported or reject artifact publication
AND the consumer MUST NOT consult ambient Nix or nixpkgs mirror state.

#### Scenario: Arbitrary fixed-output builders are not downloads

GIVEN a fixed-output derivation has candidate-like fields but uses unsupported builder transforms or semantics
WHEN the Nix producer classifies the node
THEN candidate presence MUST NOT classify it as a simple download by itself
AND the producer MUST retain an explicit blocker instead of silently bypassing the builder behavior.

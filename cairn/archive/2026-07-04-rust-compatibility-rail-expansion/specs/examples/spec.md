## ADDED Requirements

### Requirement: Rust compatibility rail is surface-matrix-backed

r[examples.rust_compatibility_surface_matrix] Mantle SHOULD maintain a source-controlled Rust compatibility surface matrix for the representative Rust project rail. The matrix SHOULD name each supported, blocked, or out-of-scope Rust/Cargo surface; identify the applicable offline Cargo and native rust-plan evidence classes; map each surface to positive or negative fixtures; and require docs to preserve bounded non-claims.

#### Scenario: supported surface has fixture evidence

GIVEN the matrix marks a Rust project surface as supported by the offline Cargo rail, native rust-plan rail, or both
WHEN the representative rail validation runs
THEN a deterministic fixture MUST exercise that surface in the named lane
AND the fixture result MUST be reported under the correct evidence class.

#### Scenario: unsupported surface has blocker evidence

GIVEN the matrix marks a Rust project surface as unsupported or blocked
WHEN the validation rail reaches that surface
THEN Mantle MUST report a stable unsupported-surface or lane-specific blocker
AND docs MUST NOT describe the surface as supported for that lane.

#### Scenario: gallery does not overclaim matrix coverage

GIVEN the examples gallery or README describes the Rust compatibility rail
WHEN the documentation drift rail runs
THEN the docs MUST name the matrix, supported lanes, blocker status, and non-claims
AND they MUST NOT present the matrix as proof of full Cargo compatibility, compiler correctness, release reproducibility, or bootstrap correctness.

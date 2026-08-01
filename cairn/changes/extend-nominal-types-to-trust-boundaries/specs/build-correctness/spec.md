# Build Correctness Nominal Boundary Delta

## ADDED Requirements

### Requirement: Admitted trust-boundary values are nominal

r[build_correctness.nominal_boundaries.admission] Mantle MUST convert structural input into checked nominal values before pure core logic uses semantic identifiers, paths, digests, or bounded quantities.

#### Scenario: Valid structural input admits nominal values

- **GIVEN** a bounded wire record contains values that satisfy the accepted scalar rules
- **WHEN** semantic admission runs
- **THEN** Mantle MUST construct checked domain types before graph, policy, evidence, or execution planning uses those values
- **AND** the admitted core MUST retain those types until diagnostics or wire projection requires text

#### Scenario: Invalid values cannot bypass admission

- **GIVEN** structural input contains an empty, oversized, control-bearing, malformed, zero, overflowing, or otherwise invalid semantic value
- **WHEN** direct wire admission or deserialization runs
- **THEN** Mantle MUST reject the value with a deterministic diagnostic
- **AND** derived or custom deserialization MUST NOT bypass the constructor invariant
- **AND** the pure core MUST NOT receive the invalid value

### Requirement: Semantic identifier and reference roles remain distinct

r[build_correctness.nominal_boundaries.identities] Mantle MUST use distinct Rust types for identifiers and references whose accidental exchange can change graph, protocol, artifact, or evidence meaning.

#### Scenario: Unrelated identifiers cannot be exchanged

- **GIVEN** stage, artifact, tool, request, session, endpoint, requirement, and release identifiers share a string representation
- **WHEN** source passes one role to an API that requires another role
- **THEN** the source MUST fail compilation or require an explicit checked conversion
- **AND** unrestricted primitive conversion MUST NOT erase the role inside admitted core logic

#### Scenario: Graph references retain resolved roles

- **GIVEN** structural graph input names an existing declared node
- **WHEN** graph admission resolves that reference
- **THEN** the admitted reference MUST retain the resolved node role
- **AND** a missing node or wrong-role node MUST fail before graph decisions use it

### Requirement: BLAKE3 format and selected digest roles are checked

r[build_correctness.nominal_boundaries.digests] Mantle MUST validate Mantle-owned lowercase BLAKE3 text at admission and MUST keep selected same-format digest roles distinct where substitution can change evidence meaning.

#### Scenario: Valid BLAKE3 value enters its role

- **GIVEN** structural input contains a lowercase BLAKE3 value with the accepted length
- **WHEN** digest admission runs for the declared role
- **THEN** Mantle MUST construct the checked digest value
- **AND** later core logic MUST NOT repeat raw format checks for that value

#### Scenario: Malformed or wrong-role digest is rejected

- **GIVEN** a digest has the wrong length, case, alphabet, algorithm, or semantic role
- **WHEN** Mantle admits or passes that digest to a role-specific API
- **THEN** admission MUST reject malformed text
- **AND** source-level wrong-role substitution MUST fail compilation or require an explicit checked conversion

#### Scenario: Interoperability digest remains algorithm tagged

- **GIVEN** a protocol or external format requires a non-BLAKE3 digest
- **WHEN** Mantle admits that value
- **THEN** it MUST retain the required algorithm, checked digest value, and interoperability reason as one validated value
- **AND** it MUST NOT relabel that digest as a Mantle-owned BLAKE3 identity

### Requirement: Unit-bearing quantities and path authorities are explicit

r[build_correctness.nominal_boundaries.units_and_paths] Mantle MUST distinguish bounded quantities and path classes when equal primitive representations have different units, limits, ordering rules, or authority.

#### Scenario: Quantity uses its declared unit and bound

- **GIVEN** time, bytes, counts, offsets, indexes, or generations enter an admitted core
- **WHEN** Mantle constructs the value
- **THEN** the type or containing aggregate MUST identify its unit and accepted range
- **AND** zero, overflow, or a value above the named bound MUST fail when the contract forbids it

#### Scenario: Related quantities preserve their relationship

- **GIVEN** two same-unit values form a validity window, range, or ordered boundary
- **WHEN** Mantle admits the pair
- **THEN** one aggregate constructor MUST validate their relationship
- **AND** later core logic MUST NOT receive an inverted or otherwise invalid pair

#### Scenario: Path roles cannot silently cross authority boundaries

- **GIVEN** absolute executable, specification, repository-relative, bundle-relative, logical-store, provisional, and final paths share text representations
- **WHEN** a core operation receives one path class
- **THEN** it MUST accept only the required checked role
- **AND** the checked path MUST NOT be described as proof of filesystem presence, authorization, or safe I/O

### Requirement: Nominal migrations preserve accepted wire identity

r[build_correctness.nominal_boundaries.compatibility] Mantle MUST project admitted values through accepted wire contracts and MUST preserve canonical bytes and identity unless a separate versioned change approves a difference.

#### Scenario: Accepted wire projection remains stable

- **GIVEN** an accepted fixture passes before and after a nominal migration
- **WHEN** Mantle compares field names, scalar spellings, enum tags, nullable fields, collection shapes, canonical bytes, and BLAKE3 identities
- **THEN** those values MUST remain equal
- **AND** a Rust API migration MUST use an explicit compatibility adapter where supported callers still need the old shape

#### Scenario: Validation retains bounded diagnostics

- **GIVEN** one structural record contains several invalid semantic values
- **WHEN** the accepted boundary promises bounded multi-issue reporting
- **THEN** Mantle MUST retain the structural wire value long enough to report the allowed issue set
- **AND** fail-fast custom deserialization MUST NOT silently reduce that diagnostic contract

### Requirement: Nominal domains have regression guards

r[build_correctness.nominal_boundaries.guard] Mantle MUST add compile-time and policy guards for migrated domains so raw primitive aliases, unchecked constructors, or unrestricted deserialization cannot silently return.

#### Scenario: Primitive regression is introduced

- **GIVEN** a migrated admitted field returns to a raw primitive or gains an unchecked construction path
- **WHEN** compile-fail fixtures, source policy, or reviewed nominal-domain checks run
- **THEN** the regression MUST fail with a deterministic finding
- **AND** exceptions MUST identify a bounded compatibility boundary rather than disable the domain check broadly

### Requirement: Nominal type claims remain local

r[build_correctness.nominal_boundaries.claim_boundary] Mantle documentation and evidence MUST limit nominal-type claims to local scalar admission, category separation, relationship checks, and preserved wire identity.

#### Scenario: Admitted values pass all nominal checks

- **GIVEN** a boundary passes constructor, admission, role, compatibility, and policy checks
- **WHEN** Mantle reports that result
- **THEN** it MAY claim the supplied values satisfied the named local type contract
- **AND** it MUST NOT claim artifact correctness, source trust, store presence, safe filesystem I/O, sandbox enforcement, remote peer trust, compiler correctness, or release eligibility

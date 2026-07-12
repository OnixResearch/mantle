# Release Provenance Specification

## Purpose

Confine release bundle tree traversal, copying, and hashing to declared source and destination roots without following symlinks.

## Requirements

### Requirement: Release tree discovery does not follow symlinks

r[mantle.release_provenance.bundle_tree_copy.no_follow] Mantle MUST classify release tree entries with no-follow metadata and MUST NOT recurse through a symlink during discovery, copying, hashing, or verification, even when the symlink target is a directory.

#### Scenario: Internal relative symlink is copied as one entry

r[mantle.release_provenance.bundle_tree_copy.fixtures.positive]
- GIVEN a release input tree contains regular nested entries and a supported relative symlink whose target stays within the planned tree
- WHEN Mantle plans and copies the tree
- THEN the symlink MUST be represented and hashed as a symlink entry
- AND traversal MUST NOT enumerate descendants through the link.

### Requirement: Tree copy planning is pure and deterministic

r[mantle.release_provenance.bundle_tree_copy.plan] Mantle MUST build a deterministic copy plan from normalized relative paths, no-follow entry kinds, modes, and symlink targets before mutation, while directory enumeration, byte reads/writes, capability handling, and metadata revalidation remain in the shell.

#### Scenario: Invalid tree shape blocks mutation

r[mantle.release_provenance.bundle_tree_copy.plan.invalid]
- GIVEN normalized observations contain an absolute path, parent traversal, duplicate path, missing real-directory parent, unsupported special file, excessive bound, or invalid symlink target
- WHEN the pure planner evaluates the tree
- THEN it MUST return ordered blockers
- AND no destination mutation MAY begin.

### Requirement: Destination writes remain capability-confined

r[mantle.release_provenance.bundle_tree_copy.destination_confinement] Mantle MUST execute every tree-copy mutation relative to the declared destination capability without following a destination symlink, and MUST revalidate source kind and destination parent kind before each operation.

#### Scenario: Directory symlink escape fails without external writes

r[mantle.release_provenance.bundle_tree_copy.fixtures.negative.symlink_escape]
- GIVEN a source tree contains a directory symlink whose followed descendants would map through a destination symlink to an external target containing a sentinel
- WHEN release bundle creation processes the tree
- THEN Mantle MUST fail closed before writing any descendant through the link
- AND the external sentinel and every path outside the destination capability MUST remain unchanged.

#### Scenario: Destination or source type drift fails closed

r[mantle.release_provenance.bundle_tree_copy.fixtures.negative.type_drift]
- GIVEN a planned source entry or destination parent changes type, becomes a symlink, or no longer matches its no-follow observation before execution
- WHEN the shell revalidates the operation
- THEN Mantle MUST stop with a deterministic drift diagnostic
- AND it MUST NOT follow the changed entry.

### Requirement: Supported symlinks remain inside the planned tree

r[mantle.release_provenance.bundle_tree_copy.symlink_policy] Mantle MAY preserve a symlink only when its target is relative, lexically resolves within the planned tree from the link's parent, names a planned entry, and is copied without traversal; absolute, escaping, unplanned, or unsupported symlinks MUST fail closed.

#### Scenario: Escaping target is rejected

r[mantle.release_provenance.bundle_tree_copy.fixtures.negative.target]
- GIVEN a release tree contains an absolute symlink target or a relative target that normalizes outside the planned root
- WHEN tree-copy planning runs
- THEN the plan MUST be rejected with a symlink-target diagnostic
- AND no bundle or external target MAY be mutated.

### Requirement: Confinement regression evidence is required

r[mantle.release_provenance.bundle_tree_copy.validation] The change MUST include positive nested-tree/internal-symlink tests and negative source-link, destination-link, target-escape, type-drift, special-file, and external-sentinel tests through the production release bundle copy path.

#### Scenario: Production regression matrix proves confinement

r[mantle.release_provenance.bundle_tree_copy.validation.production]
- GIVEN the positive and negative copy fixtures
- WHEN focused release evidence tests run
- THEN valid trees MUST preserve deterministic BLAKE3 identities
- AND every escape fixture MUST fail while all out-of-root sentinels remain unchanged.

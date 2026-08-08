## ADDED Requirements

### Requirement: Mantle owns a split nix-archive and castore NAR boundary

r[store_transports.nix_archive_boundary] Mantle MUST pin one reviewed `nix-archive` package version and source identity behind one shared adapter. The adapter MUST own filesystem NAR defaults, case-hack selection, digest behavior, error mapping, and evidence fields. Adapted Snix MUST retain castore NAR rendering and ingest until a separate accepted change proves an equivalent service-backed replacement.

#### Scenario: A caller needs filesystem NAR facts

GIVEN a Mantle caller needs NAR bytes, size, or a recursive hash from a host filesystem path
WHEN the caller uses the shared NAR adapter
THEN the adapter MUST select the reviewed `nix-archive` implementation and explicit Mantle policy
AND the caller MUST NOT select upstream defaults or import `nix_archive::nar` directly.

#### Scenario: A caller needs castore NAR work

GIVEN a Mantle caller starts from a castore node or must ingest a streamed NAR into castore services
WHEN the caller selects its NAR implementation
THEN it MUST use the accepted Snix service-backed path
AND it MUST NOT buffer the complete NAR to force it through a filesystem-only adapter.

#### Scenario: The upstream package changes

GIVEN the package version, Cargo checksum, reviewed source revision, API behavior, or license differs from accepted evidence
WHEN Mantle evaluates the dependency
THEN it MUST treat prior parity and adoption evidence as stale
AND it MUST block the changed dependency until review and positive and negative evidence are current.

### Requirement: Filesystem NAR observations are byte-safe and explicit

r[store_transports.nix_archive_filesystem_observation] Mantle MUST compute selected filesystem NAR observations through the shared adapter. Each observation MUST preserve raw filename and symlink-target bytes, executable mode, canonical child order, explicit case-hack policy, NAR size, and the requested digest. Filesystem I/O MUST run in a thin blocking shell. Observation results MUST NOT grant PathInfo, store, source, or release authority.

#### Scenario: A physical store path is checked

GIVEN a physical output tree and expected final NAR facts
WHEN Mantle checks the tree through the shared adapter
THEN it MUST compare the observed size and digest with the expected facts
AND node identity, signatures, references, content-addressed identity, and persistence MUST remain separate checks.

#### Scenario: A byte-safe tree is observed

GIVEN a valid Unix tree contains non-UTF-8 names, non-UTF-8 symlink targets, executable files, and nested directories
WHEN the adapter encodes or hashes the tree
THEN it MUST preserve those bytes and modes in canonical NAR order
AND it MUST return the same supported NAR facts as the accepted parity corpus.

#### Scenario: The tree changes during observation

GIVEN a path component changes or becomes unreadable during traversal
WHEN the adapter observes the tree
THEN it MUST return a typed non-success result without silent fallback
AND no caller MUST persist PathInfo, lock data, source identity, or a successful receipt from that observation.

### Requirement: nix-archive adoption is parity-gated

r[store_transports.nix_archive_parity] Mantle MUST retain durable parity evidence before each production filesystem NAR seam moves. Evidence MUST compare NAR bytes, size, SHA-256, requested additional hashes, byte-safe names, modes, symlinks, directory order, case-hack behavior, errors, and selected race cases across the applicable implementations.

#### Scenario: The complete parity matrix agrees

GIVEN current upstream identity and all required positive and negative fixtures
WHEN `nix-archive`, Snix, and the available Nix oracle produce the required observations
THEN every required byte and fact MUST agree before the production seam moves
AND the evidence MUST identify unavailable optional oracles without reporting them as passed.

#### Scenario: One required observation differs

GIVEN any required fixture has different bytes, size, digest, mode, name handling, case-hack behavior, or error disposition
WHEN the cutover gate evaluates the evidence
THEN it MUST reject the seam migration with the compared observations
AND production MUST remain on the previously accepted implementation.

#### Scenario: Generated tests exceed a named bound

GIVEN a generated tree would exceed the configured case count, depth, entry count, file-size, or total-byte limit
WHEN the parity harness plans the case
THEN it MUST reject the case before disallowed allocation or filesystem work
AND the rejection MUST NOT count as parity success.

### Requirement: Full-buffer decode and partial restore stay outside production transport

r[store_transports.nix_archive_castore_separation] Mantle MUST NOT use complete-byte-slice NAR decode or non-transactional restore for production archive, cache, remote, or Nario payload paths in this change. A future restore path MUST use named bounds, a fresh staging destination, cleanup, no-replace publication, and post-publication verification.

#### Scenario: A large streamed payload enters Mantle

GIVEN a native archive, Nario archive, HTTP cache, remote build, or shared Rust cache streams a NAR payload
WHEN Mantle validates or ingests that payload
THEN it MUST keep the accepted bounded streaming castore path
AND it MUST NOT collect the complete payload only to call a borrowed-slice decoder or restorer.

#### Scenario: Restore is requested through the new adapter

GIVEN production code requests filesystem restoration through `nix-archive`
WHEN the dependency guard evaluates the request before a separate restore change is accepted
THEN it MUST reject the production use
AND tests or bounded fixture tools MUST remain the only permitted decode or restore users.

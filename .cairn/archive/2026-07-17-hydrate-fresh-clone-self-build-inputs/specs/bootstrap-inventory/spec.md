## ADDED Requirements

### Requirement: Fresh clones hydrate explicit self-build inputs from a verified bundle

r[bootstrap_inventory.fresh_clone_source_hydration] Mantle MUST provide a bounded offline hydration workflow that reconstructs the ignored Cargo directory source and pins the legacy provider source records from an externally identified source bundle without consulting ambient caches or the network.

#### Scenario: Verified bundle hydrates a fresh clone

GIVEN a fresh Mantle clone has no `vendor-deps/`, an operator supplies a bootstrap source bundle, and an independently obtained expected manifest BLAKE3 matches that bundle
WHEN Mantle hydrates the self-build inputs
THEN it MUST validate exactly one vendored-Cargo record plus the required provider archive and provider manifest records
AND it MUST validate the materialized Cargo directory against the clone's `Cargo.lock`, `.cargo/vendor-config.toml`, package checksums, and file checksums before publication
AND it MUST publish `vendor-deps/` atomically without replacement, import and pin the provider records, and bind the successful report to the manifest and vendor content identities.

#### Scenario: Hydrated provider is available without network

GIVEN fresh Mantle source state was hydrated from the verified bundle
WHEN `bootstrap --fetch --offline-source-preflight` selects the same legacy provider URL with network access unavailable
THEN Mantle MUST materialize the provider from pinned source state
AND it MUST NOT silently downgrade to a live network fetch.

#### Scenario: Invalid hydration fails without clobbering outputs

GIVEN the expected manifest BLAKE3 is wrong, the bundle is tampered, a required record is missing or duplicated, the vendored Cargo payload fails lock/checksum validation, source-state persistence fails, or `vendor-deps/` already exists
WHEN hydration runs
THEN Mantle MUST fail with a deterministic diagnostic
AND it MUST NOT replace an existing path, leave a partially published vendor directory, emit a success report, or mark invalid provider state ready.

#### Scenario: Hydration evidence remains bounded

GIVEN fresh-clone hydration and offline provider preflight succeed
WHEN documentation, lifecycle evidence, or an operator report cites the result
THEN the claim MUST be limited to the declared Cargo and provider source/input payloads being locally available and identity-matched
AND it MUST NOT claim fixed-point self-build success, completeness for undeclared future bootstrap sources, compiler correctness, seed trust removal, release reproducibility, independent rebuild agreement, deployment success, or full Cargo compatibility.

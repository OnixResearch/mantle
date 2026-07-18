## ADDED Requirements

### Requirement: Hydrated fresh clones prove fixed-point self-hosting without live source acquisition

r[bootstrap_inventory.hydrated_fresh_clone_fixed_point] Mantle MUST provide a bounded proof workflow in which an externally identified source bundle hydrates a fresh clone with the complete evaluated fixed-fetch source closure and both self-build stages fail closed before any undeclared live source acquisition.

#### Scenario: Connected producer exports the complete proof source closure

GIVEN committed Mantle self-build roots contain fixed-output URL or Git sources
WHEN a connected producer creates the full-proof source bundle
THEN Mantle MUST evaluate the selected proof roots, materialize every non-local fixed-fetch payload through the ordinary fixed-output verifier, and bind those payloads to their URL, revision when applicable, hash mode, expected hash, and content BLAKE3 identities
AND the bundle MUST also carry the exact fresh-clone vendored Cargo, provider archive, and provider manifest records without treating URLs and hashes alone as payload availability.

#### Scenario: Fresh proof stages forbid unmatched source fetches

GIVEN a fresh clone hydrates an independently identity-matched full-proof bundle into fresh source state
WHEN stage0 or stage2 requests a builtin fixed-output source
THEN the request MUST resolve from a matching pinned source record or fail before URL, Git, proxy, DNS, or other live acquisition
AND successful stage evidence MUST bind the enforced source-state identity and report zero live-fetch events.

#### Scenario: Hydrated fresh clone reaches the admitted fixed point

GIVEN the clone initially lacks `vendor-deps/`, Cargo source payloads, imported source state, and prior proof outputs
WHEN the full stage0 → stage1 → stage2 proof runs with the authenticated source bundle and live source acquisition forbidden
THEN locked Cargo metadata MUST succeed from the hydrated vendor tree, stage1 and stage2 MUST complete from fresh stage state using the same authenticated source authority, and their binary BLAKE3 digests MUST match
AND the proof bundle MUST retain contracted hydration, source-policy, stage, fallback, tool, and fixed-point evidence without depending on the producer checkout's local paths.

#### Scenario: Incomplete or invalid source authority fails closed

GIVEN the expected bundle identity is wrong, a required record is missing, duplicated, stale, unpinned, tampered, unsupported, or mapped to the wrong URL or revision
WHEN hydration, preflight, or either proof stage runs
THEN Mantle MUST fail with a deterministic diagnostic before emitting admitted fixed-point success
AND it MUST NOT silently fetch the missing source, publish partial hydration success, reuse arbitrary producer build outputs, or update a successful-proof alias.

#### Scenario: Fresh-clone fixed-point evidence remains bounded

GIVEN the hydrated fresh-clone proof succeeds
WHEN operators or release tooling cite the result
THEN the claim MUST identify the committed source closure, expected bundle BLAKE3, source-state BLAKE3, provider kind, platform, proof mode, stage binary digests, and live-fetch denial result
AND it MUST NOT claim full-source bootstrap, compiler correctness, seed trust removal, bit-for-bit release reproducibility, independent rebuild agreement, deployment success, or full Cargo compatibility.

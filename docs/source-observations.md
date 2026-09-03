# Source observations and monotonic ingest

Mantle records source facts without granting source authority.
The pure model is in `crates/crunch-source-core`.
The filesystem adapter is in `src/source_bundle/`.

## Observation contract

A `mantle-source-observation-v1` value contains these canonical facts:

- source kind;
- locator class;
- an immutable revision for Git sources;
- a normalized relative projection;
- a snapshot profile;
- the measured BLAKE3 content identity;
- a domain-separated observation BLAKE3;
- the required non-claim.

The observation BLAKE3 excludes locator hints and mutable reference hints.
A mirror URL change does not change the source identity.
A branch change does not preserve identity when its immutable revision changes.

Git observations name `sha1` or `sha256` explicitly.
The revision must have the exact lowercase hexadecimal width for that format.

## Locator boundary

Locators are acquisition hints only.
They do not prove ownership, origin authority, or content identity.

Mantle rejects locator user information and recognized secret query fields.
Examples include `token`, `access_token`, `password`, `secret`, `sig`, and `x-amz-signature`.
Mantle also rejects control characters and unsafe logical paths.

The source-bundle adapter first uses the existing `FetchUrl` and `GitRevision` checks.
It then sends structural facts to `crunch-source-core`.
Provider, filesystem, and network types do not enter the core.

## Source-bundle v1 compatibility

Source-bundle v1 remains the wire format.
Its canonical bytes and manifest identity do not include the new observation sidecar.

The compatibility adapter creates an observation only when the v1 record has unambiguous facts.
A Git record needs an immutable revision with a recognized object format.
Ambiguous legacy records get `provenance-unavailable-legacy-v1`.
Mantle does not invent missing revision, projection, profile, or locator facts.

## Monotonic ingest

The pure ingest planner returns one of four results:

- `add` permits one create-new publication;
- `identical-reuse` permits no write;
- `identity-conflict` rejects changed bytes or provenance for an admitted identity;
- `invalid-rejection` rejects malformed or contradictory facts.

Only `add` grants write authority.
The shell validates the plan before it creates state.
It stages data, synchronizes it, and uses no-replace publication.
The Linux adapter uses `durable-file-publication` revision `951c27f59003cea9bfdb40ed4d89653d50fada1f`.
It never replaces an admitted source record or pin.

A failed planned import removes files that the same operation created.
Existing records, payloads, pins, roots, and readiness facts remain unchanged.
Fault tests cover record and pin interruption points.

## Release linkage

A release profile with source acquisition derives a canonical observation subject.
The subject omits locator and mutable reference hints.
It includes the observation schema, identity, content BLAKE3, profile, revision, and non-claim.

The subject is part of the canonical release-evidence manifest.
The existing release-attestation signature covers that manifest identity.
Mantle does not define a source-observation signer role.
Unknown source signatures cannot satisfy release, review, witness, or ownership policy.

Source review remains optional and separate.
Cairn and Valence remain the owners of review workflow and evidence identity.

## Claim boundary

A valid observation proves only that the admitted facts bind to the supplied content identity.
It does not prove source ownership, upstream intent, review quality, license compliance, build correctness, or release eligibility.

A valid ingest plan proves only one bounded local transition.
It does not prove source trust or external persistence.
Release and witness policy still require their existing evidence and signatures.

## Checks

Run the focused checks from the repository root:

```bash
cargo test -p crunch-source-core
cargo check -p crunch-source-core --target wasm32-unknown-unknown
cargo test -p mantle --bin mantle source_bundle::monotonic_ingest::tests::
cargo -Zscript scripts/check-source-observation-architecture.rs --self-test
cargo -Zscript scripts/check-source-observation-architecture.rs --root .
cargo -Zscript scripts/check-machine-schema-contracts.rs
```

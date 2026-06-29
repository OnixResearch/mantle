## Why

The current successful Aspen witness replay proves that a separate machine can rebuild the published binary from the release request's copied source archive. That is useful but still narrower than an independent-source witness claim: the witness did not acquire source material from an origin named by the release evidence.

Mantle needs a fail-closed way for release evidence to record an external source archive origin and for `mantle release witness-rebuild` to fetch that origin itself, verify the fetched bytes against the release manifest's BLAKE3 source digest, and only then run the rebuild workflow.

## What Changes

- Add an optional release-manifest source acquisition record for an external source archive URL bound to the existing `source_archive.digest_blake3`.
- Add release creation CLI flags to record that source archive URL when producing release evidence.
- Add a witness rebuild gate that requires independent source acquisition and rejects requests without a source acquisition record.
- Teach witness rebuild to fetch the external source archive into witness scratch, verify the BLAKE3 digest before extraction, and record source acquisition status in audit metadata.

## Impact

- **Files**: `crates/crunch-release-core/src/manifest.rs`, `src/release_evidence.rs`, `src/release_cmd.rs`, `src/main.rs`, `src/witness_rebuild.rs`, docs/evidence under this change.
- **Testing**: positive and negative core manifest tests, release evidence bundle tests, witness rebuild source-acquisition tests, focused cargo tests, Cairn validation/gates, and an evidence transcript that states bounded non-claims.

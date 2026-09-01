# Native observation v16: profile-bound host-source authority

Task-ID: I3
Covers: bootstrap_inventory.source_built_mantle_fixed_point

## Result

Profile v28 passed source-bundle verification with six added host-tool fetch
records. Proof preparation then failed before construction. The old validator
required the materialized profile records to equal only the native and StageX
manifest union:

```text
expected=53, profile=59
```

This rule excluded the declared Rust host-tool sources that I3 requires.

## Repair

The validator now requires the native and StageX union as an exact subset.
Each expected record must exist and be byte-for-byte equal. Additional records
must be validated fetch inputs. The independently expected profile BLAKE3 and
the proof plan's combined source-manifest BLAKE3 bind all added records.

The validator still rejects duplicate identities, missing bound records,
mismatched bound records, and non-fetch authority. Positive and negative tests
cover these boundaries. VibeThinker found no authority blocker under these
conditions and requested the non-fetch negative test, which is present.

## Evidence

- Failed attempt: `/home/brittonr/.cargo-target/mantle-source-built-fixed-point-runs-v16a/`
- Blocker: `materialized source record set differs from the exact native and StageX union`
- Plan status: not published because preparation failed first

## Non-claim

This evidence proves one preparation failure and its bounded authority repair.
It does not prove provider construction, the Rust provider, either Mantle stage,
fixed-point equality, compiler correctness, or release eligibility.

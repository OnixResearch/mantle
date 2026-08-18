# Source-built fixed-point profile v29

Task-ID: I3
Covers: bootstrap_inventory.source_built_mantle_fixed_point

## Result

Profile v29 contains 66 records. Its manifest BLAKE3 is:

```text
b2aff22f6a552a90f866ee3da29f6af13763329dc0cfbe453f2f70a832496bc5
```

The profile path is:

```text
/home/brittonr/.cargo-target/mantle-source-built-fixed-point-profile-v29-profile-bound-host-sources-20260804.json
```

## Construction

The bounded refresh preserved 65 non-Mantle records from profile v28.
It replaced the single Mantle source record with commit `ddc5f9a8`.
It added no new record. The replacement source includes the profile-bound
host-fetch authority validator.

The replacement Mantle source-record BLAKE3 is:

```text
a846a99ca46c2fb659f41f34a1e1b80cc8df009a8891edfb2830277cbee181a9
```

## Validation

Pueue task `7810` published the profile without replacement.
Pueue task `7815` independently verified it as:

```text
readiness=Ready missing=0 stale=0 unsupported=0 untrusted=0
```

## Non-claim

This profile proves source availability and identity only. It does not prove
provider construction, either Mantle stage, fixed-point equality, compiler
correctness, or release eligibility.

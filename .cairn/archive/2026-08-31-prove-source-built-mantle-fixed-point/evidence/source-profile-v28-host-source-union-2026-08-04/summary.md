# Source-built fixed-point profile v28

Task-ID: I3
Covers: bootstrap_inventory.source_built_mantle_fixed_point

## Result

Profile v28 contains 66 records. Its manifest BLAKE3 is:

```text
a15d70a3c74096fc6ed4f6c52e519d85f85c8318f2445e851a88bdfa3c6ab69c
```

The profile path is:

```text
/home/brittonr/.cargo-target/mantle-source-built-fixed-point-profile-v28-host-source-union-20260804.json
```

## Construction

The bounded refresh preserved 59 non-Mantle records from profile v27.
It replaced the single Mantle source record with commit `1876bc41`.
It added six materialized fetch records from the complete Rust host-tool union.
It did not add constructed provider or store-output authority.

The replacement Mantle source-record BLAKE3 is:

```text
d19a9f095bb3a6b952e78348d9457a8a5d91b69fb4e537d1c5ba50f9ea542ac7
```

## Validation

Pueue task `7780` published the profile without replacement.
Pueue task `7782` independently verified it as:

```text
readiness=Ready missing=0 stale=0 unsupported=0 untrusted=0
```

## Non-claim

This profile proves source availability and identity only. It does not prove
provider construction, either Mantle stage, fixed-point equality, compiler
correctness, or release eligibility.

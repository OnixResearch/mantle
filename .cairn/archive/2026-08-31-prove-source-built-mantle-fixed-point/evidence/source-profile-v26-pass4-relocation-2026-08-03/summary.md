# Source-built fixed-point profile v26

## Result

The regenerated profile contains 60 records.
Its manifest BLAKE3 is:

```text
1a7e3fcce3b1d00956d78845fdc6ae3fe41e6799c806b426ad0a489c6b29381d
```

The profile path is:

```text
/home/brittonr/.cargo-target/mantle-source-built-fixed-point-profile-v26-pass4-relocation-20260803.json
```

## Bound source change

The profile binds commit `cfc0238a` and the GCC pass4 published-tool relocation repair.
It retains the exact 60-record native and StageX source union from verified profile v25.
It regenerates the Mantle source record from the clean worktree and keeps provider outputs outside source authority.

## Validation

Pueue task `7524` generated the profile.
Pueue task `7530` independently verified it as:

```text
readiness=Ready missing=0 stale=0 unsupported=0 untrusted=0
```

## Non-claim

The profile proves source availability and identity only.
It does not prove provider construction, the Mantle fixed point, compiler correctness, or release eligibility.

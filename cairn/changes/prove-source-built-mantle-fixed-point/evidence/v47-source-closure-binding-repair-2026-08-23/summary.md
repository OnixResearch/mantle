# V47 source-closure binding repair

## Question

Can the final v2 receipt classify successfully after the receipt source-count repair?

## Inspected evidence

V47 used source commit `a7574778`, source profile BLAKE3 `40378afddf3dc2f9af790e223d25b0400b0c63123c0af303faafbcdd8bd49b7c`, and promoted checkpoint `3894008488d95be470c97ebe6994eda40af98564dd0d69ffd05ae801693ba6b3`.

Checkpoint restoration and native-provider admission succeeded. The proof materialized a 17-member source-built toolchain closure.

Both strict Cargo-free stages succeeded. Their Mantle binaries matched at BLAKE3 `8e06c4798391cdea850e5efdd090db3300e23babdefb0a094366e354fd0a8e8c`. The proof reported `strict_proof_admission: true`.

Receipt construction passed the eight-leaf source count, then failed generic v2 classification:

```text
MissingGenuineRebuildEvidence: rebuild descriptor source closure does not bind receipt source
```

The receipt uses the plan's aggregate source-authority BLAKE3 as `source_blake3`. The descriptor contained all eight source leaves, but no source entry carried the aggregate digest.

Three candidate repairs were inspected. Using only the Mantle source leaf would narrow receipt authority. Changing generic release-core classification would couple that core to Mantle's source framing. Replacing all leaves with one root would hide named source authority.

A focused classification regression now passes with an aggregate root and fails with leaves only. The full focused receipt test set passes 9 tests, with 2 preserved-fixture tests ignored. All 235 release-core tests also pass. Isolated focused Clippy passes with the two documented unrelated baseline allowances.

## Decision

Keep the aggregate receipt source identity. Add a `source-authority-closure` root to the descriptor while retaining all eight leaves.

Bind the same root in approved-read identities. Keep leaf-role completeness in the fixed-point plan core.

## Owner

The fixed-point plan owns source-authority framing and leaf completeness. The receipt layer owns conversion into the generic content-bound rebuild descriptor.

## Next action

Build and transfer the aggregate-root repair. Refresh the Mantle source record, then rerun the existing promoted checkpoint.

## Non-claims

V47 proves checkpoint restoration, strict admission, and fixed-point equality. It does not prove the final v2 receipt because generic classification failed closed.

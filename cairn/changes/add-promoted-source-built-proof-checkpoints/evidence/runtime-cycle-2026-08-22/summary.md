# Promoted provider checkpoint runtime cycle

## Question

Can a stopped promoted proof publish its completed provider stages, then let a fresh proof restore them and continue at stage1?

## Inspected evidence

The V41 paired source profile verified at BLAKE3 `e8d005138dc923377545ffa057a48d8cf58e33fd607012efd83cf44f3670de3e` with zero missing, stale, unsupported, or untrusted records.

Before import, task `280` retained 75 Rust-provider construction files and removed only 138,890,099,213 bytes of Rust build scratch. Task `282` then removed only 21,856,285,975 bytes of native state and 6,363,596,095 bytes of stage1 execution outputs. Plans, receipts, logs, source authority, StageX evidence, and all final providers remain in the stopped attempt.

Remote task `283` used the new source profile and revalidated the native provider at BLAKE3 `63d9bc23cfcc232726527c141132ea35bc7ef9adf4ed7a952b5baa62bd466ed9`. It then failed closed before checkpoint publication:

```text
imported StageX provider receipt is missing: .../provider-receipt.json
```

The preserved provider contains the producer-owned receipt at:

```text
share/crunch-bootstrap/stagex-lineage-receipt.json
```

The import and restore reconstruction paths now use `stagex_provider::PROVIDER_RECEIPT_RELATIVE_PATH`. A positive test requires the published path. A negative test rejects the obsolete root path.

## Decision

Keep the producer-owned StageX receipt layout. Repair checkpoint reconstruction rather than copying or inventing a second receipt.

## Owner

The promoted checkpoint integration shell owns report reconstruction. The StageX provider owns its receipt layout and exported relative-path constant.

## Next action

Build and transfer the repaired orchestrator, refresh the Mantle source record, rerun the stopped-attempt import, then start a fresh checkpoint-backed proof.

## Non-claims

Task `283` did not publish a checkpoint. No provider restore, stage1 continuation, fixed point, or final receipt is proven yet.

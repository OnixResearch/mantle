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

The V42 profile then verified at BLAKE3 `e99cd591cf76435d9e5a10b7b70dadfb7a35b9bcf174b9ede73830be4d129dd2`. Task `285` imported checkpoint `3894008488d95be470c97ebe6994eda40af98564dd0d69ffd05ae801693ba6b3`. Its manifest records promoted origin, four completed stages, seven payloads, no authority violations, and no fallback events.

Task `287` admitted and restored that checkpoint into a fresh proof root. It revalidated the native provider, but rejected stage1 before execution because the exact closure payload still named the origin attempt's absolute Rust and native provider paths.

The restore path now keeps the exact closure payload under `provider-checkpoint-origin/`. It derives a current closure from the restored providers. A pure validator requires identical schema, seeds, member identities, content digests, trust, source, build receipts, and provider-relative paths. Only the two absolute provider roots may change.

The V43 profile verified at BLAKE3 `7a1cf089b63282efba8c4f42836d77144155f3c5daf035ee383a512693134596`. Task `290` admitted and restored checkpoint `3894008488d95be470c97ebe6994eda40af98564dd0d69ffd05ae801693ba6b3`. Its closure relocation report and restore transcript were present before stage1.

Both strict Cargo-free stages succeeded. Their Mantle binaries matched at BLAKE3 `4e48a03ef41c18bf98525c9c72d20b99e446db546ede971dc2c5f0118c3ecda7`. The run reported `strict_proof_admission: true`.

Final receipt construction then failed closed. The plan named `stagex-provider-publication`, but evidence used `stagex-provider`. The same stale alias pattern affected `stage1` and `stage2`. Receipt stage IDs now come from the same canonical constants as the plan.

## Decision

Keep the producer-owned StageX receipt layout. Repair checkpoint reconstruction rather than copying or inventing a second receipt. Keep the original closure as immutable evidence, but use a validated path-rebound closure for current stage execution. Use the immutable plan's six stage identifiers in final evidence.

## Owner

The promoted checkpoint integration shell owns report reconstruction. The StageX provider owns its receipt layout and exported relative-path constant. The fixed-point plan owns canonical stage identifiers.

## Next action

Build and transfer the receipt stage-ID repair, refresh the Mantle source record, and rerun the checkpoint-backed proof. The existing checkpoint remains valid because no provider authority changed.

## Non-claims

Task `290` proves checkpoint restore, stage1 continuation, stage2 completion, strict admission, and fixed-point equality. The final v2 receipt is not proven yet.

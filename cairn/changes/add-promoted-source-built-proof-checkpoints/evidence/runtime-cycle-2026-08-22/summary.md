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

The V44 profile verified at BLAKE3 `3042005943ca8d1343da2231c3126bbbcb9ea232f27e509833e04dcf6bc40728`. Task `293` restored the checkpoint, but the remote pueue user service restarted before stage1 finished. No proof process survived. The attempt retained `status: running` without a validator blocker, so this was an external interruption.

V46 used a detached `nohup` and `setsid` wrapper. It restored the same checkpoint and completed both strict Cargo-free stages. Their binaries matched at BLAKE3 `542e51fd615fb2916b923ff5a243434e562baeea4adcbd2726d2d846b03cdc87`.

V46 then failed closed during receipt construction. The plan carried all eight required source roles, but the receipt layer retained an obsolete private count of six.

V47 consumed the shared count and completed both strict Cargo-free stages. Their binaries matched at BLAKE3 `8e06c4798391cdea850e5efdd090db3300e23babdefb0a094366e354fd0a8e8c`.

Generic v2 classification then rejected the descriptor because its eight leaves did not include the aggregate receipt source digest. Receipt construction now adds one aggregate source-authority root while retaining every leaf.

V48 restored the same checkpoint and completed both strict stages. Its binaries matched at BLAKE3 `2403feed4dba0959d1dbb1a202777827e82361492f19a7fc8c642f3b138f599c`. The final receipt verified with verdict `self-rebuild-match`, and all four provider stages retain `restored-checkpoint` origins.

The proof establishes the historical seven-payload checkpoint composition through final receipt publication. It does not establish the separate root child-action trust report required by the broader Cairn task.

The later root action-trust boundary supersedes that checkpoint for future promotion. Checkpoint schema v2 requires native action-plan and reconciliation payloads and uses a new lookup-policy identity. This does not alter V48's historical receipt; it requires a new checkpoint before another promoted proof.

## Decision

Keep the producer-owned StageX receipt layout. Repair checkpoint reconstruction rather than copying or inventing a second receipt. Keep the original closure as immutable evidence, but use a validated path-rebound closure for current stage execution. Use the immutable plan's six stage identifiers in final evidence. Use the fixed-point core's source-role count in the receipt layer. Bind the aggregate receipt source through a descriptor root without removing source leaves.

## Owner

The promoted checkpoint integration shell owns report reconstruction. The StageX provider owns its receipt layout and exported relative-path constant. The fixed-point plan owns canonical stage identifiers and source-role completeness. The receipt layer owns the aggregate source-closure root.

## Next action

Keep V48 as the successful checkpoint-composition evidence. Implement root child-action planning and the derived trust-report view as a separate bounded authority layer.

## Non-claims

V48 proves checkpoint restore, stage continuation, strict admission, fixed-point equality, and final v2 receipt publication. It does not prove complete child-action reconciliation or broader bootstrap parity.

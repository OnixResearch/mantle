# V48 promoted fixed-point success

## Question

Does the checkpoint-backed V48 proof produce and verify the promoted v2 deterministic receipt without weakening source, provider, hermeticity, or fallback policy?

## Inspected evidence

V48 used source commit `01664f45` and release-orchestrator BLAKE3 `f1626ba807d97e2659a1d9addabd6c8115354e391d377bc40d05fec12b35962a`. Rsync checksum parity was exact, with root-anchored exclusions.

The source profile verified before and after the proof at BLAKE3 `f1c4c9d64c0ba86fc7ea8015cf96beb31f726d3f90a5554a95bcb234beafea77`. Both checks reported `Ready` with zero missing, stale, unsupported, or untrusted records.

The proof admitted promoted checkpoint `3894008488d95be470c97ebe6994eda40af98564dd0d69ffd05ae801693ba6b3`. It restored the first four provider stages and validated closure relocation for all 17 members. Each restored stage retained its original execution-evidence digest.

Stage1 and stage2 each executed 789 units with zero failed units. Both smoke checks returned zero. Their Mantle binaries matched at BLAKE3 `2403feed4dba0959d1dbb1a202777827e82361492f19a7fc8c642f3b138f599c`.

The fixed-point metadata records strict hermeticity, admitted proof eligibility, zero seed exceptions, and an enforced 17-member source-built closure. Stage evidence records six complete stages, four restored checkpoint origins, two current executions, zero authority violations, and zero fallback events.

The final receipt records:

- schema `mantle-deterministic-proof-receipt-v2`;
- verdict `self-rebuild-match`;
- receipt BLAKE3 `698e0eea069c7ba2fbe7531d3dea214a841d63e33d85a508545fad21cf71193e`;
- plan BLAKE3 `fc61749204d950065ae1c0c40a9272a64040a7a4365e4e60d7bb25ea6fb5e464`;
- proof-bundle BLAKE3 `c97ff46ff25df813bd893dec84fd7f01244d0eabc241e53ae9ae5388a61011c2`;
- native-provider BLAKE3 `63d9bc23cfcc232726527c141132ea35bc7ef9adf4ed7a952b5baa62bd466ed9`;
- Rust-provider BLAKE3 `d88f60b1bd5291b5fc42ebd0247ccb9288d52ca7531046e489bfe73706f24cd6`;
- relocated closure BLAKE3 `f98d1480859a582864fdd7c7e23c443dfb57760e2c540fbd1d636a2ea8051912`;
- source-authority BLAKE3 `b068abe4da3007dd1544af982d2a74a8c9eb386fc9f02ece90a04e6d175649ab`.

The descriptor contains all eight source leaves and the aggregate source-authority root. Both runs bind the same approved reads, output digest, descriptor, and authority-plan identities. They record no substitutions, hermeticity events, or authority violations.

The successful output `attempt-status.json` has `status: complete` and no blocker. Both `latest` and `latest-source-built-fixed-point` point to V48.

A separate Rust validator canonicalized the copied receipt, recomputed its BLAKE3, checked the stage and closure summaries, and rejected a wrong receipt digest.

The command required by Cairn I5, `mantle --json bootstrap trust-report --proof-root <path>`, is not implemented. Clap rejects `--proof-root`. No root-scoped action-trust report or complete planned-versus-observed child-action reconciliation is present.

## Decision

Accept V48 as the verified promoted source-built fixed-point receipt. Do not promote it to the broader root action-trust claim.

Keep Cairn I3, I4, I5, and V2 open until the action-trust plan, reconciliation, and operator view exist. This gap does not invalidate the verified fixed-point receipt.

## Owner

The fixed-point shell owns provider restoration and stage execution. The receipt layer owns v2 receipt verification. The pending action-trust component owns full child-action planning and reconciliation.

## Next action

Implement the root-scoped action-trust plan and derived `bootstrap trust-report` view without changing receipt authority. Then run the remaining focused and lifecycle gates.

## Non-claims

V48 does not prove compiler correctness, seed correctness, kernel isolation, complete Cargo compatibility, independent rebuild agreement, bit-for-bit release reproducibility, deployment success, or release eligibility.

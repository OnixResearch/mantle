# V46 rebuild source-count repair

## Question

Can the checkpoint-backed proof finish its final receipt after the canonical stage-ID repair?

## Inspected evidence

The detached V46 wrapper started from source commit `5fb4ea42`. It used source profile BLAKE3 `3042005943ca8d1343da2231c3126bbbcb9ea232f27e509833e04dcf6bc40728` and promoted checkpoint `3894008488d95be470c97ebe6994eda40af98564dd0d69ffd05ae801693ba6b3`.

Checkpoint restoration completed without provider reconstruction. The proof admitted the native provider at BLAKE3 `63d9bc23cfcc232726527c141132ea35bc7ef9adf4ed7a952b5baa62bd466ed9` and materialized a 17-member closure.

Both strict Cargo-free stages succeeded. Their Mantle binaries matched at BLAKE3 `542e51fd615fb2916b923ff5a243434e562baeea4adcbd2726d2d846b03cdc87`. The proof reported `strict_proof_admission: true`.

Final receipt construction then failed closed:

```text
source-built fixed-point receipt blocked: rebuild descriptor source input count is incomplete
```

The immutable V46 plan contains eight source inputs. `source_built_fixed_point.rs` already requires those eight named roles. `source_built_fixed_point_receipt.rs` retained an older private count of six.

The focused receipt baseline passed 6 tests, with 2 ignored preserved-fixture tests. After the repair, receipt tests passed 7 tests, with 2 ignored. The fixed-point plan tests passed all 10 tests.

Focused Clippy reached unrelated existing findings in `src/remote_nominal.rs` and `src/nix_free_demo_cmd.rs`. It did not report a finding in either changed fixed-point file.

## Decision

Keep one source-role count under the fixed-point plan core. Make receipt construction consume that count instead of retaining a second value.

Add positive coverage for the complete count. Add negative coverage for missing and extra inputs.

## Owner

`src/source_built_fixed_point.rs` owns the required source-role contract. `src/source_built_fixed_point_receipt.rs` converts that validated authority into the rebuild descriptor.

## Next action

Build and transfer the repaired orchestrator. Refresh the Mantle source record, then rerun the existing promoted checkpoint.

## Non-claims

V46 proves checkpoint restoration, strict admission, and fixed-point equality. It does not prove the final receipt because receipt construction failed closed.

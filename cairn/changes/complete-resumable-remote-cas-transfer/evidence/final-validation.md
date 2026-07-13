# Final focused validation

Date: 2026-07-12

All Cargo targets were outside the checkout under `/tmp/mantle-agent-transfer-target-final`. The documented clang/mold/pkg-config/OpenSSL environment and `SNIX_BUILD_SANDBOX_SHELL=/bin/sh` were supplied.

## Current passing rails

Pueue task 735 ran the complete focused chain after the final runtime commit:

```text
rustfmt --edition 2024 --check \
  crates/crunch-build/src/distributed/remote_transfer.rs \
  src/remote_transfer.rs src/remote_farm_config.rs
git diff --check
cargo test -p crunch-build --lib distributed::remote_transfer::tests:: -- --nocapture
cargo test -p crunch-store --lib
cargo test -p crunch-delta --lib --tests
cargo test -p mantle --bin mantle remote_transfer::tests:: -- --nocapture
cargo test -p mantle --bin mantle remote_farm_config::tests:: -- --nocapture
cargo test -p mantle --bin mantle remote_build::tests:: -- --nocapture
```

The `&&`-chained task exited successfully. Its final leg reported:

```text
test result: ok. 104 passed; 0 failed; 0 ignored; 0 measured; 1283 filtered out; finished in 1.01s
```

Separate current summaries make the focused package counts explicit:

- Pueue task 747: transfer core `11 passed; 0 failed; 527 filtered out`.
- Pueue task 746: `crunch-delta --lib --tests` `36 passed; 0 failed`.
- Pueue task 753: serialized `crunch-store --lib` `199 passed; 0 failed`.
- Pueue task 719: transfer shell/process rails `13 passed; 0 failed`; both subprocess sentinels separately reported `1 passed; 0 failed`.
- Pueue task 673: typed remote policy/config `14 passed; 0 failed`.

Pueue task 736 used the canonical non-symlink Cairn checkout and passed this `&&`-chained packet:

```text
nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- validate --root .
nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- gate proposal complete-resumable-remote-cas-transfer --root .
nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- gate design complete-resumable-remote-cas-transfer --root .
nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- gate tasks complete-resumable-remote-cas-transfer --root .
```

The final tasks receipt reported `"issues": []`, `"valid": true`, and `"verdict": "PASS"`.

## Validation triage retained for honesty

- Task 698 used plain `cargo test -p crunch-delta`, which compiled the feature-gated `vectorcdc_candidate_compare` example without its feature and failed. The intended first-party library/test rail is `--lib --tests`; tasks 735 and 746 prove that corrected rail.
- Task 702 addressed Cairn through the legacy symlink and Nix rejected the flake path as a symlink. Tasks 736 and the post-completion packet use the canonical checkout path.
- Task 745 exposed an existing `crunch-store::completeness` global-marker race under parallel libtest (`198 passed; 1 failed`). The full package had passed inside task 735; task 753 reran all 199 tests serialized and passed. No transfer implementation code or store test was changed to hide the race.
- `cargo-kani` is unavailable. Kani source harnesses exist, but execution remains explicitly unclaimed.

## Task 735 completion boundary (historical)

That packet proved the focused transfer core/shell, store, delta, existing remote-build behavior, bounded child-process 8 MiB shell rail, formatting, diff hygiene, typed Nickel policy, and then-current Cairn validation/gates. It did not sync or archive the change, claim Kani execution, or collapse transfer completion into output admission.

At that point, an adversarial call-graph audit correctly found that the packet did **not** satisfy `r[verification_evidence.production_transfer_completion_claim]`: the 8 MiB child-process rail called `execute_prepared_remote_transfer` directly while the production framed path still used whole payload vectors. The later task 1017 evidence below supersedes that production-path blocker, but not the still-unproven production delta fallback or final broad completion rail.

## Production integration evidence (pueue task 1017)

Task 1017 created its isolated TMPDIR first and ran this exact `&&`-chained packet after commits through `a6a8d615`:

```text
cargo test -p mantle --bin mantle remote_build::tests:: -- --nocapture
cargo test -p mantle --bin mantle remote_transfer::tests:: -- --nocapture
cargo test -p mantle --test remote_transfer_production -- --nocapture
```

The full pueue log reported:

```text
running 109 tests
test result: ok. 109 passed; 0 failed; 0 ignored; 0 measured; 1318 filtered out; finished in 1.01s
running 17 tests
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1426 filtered out; finished in 0.01s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1426 filtered out; finished in 0.15s
test result: ok. 17 passed; 0 failed; 0 ignored; 0 measured; 1410 filtered out; finished in 0.22s
running 3 tests
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.01s
```

The three production multi-process tests prove:

- a production output transfer is interrupted after a durable bounded chunk, restarts from durable coordinator/receiver state, reports reused bytes, sends the missing remainder, and imports byte-identical output through ordinary signed PathInfo/store admission;
- a production input upload crosses multiple chunks, is interrupted after a durable acknowledgement, restarts from the same checkpoint, transfers the missing remainder, and still reaches ordinary output admission; and
- a ticket upload quota rejects before any transfer checkpoint or output admission.

The 109 remote-build tests additionally cover pre/post-write fence validation, stale-final-chunk invalidation before acknowledgement/completion, stale input cutoff before source disclosure, excess-credit rejection with zero disclosure, bounded child-timeout return/classification, and ordinary streamed admission. The 17 transfer-shell tests and their two subprocess legs cover resumable/security behavior including tampered receiver facts and no-follow authority-state handling.

This evidence checks the production streaming replacement and production interruption/resume tasks. At task 1017 time it did **not** yet prove production fallback, current store/delta package rails, a production-scale 8 MiB transfer, or Kani.

## Production fallback, scale, and current package evidence

Pueue task 1140 ran the public production capability path plus current package rails:

```text
cargo test -p crunch-store --lib -- --nocapture
cargo test -p crunch-delta --lib -- --nocapture
cargo test -p mantle --test remote_transfer_production -- --nocapture

running 199 tests
test result: ok. 199 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.41s
running 36 tests
test result: ok. 36 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.10s
running 5 tests
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.29s
```

Pueue task 1166 independently reran serialized `crunch-store --lib`, `crunch-delta --lib --tests`, all 109 `remote_build::tests::`, and the five production tests. It reported 199, 36, 109, and 5 passing tests respectively, with zero failures.

The two added production rails prove:

- the release-reachable public `mantle build --remote-delta` option offers delta/full/streaming capabilities while the production server offers the same set; because no production delta runtime is bound, negotiation deterministically reports `mode = "full"` and `fallback_reason = "delta-unavailable"`, then streams the full NAR in multiple bounded chunks and reaches ordinary signed PathInfo/store admission; and
- an 8 MiB output crosses more than 100 receiver-acknowledged chunks, reports streaming with no fallback, imports through ordinary admission, and is byte-identical to the requested source.

This proves the production fallback and production-scale rails without claiming a delta hit. Pueue task 1185 then passed focused Rustfmt checks for `src/main.rs` (without cascading into child modules) and `tests/remote_transfer_production.rs`, followed by `git diff --check`. Kani execution remains unclaimed and is not part of the final task's exact command list.

## Current lifecycle packet

Pueue task 1196 ran current Cairn validation and the proposal, design, and tasks gates without sync or archive. The exact receipts were:

```text
validate: "specs_validated": 32, "valid": true
proposal: "receipt_hash": "3f502f1b7a0d7b8792d8b634b48f2780339186f284d98b7d570b03e0fbbe63c8", "valid": true, "verdict": "PASS"
design: "receipt_hash": "70b00cb759b2b7fcc021018105ebedac0d1a9e784bf38dd506967479101d8f1e", "valid": true, "verdict": "PASS"
tasks: "receipt_hash": "77d6f451bd5f449d95c421b293aebe5826fd3369636233afab852f6f5c34f40f", "valid": true, "verdict": "PASS"
```

Together with tasks 1017, 1140, 1166, and 1185, this satisfies the exact final validation task. After checking that final marker, pueue task 1208 reran the tasks gate and reported `"issues": []`, `"receipt_hash": "5c1b10c1730ae5047f398f2f97330def4b4d5fdf8f1617d4b63d6a8d6046d384"`, `"valid": true`, and `"verdict": "PASS"`. No command in this packet synchronized specs or archived the active change.

## Historical post-completion Cairn packet

After checking the final task marker, pueue task 755 reran the same canonical-path `validate` plus proposal/design/tasks gate chain against the current tree. The final tasks receipt reported:

```text
"input_hash": "ca538f736a7fa43b5f0d7c60013eac91620519695392adc17477f78461c5795a"
"issues": []
"receipt_hash": "fe55e443f13d9929a26147b6b8e2b305e722a7bb3b33c43ecd38b6708d22d637"
"stage": "tasks"
"valid": true
"verdict": "PASS"
```

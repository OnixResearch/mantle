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

## Completion boundary

This packet proves the focused transfer/store/delta/remote-build behavior, bounded multi-process 8 MiB output rail, formatting, diff hygiene, typed Nickel policy, and current Cairn validation/gates. It does not sync or archive the change, push commits, claim Kani execution, or collapse transfer completion into output admission.

## Post-completion Cairn packet

After checking the final task marker, pueue task 755 reran the same canonical-path `validate` plus proposal/design/tasks gate chain against the current tree. The final tasks receipt reported:

```text
"input_hash": "ca538f736a7fa43b5f0d7c60013eac91620519695392adc17477f78461c5795a"
"issues": []
"receipt_hash": "fe55e443f13d9929a26147b6b8e2b305e722a7bb3b33c43ecd38b6708d22d637"
"stage": "tasks"
"valid": true
"verdict": "PASS"
```

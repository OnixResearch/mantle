# Cargo unit dynamic-plan signed replay — 2026-10-11

Mantle's opt-in two-package Cargo-unit producer built a signed dynamic plan in a fresh publisher store, executed the app, rejected an unknown root before running the consumer, compared the app with same-input locked offline Cargo, and imported the signed results into a separate initially empty client. The maintenance owner is Mantle (`scripts/rust-unit-plan-replay/`); the reusable capability is bounded per-package source slicing and lazily scheduled Rust unit outputs, not whole-workspace Cargo replacement.

Command: `CU_B=$HOME/mantle-cairn-finish/tmp/rust-efficiency-20261011/cu-unit-final-v5 CU_STORE_NAMESPACE=cu-final5 nix --option min-free 0 --option max-free 0 develop --offline --no-write-lock-file -c bash scripts/rust-unit-plan-replay/cu-replay.sh`. Detached pueue task 902 exited 0 and printed `rust-unit-efficiency-proof: fresh, unchanged, package-edit, outside-edit, wrong-root, matched offline Cargo behavior, signed clean client PASS`. Full captured transcript: `$HOME/mantle-cairn-finish/logs/rust-eff-dynamic-final-v5-20261011.log` (SHA-256 `9b15af0f7c39e9df559e6a8248b53dce019738f7e1faa410b74b7de481e9c1fa`). Bounded per-run JSONL: `$CU_B/summary.jsonl` (SHA-256 `e57fb98214f59e2bd66406d09ec5da016c148a3e881ce8b87088f78633f5999f`). Runtime observations, not review-invented counts:

|Run|Exit|Built|Cached|Failed|Rust units executed|Rust units reused|
|---|---:|---:|---:|---:|---:|---:|
|01-fresh-baseline-producer|0|3|0|0|2|0|
|02-fresh-baseline-app|0|1|3|0|0|2|
|03-unchanged-rerun|0|0|4|0|0|2|
|04-app-edit-producer|0|2|1|0|1|1|
|05-app-edit-app|0|1|3|0|0|2|
|06-outside-edit|0|2|2|0|0|2|
|07-wrong-root|1|0|3|1|0|2|
|09-offline-cargo-baseline|0|1|0|0|0|0|
|08-clean-client|0|1|3|0|0|2|

The wrong-root binding rejected `plan-output-root-missing`, did not execute `app`, and reported no built outputs. The successful unit-plan and offline-Cargo app executions both had exit 0 and identical stdout and stderr; executable byte hashes differed, so raw binary parity is **not** claimed. The fresh client imported 58 signed input PathInfos and six separately exported signed publisher results, reused both Rust units, and rebuilt the app verifier. Producer outcomes may be omitted or appear cached in a report; the replay compares the actual two unit outcomes and the consumer result rather than asserting an incidental cached-total count. An outside-package edit executes zero Rust units even though its producer and consumer derivations rebuilt. Mantle's measured 925-unit, 711-package-source workspace exceeds the 256-source-slice bound: this proof does not admit it.

Non-claims: no full Cargo compatibility, LLVM/compiler correctness, byte-identical executables, cross-host reproduction, release readiness, or bootstrap correctness. No separate native build-script acceptance is implied by this two-package replay.

The shared first-party quality gate completed afterward: detached task 946 exited 0 on `scripts/check-first-party-quality.sh` with GCC bridge, selected rustfmt, strict workspace Clippy, and serialized first-party tests passing. Its transcript is `$HOME/mantle-cairn-finish/logs/rust-eff-first-party-quality-all-prereqs-20261012.log` (SHA-256 `047f73d61b57b59bd5c2162b5b98a551284cc6f5d04710a1d61ff2072d0f08eb`); both root binary suites reported 2566 passed, 0 failed, 68 ignored. The four changed first-party packages outside the standard fmt selection passed separate `cargo fmt --check -p` checks. The shared gate does not expand the bounded two-package application, native-script, bootstrap, or release claims above.

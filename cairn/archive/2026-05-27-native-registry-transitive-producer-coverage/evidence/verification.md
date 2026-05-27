# Verification Evidence: Native Registry Transitive Producer Coverage

Task-ID: native-registry-transitive-producer-coverage.V1
Covers: r[rust_package_planning.native_registry_transitive_producer_coverage]
Date: 2026-05-27
Decision owner: coding agent

## Oracle checkpoint: planning-gate evidence availability

Question: Did the initial `native-registry-transitive-producer-coverage` Cairn validation and stage gates really pass, and are their outputs repo-local enough for review?

Inspected evidence: pueue log for tasks `16` through `19`.

```text
Task 16: cairn validate -- valid=true, changes=1, specs_validated=2, issues=[]
Task 17: gate proposal -- verdict=PASS, valid=true, receipt_hash=2c6764f9892a514a367ee80674532090ea0cc7561c77bee13ad4a44a0e7e6f08
Task 18: gate design -- verdict=PASS, valid=true, receipt_hash=6cf9abef96418a5406d34e75773042efc28d7ea8883a8bd2e1ce801bc7d08749
Task 19: gate tasks -- verdict=PASS, valid=true, receipt_hash=e75dc6db048d71fccdc689e2d4640c36a32a017377d87a21b0a510c14e852779
```

Decision: treat the planning artifacts as validated, but keep this checkpoint in-repo so reviewers do not need hidden queue state to trust the earlier PASS claim.

Next action: rerun validation and gates after implementation and archive sync.

## Focused tests

Command evidence: pueue task `18` after same-family review remediation.

```text
cargo test -p mantle --bin mantle native_unit_graph_ -- --nocapture

running 8 tests
test rust_plan::tests::native_unit_graph_follows_selected_unit_dependencies_not_all_manifest_dependencies ... ok
test rust_plan::tests::native_unit_graph_blocks_selected_dependency_without_native_package_fact ... ok
test rust_plan::tests::native_unit_graph_blocks_selected_dependency_without_source_fact ... ok
test rust_plan::tests::native_unit_graph_keeps_selected_lib_dependency_with_host_target_sibling ... ok
test rust_plan::tests::native_unit_graph_fragment_blocks_mismatch_and_missing_edges ... ok
test rust_plan::tests::native_unit_graph_blocks_registry_dependency_without_lib_producer ... ok
test rust_plan::tests::native_unit_graph_adds_transitive_registry_producer_unit ... ok
test rust_plan::tests::native_unit_graph_fragment_feeds_supported_unit_derivations ... ok

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 455 filtered out; finished in 0.00s
```

Decision: positive and negative producer coverage is present. The negative cases verify missing native package facts, missing source facts, and missing lib producers all keep `unit_derivation_graph.ready=false`. The selected-dependency test verifies native producer closure follows Cargo's selected unit dependency artifacts instead of all manifest dependencies. The host-sibling regression verifies a selected normal `lib` dependency is retained when the dependency package also has a host/build-script target.

## Self-probe blocker movement

Command evidence: pueue task `15` after the selected-dependency remediation commit.

Summary artifact: `target/mantle-self-rust-plan-probe-after-621bbfd9/blocker-summary.txt`.

```text
head: 621bbfd9ca7b2fed5ba2f35053219523f7646b4c
git_status_short_bytes=0

probe_status=0
topology_execution=blocked
source_closure=true
native_registry_source_planning=true
native_git_source_planning=true
native_package_target_planning=true
native_unit_graph_planning=true
native_host_unit_graph_planning=true
unit_derivation_graph=true

blocker classes:
      1 missing-host-dependency-producer

topology blocker:
- missing-host-dependency-producer: no supported target producer lib unit for host dependency package registry+https://github.com/rust-lang/crates.io-index#rustversion@1.0.22

itertools producer units:

itertools dependency artifacts:
```

Decision: the `itertools@0.10.5` missing target producer blocker is resolved. Same-family review showed the earlier producer was from unselected manifest dependency expansion; after remediation there is no selected `itertools@0.10.5` dependency artifact and therefore no unsupported producer requirement. Remaining topology blocker moved to host-dependency producer coverage for `rustversion@1.0.22`.

Next action: handle host dependency producer coverage in a later Cairn change if requested.

## Cairn validation and gates before archive

Command evidence: pueue tasks `26` and final rerun `27` after all tasks were checked.

```text
cairn validate -- valid=true, changes=1, specs_validated=2, issues=[]
gate proposal -- verdict=PASS, valid=true, receipt_hash=1b724c51d09d0aafe4c3b2dabfe674b3a1021e78773f16f4d71a487fea77219f
gate design -- verdict=PASS, valid=true, receipt_hash=c2245edd1d87b1cfdd1094f10ed7d25e13cfd1bb0d1bb3bacd97fcd1cfde440c
gate tasks -- verdict=PASS, valid=true, receipt_hash=15f7ccb3a26a867d7d823923ac1b672e5fb16230570232e37e7f10cbf5b8fe67
git diff --check -- PASS
```

Decision: implementation evidence is ready to sync and archive.

## Sync and archive

Command evidence: pueue tasks `28` and `29`.

```text
cairn validate before archive -- valid=true, changes=1, specs_validated=2, issues=[]
cairn archive -- mutated=true, receipt_hash=af21ddac186e47b20334ed15472aff1a440350690d55c954b5e62a003f2a304a
git diff --check -- PASS
cairn validate after archive -- valid=true, changes=0, specs_validated=1, issues=[]
```

Decision: the change is synced into `cairn/specs/rust-package-planning/spec.md` and archived under `cairn/archive/2026-05-27-native-registry-transitive-producer-coverage/`.

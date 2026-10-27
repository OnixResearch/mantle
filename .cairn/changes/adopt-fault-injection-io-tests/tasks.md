## Fixture adoption

- [x] [serial] Add `fault-injection` as a pinned dev-dependency and record transport `crates.io`, plane `validation` in the dependency catalog. r[mantle.io_fault.fixtures]
- [x] [serial] Wrap store blob reads, NAR stream writes, post-build action-result record pre-link fsync, and cache writes with `fallible!` in test builds. r[mantle.io_fault.fixtures]
- [x] [serial] Add a trigger function that records the injection site for fixture assertions. r[mantle.io_fault.fixtures]

## Determinism

- [x] [serial] Add a setup guard storing an explicit counter value and restoring `u64::MAX` after each fixture. r[mantle.io_fault.determinism]
- [x] [serial] Keep `SLEEPINESS` at zero in every fixture and in CI configuration. r[mantle.io_fault.determinism]

## Boundary

- [x] [serial] Document that the dependency is test-only and that injection coverage grants no sandbox-hermeticity, store-correctness, or release-eligibility claim. r[mantle.io_fault.boundary]

## Verification

- [x] [parallel] Add a default-counter store/NAR/cache fixture and prove an unchanged actual build/cache cycle with the existing smoke scenario. r[mantle.io_fault.verification]
- [x] [parallel] Add negative fixtures at each wrapped site asserting the existing store/export error or cache-publication diagnostic, while preserving CA-required fail-closed publication. r[mantle.io_fault.verification]
- [x] [parallel] Add boundary fixtures asserting the crate/file/line annotation, default counter restoration, and fixture-serialized counter ownership. r[mantle.io_fault.verification]
- [x] [serial] Run package, workspace, Clippy, Cairn, and Nix checks; append the komora-io reference under `## References` in `README.md`; document non-claims. r[mantle.io_fault.verification]

## Current implementation and remaining evidence

The named store read, NAR stream copy, and action-result write/pre-link fsync wrappers already exist under `cfg(test)` in `crunch-store`; production expressions are unchanged. The fixture completion adds a thread-bound serialized guard, restoration assertions under the owner mutex, unarmed-thread isolation and unwind cases, the record/index failure-and-retry matrix, and complete NAR size/hash comparison using `crunch-nar`. `crunch-build` additionally covers ordinary diagnostic-only publication and fail-closed floating CA / CA-resolved IA intermediate publication with a real filesystem obstruction; the dependent must not execute. These statements describe source, not accepted runtime results, so the evidence-dependent boxes above remain open.

Original fixture-worker handoff commands (not executed by that worker):

```sh
cargo test --locked --offline -p crunch-store --lib io_fault_ -- --test-threads=4 --nocapture
cargo test --locked --offline -p crunch-build --lib io_fault_publication_is_diagnostic_only_for_ordinary_builds -- --nocapture
cargo test --locked --offline -p mantle --test smoke smoke_build_cached_on_second_run -- --exact --nocapture
```

The last scenario requires `/nix/store` and a working bwrap sandbox; its skip message is not build/cache-cycle evidence. Package/workspace, Clippy, Cairn, Nix, dev-edge dependency classification, and current-source lifecycle receipts remain the integration owner's gates. The existing source-only receipt is historical, not evidence for these new fixtures. No archive/sync or accepted-spec change is authorized by this source completion.

## Completed bounded runtime observation (2026-10-23)

`evidence/parallel-fault-and-finish-runtime-2026-10-23.json` retains the completed driver, exact child commands/environment overrides, result, executable pins, raw streams and BLAKE3. The release `crunch-store --lib io_fault_` command with four test threads completed **6 passed, 0 failed, 0 ignored**: default-counter store/NAR/cache, classified read/export faults, all four record/index write/fsync failure-and-retry phases, thread-owned isolation/serialization and caught-unwind restoration. The expected caught panic and independent kache disk-I/O warnings remain in raw stderr.

Real bwrap admission exited 0; release `smoke_build_cached_on_second_run` completed **1 passed, 0 failed, 0 ignored**, with no skip in either stream. Thus the reachable actual build/cache cycle is exercised, not still wholly unrun. Cargo compiled separate fixture/smoke executables; this driver did not BLAKE3-pin those test executables. Its immutable wait-request-repaired CLI pins bind the separately retained finish CLI observations, not a new exact-exec or final integrated cohort.

The store wrapped-site matrix is now observed for the recorded subject. Completed source-bound `crunch-build` ordinary/floating-CA/CA-resolved intermediate publication evidence, every-CI SLEEPINESS/dev-edge classification and final integration gates remain local actions. This worker launches no checks; Main owns integration, and store/action-result maintainers own interpretation/repeatability. No external blocker is asserted for executable local actions. The tracked receipt maps original criteria precisely; no checkbox, lifecycle acceptance, hermeticity/store-correctness or release claim changes.

## Retained individual task support (2026-10-26)

`evidence/fault-injection-individual-tasks-gate-v13-2026-10-26.json` records all six store fixtures, the ordinary/floating-CA/CA-resolved intermediate publication fixture and `smoke_build_cached_on_second_run` as `ok` in the tree-bound v13 workspace step (exit 0). It maps the seven newly checked tasks to source sites, trigger/guard assertions, current zero-SLEEPINESS fixture/CI settings and those named results. CI source excerpts do not claim that Main's later source-filter repair passed Nix checks.

The actual retained `target/cairn-finish-20261023/gate-env-probe-v13/probe.txt` records bwrap version/namespace exit 0 and Nix store exit 0. Together with the inspected smoke's prerequisite gate and cache assertions, this supports execution rather than treating a potential skip as success. It does not pin a new test executable or establish arbitrary fault coverage.

The full package/workspace/Clippy/Cairn/Nix task stays open. The default all-gates completion recorder still refuses v13 because flake exited 1 on omitted compile-time Python inputs, a repository failure. Only its explicit `--individual-fixtures` evidence mode was executed; it retains `all_gate_steps_passed=false` and does not perform lifecycle acceptance, spec sync or archive.

## Completed final verification (2026-10-26)

`evidence/final-verification-2026-10-26.json` closes the remaining verification task using the retained ordinary first-party gate, the later completed full x86_64-linux flake check (Pueue 40, exit 0), captured post-gate Cairn validation, and the existing README reference/non-claims. Exact named passes for the six store fixtures, publication-policy fixture and actual build/cache smoke remain bound to the v13 ordinary log; the root-package full-flake Nextest count is not substituted for library-fixture execution.

The ten selected current fault, smoke, dependency and CI subjects match the later passing full-gate snapshot. Nine also match v13; `flake.nix` changed and is covered by the later full gate. The v13 flake failure and prior supervisor interruption remain historical, not relabeled. This closes authored implementation/verification checkboxes only: parent semantic/lifecycle acceptance, accepted-spec sync and the authorized isolated archive workflow remain separate. No sandbox-hermeticity, store-correctness or release-eligibility claim follows.

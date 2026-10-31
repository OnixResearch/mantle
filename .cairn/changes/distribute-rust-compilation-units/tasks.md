# Tasks: Distribute Rust compilation units

Proposed work only. All implementation and acceptance tasks remain unchecked.
Source availability and lifecycle authoring gates do not accept the lane.

## Phase 1: Frontend admission

- [ ] [task:R1][serial] Retain the accepted native-pool and unit-plan prerequisites and a same-toolchain local baseline for the existing small plan/build-script fixtures; record current supported triples, bounds, root bindings and non-claims. r[distributed_compilation.rust_acceptance]
- [ ] [task:R2][after:R1] Express compiler execution system, artifact target, proc-macro ABI, script host/target facts, sysroots and target linker as frontend-owned generic inputs/requirements; reject unsupported combinations before transfer. r[distributed_compilation.rust_platform_roles]
- [ ] [task:R3][after:R2] Connect verified producer plan/source outputs and final plan-root consumers to native pool dispatch without a Rust-specific Worker/store branch or a host Cargo fallback. r[distributed_compilation.rust_unit_graph]

## Phase 2: Artifacts and identities

- [ ] [task:R4][after:R3] Carry complete library manifests, proc-macro runtimes, script `OUT_DIR`/flags/generated files, native roles and linker closures between workers; require current finished verification before any consumer executes. r[distributed_compilation.rust_artifact_barriers]
- [ ] [task:R5][after:R4] Preserve canonical unit/plan/action identities, metadata salts and source remapping across placement; integrate signed completed-artifact reuse and forbid cross-worker incremental/live workspace or provisional metadata reuse. r[distributed_compilation.rust_identity_reuse]

## Phase 3: Real compiler acceptance

- [ ] [task:R6][after:R5][parallel] Build identical small branching unit plans locally and on one/two real workers with cache-off cold runs; prove overlapping rustc execution, verify admitted final artifacts and execute the app against an independent expected result. r[distributed_compilation.rust_unit_graph] r[distributed_compilation.rust_acceptance]
- [ ] [task:R7][after:R5][parallel] Run a real proc-macro plus sandboxed build-script/generated-source/native-library application with the final link on a worker lacking prior inputs; verify correct host/target roles and complete transfer. r[distributed_compilation.rust_platform_roles] r[distributed_compilation.rust_artifact_barriers]
- [ ] [task:R8][after:R5][parallel] Perturb one package, proc macro, generated input, native provider and toolchain in separate exact-identity runs; inspect affected derivation/action/results and reuse of an unchanged independent branch, without asserting nominal unit labels alone. r[distributed_compilation.rust_identity_reuse]
- [ ] [task:R9][after:R6][after:R7][after:R8] Exercise wrong execution system/ABI, removed native input, missing transitive artifact, real script/rustc rejection, cancellation/worker loss and stale prior generated output; prove no successful final publication or premature consumer execution. r[distributed_compilation.rust_platform_roles] r[distributed_compilation.rust_artifact_barriers] r[distributed_compilation.rust_acceptance]
- [ ] [task:R10][after:R9] Repeat the admitted app from independent fresh stores and compare the same-plan local/pool artifacts plus final runtime semantics; preserve unsupported full-workspace/limit cells and the default offline Cargo lane's separate comparison contract. r[distributed_compilation.rust_acceptance]

## Phase 4: Verification and delivery

- [ ] [task:R11][after:R10] Run real supported CLI smokes, affected first-party formatting/Clippy/focused tests, no-std boundaries, Nickel/compatibility matrix rails and required repository gates; retain exact subject identities and raw outputs. r[distributed_compilation.rust_acceptance]
- [ ] [task:R12][after:R11] Update unit-plan examples, operator/lane docs, compatibility matrix and release notes with explicit pool selection, support cells, finished-artifact barriers, limits and required non-claims; run Cairn validation and all three authoring gates, then obtain acceptance and commit before separately authorized sync/archive. r[distributed_compilation.rust_acceptance]

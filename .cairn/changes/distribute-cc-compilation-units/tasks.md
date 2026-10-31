# Tasks: Distribute C and C++ compilation units

Planning only. All implementation and runtime acceptance tasks are unchecked.
The explicit unit-plan lane does not make arbitrary Make/Ninja graphs supported.

## Phase 1: Consumer baseline and frontend contract

- [ ] [task:C1][serial] Record the existing `greet` Make baseline and installed consumer behavior; extract one shared tracked source fixture for baseline and new lane, and add a bounded C++/generated-header companion with independent expected results. r[distributed_compilation.cc_acceptance]
- [ ] [task:C2][after:C1] Add a typed Nickel graph for declared compiler/execution/target/sysroot, source/include roots, generators, compile/archive/link/install actions, ordered args/environment, explicit edges and named outputs. r[distributed_compilation.cc_explicit_graph]
- [ ] [task:C3][after:C2] Implement pure bounded graph validation/lowering in a no-std frontend core and offline producer/helper adapter, reusing standard dynamic plans/placeholders/source admission and the existing compile classifier without changing generic scheduler/store semantics. r[distributed_compilation.cc_explicit_graph] r[distributed_compilation.cc_supported_fragment]

## Phase 2: Complete inputs and native execution

- [ ] [task:C4][after:C3] Bind complete ordered header-search trees, quote-relative paths, generated outputs, source slices and compiler/sysroot/runtime/helper closures; reject undeclared roots, escapes and unbound declared generator references before dispatch, cover newly shadowing headers, and keep entirely omitted inputs as fail-closed sandbox failures rather than inferred dependencies. r[distributed_compilation.cc_complete_inputs]
- [ ] [task:C5][after:C4] Admit the documented pinned C/C++ compile fragment, explicit depfile outputs and deterministic archive/link/install actions; reject unknown/side-effectful/LTO/host-tuned modes rather than alter ordinary local-driver forwarding. r[distributed_compilation.cc_supported_fragment]
- [ ] [task:C6][after:C5] Emit final root bindings and run all actions through native-pool goals and standard signed result admission; preserve action identity across placement and finished-object barriers before archives/links, without daemon or broker sockets in sandboxes. r[distributed_compilation.cc_artifact_execution]

## Phase 3: Real compiler acceptance

- [ ] [task:C7][after:C6][parallel] Run local/one-worker/two-worker C and C++ plans with real compilers, retain actual overlapping object lifetimes, verify archive/library use and execute installed programs against expected values and the Make baseline. r[distributed_compilation.cc_acceptance] r[distributed_compilation.cc_artifact_execution]
- [ ] [task:C8][after:C6][parallel] Perturb source, macro, header contents, new earlier-search header, root ordering, generator, compiler and sysroot; inspect action/input identities and expected runtime transitions, plus unchanged-action relocation without semantic identity churn. r[distributed_compilation.cc_complete_inputs] r[distributed_compilation.cc_artifact_execution]
- [ ] [task:C9][after:C6][parallel] Pair valid graphs with cycle/missing-edge/path-escape/undeclared-root/unknown-flag/unsupported-mode/limit negatives; after a successful run cause a real compiler error and reject prior objects, wrong-tool results, worker-loss/stale-fence outputs and any successful downstream publication. r[distributed_compilation.cc_explicit_graph] r[distributed_compilation.cc_supported_fragment] r[distributed_compilation.cc_artifact_execution]
- [ ] [task:C10][after:C7][after:C8][after:C9] Repeat from fresh independent stores and compare same-plan local/pool stable artifacts and declared Make/runtime/install semantics; retain all unavailable/failed cells and no bootstrap or arbitrary build-system support claim. r[distributed_compilation.cc_acceptance]

## Phase 4: Verification and delivery

- [ ] [task:C11][after:C10] Run actual frontend/build CLI smokes, affected first-party formatting/Clippy/focused tests, no-std boundaries, Nickel/inventory checks and required repository gates; retain exact inputs, commands, outputs and admission evidence. r[distributed_compilation.cc_acceptance]
- [ ] [task:C12][after:C11] Document the opt-in entry point, explicit graph adoption, supported tool/flag fragment, include identity, generator barriers and limits; update examples/index and release notes, run pinned Cairn validation and all authoring gates, obtain acceptance and commit before separately authorized sync/archive. r[distributed_compilation.cc_acceptance]

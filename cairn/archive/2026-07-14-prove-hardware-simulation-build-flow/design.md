## Context

The DVCon case study reports that Bazel improved a hardware verification workflow through demand-driven dependencies, content-based action caching, sandboxed dependency checking, and remote execution. Its strongest architectural lesson is not a Bazel-specific rule API: an RTL flow becomes reusable when generation, compilation, linking, and tests are explicit actions with complete inputs and independently reusable outputs.

Mantle already owns generic derivations, fixed-output fetchers, strict bwrap execution, `mantle-plan-v1`, build reports, CAS/PathInfo admission, and shared action-result work. The missing evidence is a maintained workload that composes those surfaces. The reference must not turn Verilog, Verilator, VIP, regressions, or expected DUT behavior into Mantle core concepts.

## Decisions

### 1. Keep the hardware profile outside Mantle core

**Choice:** Place the reference manifest, contracts, and helper functions under a cataloged hardware example/fixture subtree. Typed Nickel describes source packages, tops, source sets, tool cohort, generation options, compile/link parameters, smoke cases, expected result shape, and validation tier. It lowers to ordinary derivations and dynamic plans.

**Rationale:** Mantle should prove it can execute the graph while an HDL-specific library or higher frontend owns domain semantics.

### 2. Use one exact open-source tool cohort

**Choice:** The reference profile binds exact Verilator, C++ compiler, linker, runtime-support, and shell store inputs plus BLAKE3 cohort identity. The initial rail may consume an explicitly declared Nix-produced tool cohort, but evidence must label that seed boundary and may not claim Mantle bootstraps Verilator.

**Rationale:** Ambient `verilator`, compiler, include directories, or user configuration would make cache and invalidation evidence meaningless.

### 3. Make external source closure demand-driven

**Choice:** Use test-owned pinned local git repositories to model separately maintained IP/VIP packages. Only the selected top/testbench’s reachable fixed-output sources enter evaluation and realization. A sibling source carries a launch/fetch sentinel so tests can prove it was not acquired.

**Rationale:** Local repositories keep the validation deterministic while exercising the same source-identity and reachability decisions as remote pinned repositories.

### 4. Split generation, compilation, linking, and smoke actions

**Choice:** One sandboxed generation action elaborates the selected SystemVerilog top and emits generated C++ plus a declared `mantle-plan-v1`. The plan creates bounded per-translation-unit compile units and one link root over their exact outputs. Each smoke vector is a separate action over the linked simulator and a canonical case descriptor.

**Rationale:** This is the granularity needed for selective invalidation, local concurrency, shared action reuse, and later remote dispatch. Dynamic-plan bounds prevent generated file count or names from becoming unbounded scheduler input.

### 5. Treat smoke results as ordinary admitted outputs

**Choice:** A fixture-owned `mantle-hardware-smoke-result-v1` artifact records schema, case id, input vector, expected/observed bounded values, verdict, simulator ref, action ref, tool/source profile refs, bounded stdout/stderr refs, and non-claims. Exit status and result artifact must agree. Only passing, complete, admitted actions may publish a successful shared action result.

**Rationale:** Logs alone are unstructured and untrusted; a process exit alone does not provide a durable result identity. The artifact remains workload data rather than a new core test framework.

### 6. Reuse generic shared action-result admission

**Choice:** Clean-client reuse goes through `mantle-action-result-v1`, object completeness, PathInfo, receipt, signature, producer-policy, sandbox/network-policy, and reference-scan admission. Different otherwise admissible smoke outputs for one action ref produce the existing conflict diagnostic.

**Rationale:** An HDL-specific cache would duplicate and weaken Mantle’s generic trust boundary.

### 7. Measure work, not promised speedup

**Choice:** The rail emits a versioned benchmark/evidence bundle for fresh, selected-source-change, unrelated-source-change, and full-shared-hit runs. It records requested/executed/reused action counts by stage, transferred/reused bytes, selected invalidations, tool/source/action refs, and monotonic elapsed diagnostics when available. Correctness assertions target graph behavior and execution counts; elapsed time has no hard pass threshold.

**Rationale:** The paper’s results motivate the workload but do not establish Mantle performance on another host or toolchain.

### 8. Keep network, licenses, and mutable workspaces explicit

**Choice:** Ordinary reference actions run offline and strict-hermetic. The fixture has no commercial license dependency and no mutable workspace. Later commercial profiles must use separate typed network/resource/workspace policy and cannot inherit strong reuse claims from this fixture.

**Rationale:** EDA license servers and incremental work libraries are major hidden-state risks and require dedicated policy rather than permissive example defaults.

## Functional Core / Imperative Shell

- **Core**: profile validation, cohort/source identity, source-closure selection, stage/action planning, dynamic-plan validation, smoke-result validation, invalidation expectation, run comparison, bounds, and non-claim construction.
- **Shell**: local git fixture creation, tool discovery/materialization, file reads, process execution, sandboxing, hashing bytes, store/action-result publication, timing observation, and report rendering.

## Risks / Trade-offs

- A real Verilator cohort makes the deep rail heavier and host-capability-dependent; fast checks should validate plans and fixture contracts without claiming real compilation.
- Generated C++ structure can change across Verilator versions, so cohort upgrades require explicit fixture and expected-plan refresh.
- Per-file actions add scheduling and metadata overhead for tiny fixtures. The rail measures that overhead but is optimized for architectural coverage, not benchmark wins.
- Result conflicts may expose tool nondeterminism rather than cache defects; strong reuse must still fail rather than hide the disagreement.

## Non-Goals

- No Mantle-native HDL parser, elaborator, simulator, UVM framework, regression manager, coverage-merging semantics, synthesis flow, or physical-design flow.
- No real external network, proprietary IP, credentials, waveform corpus, or commercial license server in ordinary validation.
- No assertion that a passing smoke case proves RTL correctness beyond the declared fixture observations.
- No claim that local fixture evidence proves production remote execution or the DVCon paper’s reported speedups.

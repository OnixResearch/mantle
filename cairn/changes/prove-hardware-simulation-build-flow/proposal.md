## Why

Mantle has the generic ingredients described by the DVCon paper [“Using a modern software build system to speed up complex hardware design”](https://dvcon-proceedings.org/wp-content/uploads/1132-Using-a-modern-software-build-system-to-speed-up-complex-hardware-design.pdf): explicit source acquisition, content identity, sandboxed actions, lazy concurrent scheduling, shared action results, remote execution seams, and structured evidence. It does not yet have one maintained hardware-design workload proving those ingredients compose into a staged RTL simulation and smoke-test flow.

Without a bounded reference rail, claims that Mantle is useful for hardware builds remain architectural inference. A real open-source fixture is needed to expose undeclared include paths, generated-source partitioning, simulator/test result identity, selective invalidation, and cache reuse without importing HDL semantics into Mantle core.

## What Changes

- Add a cataloged, open-source SystemVerilog plus C++ reference flow using a pinned Verilator/compiler/linker cohort supplied as explicit store inputs.
- Keep the hardware profile and rule composition in example-owned typed Nickel; lower it to ordinary derivations and `mantle-plan-v1` units rather than adding HDL concepts to Mantle core.
- Model pinned local IP/VIP fixture repositories as demand-driven fixed-output sources and prove that building one testbench does not acquire or realize an unrelated source closure.
- Separate elaboration/code generation, generated C++ translation-unit compilation, simulator linking, and parameterized smoke cases into independently identified actions.
- Materialize each smoke result as a bounded artifact containing verdict, case identity, simulator/action refs, and log refs; failed tests remain failed actions and cannot publish successful shared results.
- Reuse the active `publish-shared-action-results` work for clean-client smoke/build hits and explicit conflicting-result diagnostics instead of inventing an HDL-specific cache.
- Record fresh, single-source-change, and full-shared-hit execution counts, reuse counts, transferred bytes, and elapsed diagnostics without adopting the paper’s speedups as Mantle guarantees.

## Impact

- **Surfaces**: `examples/` catalog and documentation, test-owned HDL/IP/VIP fixtures, example-owned Nickel contracts/helpers, dynamic-plan fixtures, build reports, action-result evidence, and benchmark bundles.
- **Dependencies**: consumes accepted sandbox, source-fetch, dynamic-plan, action-receipt, and build-report primitives; the clean-client shared-hit proof depends on `publish-shared-action-results` completing its focused validation.
- **Boundary**: Mantle remains a frontend-neutral build tool. HDL language semantics, testbench structure, expected functional behavior, and commercial-tool policy remain in the external profile or frontend.
- **Non-claims**: no commercial simulator/synthesis support, no FPGA/ASIC correctness proof, no timing-closure or physical-design claim, no remote-farm throughput claim, no license-server integration, and no promise matching the DVCon/Bazel timing results.
- **Testing**: positive staged builds and smoke cases; negative undeclared include, wrong reference-model, malformed dynamic-plan, stale tool/source identity, cache poisoning, unrelated-source acquisition, selective-invalidation, and conflicting-result fixtures; focused Cairn and examples gates.

# decouple Rust bootstrap patch plan

## Problem

Mantle's source-built Rust provider now carries real compiler-bootstrap fixes: mrustc/minicargo rewrites, first-stage musl wrapper selection, dynamic musl `rustc` relinking for proc-macro loading, LLVM backtrace probe suppression, and runtime wrapper normalization. Those fixes are conceptually Rust/mrustc/musl bootstrap behavior, but today they live as generated shell fragments inside Mantle's provider materializer.

That keeps the current work moving, but it makes Mantle more coupled to compiler internals than its architecture requires. Most of Mantle should depend on a stable source-built toolchain provider contract: executable paths, triples, sysroot layout, proc-macro runtime capability, static/dynamic link policy, and receipt-bound provenance. Native planning, topology execution, and self-build should not need to know whether the provider was repaired by an LLVM CMake flag, a mrustc Makefile rewrite, or a runtime loader wrapper.

## Proposed change

Introduce a deterministic Rust bootstrap patch-plan boundary. The functional core will derive an ordered patch plan from explicit route facts, compiler versions, source-tree identities, and selected bootstrap profile. The imperative provider shell will apply that plan, fail closed on missing anchors or mismatched sources, and record patch-plan evidence in provider receipts.

Keep Mantle's downstream consumers coupled only to the provider capability contract. Rust planning and self-build code should read normalized provider metadata and executable capabilities, not mrustc/LLVM/minicargo patch details. Compiler-bootstrap fixes may continue to evolve inside the provider layer, but changes that preserve the provider contract should not require edits in native Rust planning, topology execution, or self-build consumers.

## Success criteria

- Compiler-bootstrap edits are represented as a deterministic patch plan with a BLAKE3 digest, explicit inputs, ordered operations, and fail-closed anchor checks.
- Patch-plan derivation is testable without filesystem mutation, process execution, ambient environment reads, or provider materialization.
- The provider shell applies the plan and records route facts, compiler identities, source identities, operation summaries, and patch-plan digest in provider evidence.
- Downstream Mantle consumers use the provider capability contract and do not inspect mrustc, minicargo, LLVM, or wrapper implementation details.
- Positive and negative tests cover valid patch plans, missing anchors, mismatched compiler/source versions, unsupported patch operations, and provider-contract preservation.
- Cairn validation/gates and focused Rust tests record the boundary before implementation is claimed complete.

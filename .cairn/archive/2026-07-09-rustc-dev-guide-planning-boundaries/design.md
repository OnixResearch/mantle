## Design

Create a repo-local rustc-dev-guide reference map for Mantle Rust planning. Each entry names the guide section (`hir`, `mir`, `backend`, or parent architecture material), the pinned guide revision or content digest, the Mantle surface that consumes it, and the bounded claim it supports. The accepted README links are discovery-only; reference-map entries are the claim-bearing material.

Keep the validation core pure: parse reference-map records and Rust planning evidence into owned values, validate required fields and claim boundaries in memory, and return deterministic blockers. The CLI/docs/tests own filesystem reads, fixture loading, and rendered diagnostics.

Use the map in three implementation seams:

1. `rust-plan` unit derivation and execution receipts: backend-facing assumptions such as crate type, target triple, codegen backend, linker arguments, metadata hash, `--extern`, `--cfg`, and path remaps must be explicit receipt material when guide-backed behavior is relevant.
2. Compiler-policy adapters: HIR/MIR/rustc_driver inspection providers must bind adapter identity, selected rustc identity, policy digest, guide reference IDs, and non-claims. A policy adapter can claim configured profile compliance only, not program correctness.
3. Source-built Rust provider and patch planning: bootstrap patches that rely on rustc_driver/sysroot/codegen layout must carry source anchors and guide reference IDs so source-provider evidence explains why the patch belongs at that stage.

Fail closed when evidence depends on a moving `main` URL, omits a required guide reference, tries to promote HIR/MIR inspection into compiler-correctness evidence, or relies on backend behavior outside the supported receipt surface.

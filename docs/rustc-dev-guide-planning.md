# rustc-dev-guide planning references

Mantle can use rustc-dev-guide material to explain Rust planning, compiler-policy
adapter, and source-built Rust provider boundaries. Moving `main` links are
discovery links only. Claim-bearing evidence must use a pinned guide revision or
content digest and must state the bounded Mantle surface it supports.

## Reference map shape

Each claim-bearing reference row records:

- guide reference id;
- guide section such as HIR, MIR, backend, rustc_driver, sysroot, crate type,
  linker, metadata, or path remapping;
- pinned revision and optional BLAKE3 content digest;
- Mantle consuming surface; and
- bounded claim.

`rustc_dev_guide::validate_rustc_dev_guide_boundaries` is the pure validator for
already-loaded rows. It also validates Rust planning evidence, compiler-policy
adapter receipts, and source-provider patch plans that cite those references.

## Evidence boundaries

Rust planning evidence must bind backend-facing assumptions such as crate type,
target triple, codegen backend, linker arguments, metadata identity, `--extern`,
`--cfg`, path remaps, toolchain identity, guide reference ids, and output
digests. Compiler-policy adapter receipts must bind adapter identity, selected
rustc identity, policy digest, guide reference ids, invocation status, waiver
summary, and non-claims. Source-provider patch plans must bind the operation,
source anchor, stage, guide reference id, input digest, output digest, and
non-claims.

Passing this validator proves only that guide-backed planning claims are pinned,
reference-linked, and non-overclaiming. It does not prove compiler correctness,
program correctness, full Cargo compatibility, release reproducibility, or
bootstrap correctness.

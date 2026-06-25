# Design: decouple Rust bootstrap patch plan

## Context

Mantle's current source-built Rust provider is doing two jobs in one large imperative generator:

1. It decides which compiler-bootstrap repairs are needed for a selected Rust/mrustc/musl route.
2. It writes and executes shell that mutates extracted sources, compiles helper shims, relinks temporary tools, and materializes the final provider layout.

That was acceptable while discovering frontiers, but it makes review and reuse harder. The decision logic is the part that should be pure and stable. The materializer is the shell that should perform I/O, run commands, copy files, and emit evidence.

## Core boundary

Add a Rust bootstrap patch-plan core that takes owned, explicit inputs:

- Rust source version and expected source-tree identity.
- mrustc version and expected source-tree identity.
- compiler-host triple and provider-target triple.
- selected route capabilities, including musl runtime layout and proc-macro runtime policy.
- source/provider feature facts needed to choose bootstrap repairs.

The core returns a bounded ordered plan. Operation variants should describe intent, not shell syntax. Candidate variants include source text replacement, makefile option append, generated wrapper rewrite, dynamic rustc relink request, compatibility-object compile request, and provider metadata assertion.

The core must be deterministic: no filesystem reads, no process execution, no environment reads, no clocks, and no mutation. It should compute a BLAKE3 digest over the canonical plan input and output so receipts can bind the selected compiler-bootstrap repair set.

## Provider shell

The existing source-provider materializer remains the imperative shell. It should:

- Read/extract verified sources.
- Pass explicit route/source facts into the patch-plan core.
- Apply operations in order.
- Check every anchor before mutation and fail closed with deterministic diagnostics when an expected source shape is absent.
- Compile helper objects and relink temporary tools only when directed by the plan.
- Record patch-plan input digest, output digest, operation summaries, route facts, compiler versions, and source identities in provider evidence.

Generated shell may remain as an implementation detail, but it should be emitted from plan operations rather than from ad-hoc conditionals spread across the materializer.

## Provider contract boundary

Downstream Mantle code should consume a normalized provider contract. The contract should expose facts such as:

- `rustc`, `rustdoc`, `cargo`, linker, archive, and ranlib executable paths.
- host/target triples and sysroot root.
- proc-macro runtime support for the host compiler.
- static executable support and dynamic/shared-library support policy.
- provider source/provenance digests and patch-plan digest.

Downstream consumers must not branch on mrustc/minicargo/LLVM internals. If a compiler-bootstrap repair changes while the provider still satisfies the same contract, native Rust planning, topology execution, and self-build consumers should not require code changes.

## Validation

Focused validation should start with pure core tests:

- A supported musl-host route produces the expected operation classes and stable digest.
- A GNU-host route omits musl-host-only operations.
- Missing expected anchors and mismatched source versions fail before mutation.
- Unsupported operation kinds fail closed in the shell adapter.

Shell/provider tests should then prove operation application records evidence and preserves the existing provider contract. A real provider rerun remains the final long-running proof that refactoring did not regress the source-built Rust frontier.
